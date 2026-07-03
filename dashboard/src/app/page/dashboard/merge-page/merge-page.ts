import { ChangeDetectionStrategy, Component, computed, inject, signal, WritableSignal } from "@angular/core";
import { MatIcon } from "@angular/material/icon";
import { MatProgressSpinner } from "@angular/material/progress-spinner";
import { Router } from "@angular/router";
import {
    buildHeaderLine,
    entriesFromUrls,
    MergeRow,
    MergeStatus,
    mergeRuleEntries,
    parseRuleBlock,
    ParsedBlock,
    patchSurgeHeader,
    renderManagedBlock,
    renderRuleBlock,
    RuleEntry,
} from "../../../common/patch/surge-patch";
import { inlineDiff, InlineDiff } from "../../../common/util/inline-diff";
import { LocalFsService } from "../../../service/local-fs.service";
import { PreviewContextService } from "../../../service/preview-context.service";

interface RuleChoice {
    row: MergeRow;
    side: WritableSignal<"local" | "new">;
    enabled: WritableSignal<boolean>;
    /** 本地 vs 新版 URL 的字符级 diff（仅 changed/unchanged 两边都在时有值） */
    diff: InlineDiff | null;
}

const STATUS_LABEL: Record<MergeStatus, string> = {
    unchanged: "未变",
    changed: "变更",
    added: "新增",
    removed: "删除",
};

@Component({
    selector: "app-merge-page",
    imports: [MatIcon, MatProgressSpinner],
    templateUrl: "./merge-page.html",
    styleUrl: "./merge-page.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MergePage {
    private fs = inject(LocalFsService);
    private previewCtx = inject(PreviewContextService);
    private router = inject(Router);

    private ctx = this.previewCtx.get();

    readonly dirName = this.ctx?.binding.dirName ?? "";
    readonly mainName = this.ctx?.binding.mainProfile ?? "";
    readonly rulesName = this.ctx?.binding.rulesProfile ?? "";

    readonly loading = signal(true);
    readonly error = signal<string | null>(null);
    readonly busy = signal(false);
    /** 句柄权限已失效：load() 不在用户手势内运行，无法自动弹出系统授权框，需要按钮重试 */
    readonly needsAuth = signal(false);

    readonly headerOld = signal("");
    readonly headerNew = signal("");
    readonly headerSide = signal<"local" | "new">("new");
    readonly headerDiff = computed<InlineDiff>(() => inlineDiff(this.headerOld(), this.headerNew()));

    readonly choices = signal<RuleChoice[]>([]);

    private mainContent = "";
    private rulesParsed: ParsedBlock | null = null;

    constructor() {
        if (!this.ctx) {
            // 直接访问 /preview 而无上下文：退回主页
            void this.router.navigate(["/"]);
            return;
        }
        void this.load();
    }

    /** 也用作重新授权按钮的点击处理：重新调用本身是用户手势触发的，requestPermission 才会真正弹窗 */
    async load(): Promise<void> {
        const ctx = this.ctx!;
        this.loading.set(true);
        this.error.set(null);
        this.needsAuth.set(false);
        try {
            if (!(await this.fs.ensurePermission(ctx.binding.dirHandle, "readwrite"))) {
                this.error.set("未获得目录写入授权");
                this.needsAuth.set(true);
                this.loading.set(false);
                return;
            }

            this.mainContent = await this.fs.readFile(ctx.binding.dirHandle, this.mainName);
            this.headerOld.set(this.mainContent.split("\n")[0] ?? "");
            this.headerNew.set(buildHeaderLine(ctx.urlResult.profile_url));

            const rulesContent = await this.fs.readFile(ctx.binding.dirHandle, this.rulesName);
            this.rulesParsed = parseRuleBlock(rulesContent);
            const newEntries = entriesFromUrls(ctx.urlResult.rule_provider_urls, ctx.version);
            const rows = mergeRuleEntries(this.rulesParsed.entries, newEntries);
            this.choices.set(rows.map(row => ({
                row,
                side: signal<"local" | "new">(row.status === "removed" ? "local" : "new"),
                enabled: signal<boolean>(this.defaultEnabled(row)),
                diff: row.oldEntry && row.newEntry ? inlineDiff(row.oldEntry.url, row.newEntry.url) : null,
            })));
            this.loading.set(false);
        } catch (e) {
            this.error.set(this.message(e));
            this.loading.set(false);
        }
    }

    private defaultEnabled(row: MergeRow): boolean {
        if (row.status === "added") {
            return true;
        }
        if (row.status === "removed") {
            return false;
        }
        return !row.oldEntry!.disabled;
    }

    private currentEntries(): RuleEntry[] {
        const entries: RuleEntry[] = [];
        for (const c of this.choices()) {
            const enabled = c.enabled();
            if (c.row.status === "removed") {
                if (enabled) {
                    entries.push(c.row.oldEntry!);
                }
                continue;
            }
            const base = c.side() === "local" ? c.row.oldEntry! : c.row.newEntry!;
            entries.push({ ...base, disabled: !enabled });
        }
        return entries;
    }

    readonly mergedMain = computed(() => {
        const line = this.headerSide() === "local" ? this.headerOld() : this.headerNew();
        return patchSurgeHeader(this.mainContent, line);
    });

    /** 总预览：仅 START/END 受管辖块 */
    readonly mergedPreview = computed(() => renderManagedBlock(this.currentEntries()));

    statusLabel(status: MergeStatus): string {
        return STATUS_LABEL[status];
    }

    enableLabel(status: MergeStatus): string {
        return status === "removed" ? "保留" : "启用";
    }

    setSide(choice: RuleChoice, side: "local" | "new"): void {
        if (choice.row.status === "added" || choice.row.status === "removed") {
            return;
        }
        choice.side.set(side);
    }

    toggle(choice: RuleChoice): void {
        choice.enabled.set(!choice.enabled());
    }

    async apply(): Promise<void> {
        const ctx = this.ctx!;
        this.busy.set(true);
        try {
            const fullRules = renderRuleBlock(this.rulesParsed!, this.currentEntries());
            await this.fs.writeFile(ctx.binding.dirHandle, this.mainName, this.mergedMain());
            await this.fs.writeFile(ctx.binding.dirHandle, this.rulesName, fullRules);
            this.back();
        } catch (e) {
            this.error.set(this.message(e));
            this.busy.set(false);
        }
    }

    back(): void {
        this.previewCtx.clear();
        void this.router.navigate(["/"]);
    }

    private message(e: unknown): string {
        return e instanceof Error ? e.message : String(e);
    }
}
