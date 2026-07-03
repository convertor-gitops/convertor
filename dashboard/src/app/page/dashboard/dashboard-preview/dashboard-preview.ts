import { ChangeDetectionStrategy, Component, computed, effect, inject, signal, WritableSignal } from "@angular/core";
import { ActivatedRoute, Router } from "@angular/router";
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
import { WorkbenchEmpty } from "../../shared/workbench-empty/workbench-empty";

interface RuleChoice {
    row: MergeRow;
    side: WritableSignal<"local" | "new">;
    enabled: WritableSignal<boolean>;
    diff: InlineDiff | null;
}

type ApplyStage = "idle" | "confirm" | "writing" | "done";

const STATUS_LABEL: Record<MergeStatus, string> = {
    unchanged: "未变",
    changed: "变更",
    added: "新增",
    removed: "删除",
};

@Component({
    selector: "app-dashboard-preview",
    imports: [WorkbenchEmpty],
    templateUrl: "./dashboard-preview.html",
    styleUrl: "./dashboard-preview.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardPreview {
    private fs = inject(LocalFsService);
    private previewCtx = inject(PreviewContextService);
    private router = inject(Router);
    private route = inject(ActivatedRoute);

    readonly ctx = this.previewCtx.current;
    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly needsAuth = signal(false);
    readonly applyStage = signal<ApplyStage>("idle");

    readonly headerOld = signal("");
    readonly headerNew = signal("");
    readonly headerSide = signal<"local" | "new">("new");
    readonly headerDiff = computed<InlineDiff>(() => inlineDiff(this.headerOld(), this.headerNew()));
    readonly choices = signal<RuleChoice[]>([]);

    private mainContent = "";
    private rulesParsed: ParsedBlock | null = null;

    readonly dirName = computed(() => this.ctx()?.binding.dirName ?? "");
    readonly mainName = computed(() => this.ctx()?.binding.mainProfile ?? "");
    readonly rulesName = computed(() => this.ctx()?.binding.rulesProfile ?? "");

    readonly changeCount = computed(() => {
        const header = this.headerSide() === "new" && this.headerOld() !== this.headerNew() ? 1 : 0;
        const rules = this.choices().filter(c => c.enabled() && c.side() === "new" && c.row.status !== "unchanged").length;
        return header + rules;
    });

    readonly changeSummary = computed(() => `${this.changeCount()} changes · 2 files`);

    readonly mergedMain = computed(() => {
        const line = this.headerSide() === "local" ? this.headerOld() : this.headerNew();
        return patchSurgeHeader(this.mainContent, line);
    });

    readonly mergedPreview = computed(() => renderManagedBlock(this.currentEntries()));
    readonly previewLines = computed(() => this.mergedPreview().split("\n").map((text, i) => ({
        no: i + 1,
        text,
        kind: text.startsWith("#") ? "comment" : "new",
    })));

    readonly summaryChips = computed(() => {
        const choices = this.choices();
        return [
            { text: `${choices.filter(c => c.enabled() && c.side() === "new").length} × NEW`, kind: "new" },
            { text: `${choices.filter(c => c.enabled() && c.side() === "local").length} × OLD`, kind: "old" },
            { text: `${choices.filter(c => !c.enabled()).length} 停用`, kind: "off" },
        ].filter(c => !c.text.startsWith("0 "));
    });

    constructor() {
        effect(() => {
            const ctx = this.ctx();
            if (!ctx) {
                this.reset();
                return;
            }
            void this.load(ctx);
        });
    }

    async load(ctx = this.ctx()): Promise<void> {
        if (!ctx) {
            return;
        }
        this.loading.set(true);
        this.error.set(null);
        this.needsAuth.set(false);
        this.applyStage.set("idle");
        try {
            if (!(await this.fs.ensurePermission(ctx.binding.dirHandle, "readwrite"))) {
                this.error.set("未获得目录写入授权");
                this.needsAuth.set(true);
                this.loading.set(false);
                return;
            }

            this.mainContent = await this.fs.readFile(ctx.binding.dirHandle, ctx.binding.mainProfile!);
            this.headerOld.set(this.mainContent.split("\n")[0] ?? "");
            this.headerNew.set(buildHeaderLine(ctx.urlResult.profile_url));
            this.headerSide.set("new");

            const rulesContent = await this.fs.readFile(ctx.binding.dirHandle, ctx.binding.rulesProfile!);
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

    statusLabel(status: MergeStatus): string {
        return STATUS_LABEL[status];
    }

    enableLabel(status: MergeStatus): string {
        return status === "removed" ? "保留" : "启用";
    }

    setSide(choice: RuleChoice, side: "local" | "new"): void {
        if (choice.row.status === "added" || choice.row.status === "removed" || !choice.enabled()) {
            return;
        }
        choice.side.set(side);
    }

    toggle(choice: RuleChoice): void {
        choice.enabled.set(!choice.enabled());
    }

    askApply(): void {
        this.applyStage.set("confirm");
    }

    cancelApply(): void {
        this.applyStage.set("idle");
    }

    async doApply(): Promise<void> {
        const ctx = this.ctx();
        if (!ctx || !this.rulesParsed) {
            return;
        }
        this.applyStage.set("writing");
        try {
            const fullRules = renderRuleBlock(this.rulesParsed, this.currentEntries());
            await this.fs.writeFile(ctx.binding.dirHandle, ctx.binding.mainProfile!, this.mergedMain());
            await this.fs.writeFile(ctx.binding.dirHandle, ctx.binding.rulesProfile!, fullRules);
            this.applyStage.set("done");
        } catch (e) {
            this.error.set(this.message(e));
            this.applyStage.set("idle");
        }
    }

    finish(): void {
        this.previewCtx.clear();
        void this.router.navigate(["../"], { relativeTo: this.route });
    }

    private reset(): void {
        this.loading.set(false);
        this.error.set(null);
        this.needsAuth.set(false);
        this.applyStage.set("idle");
        this.headerOld.set("");
        this.headerNew.set("");
        this.choices.set([]);
        this.mainContent = "";
        this.rulesParsed = null;
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

    private message(e: unknown): string {
        return e instanceof Error ? e.message : String(e);
    }
}
