import { ChangeDetectionStrategy, Component, computed, inject, signal } from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { ActivatedRoute, Router } from "@angular/router";
import { ProxyClient } from "../../../common/model/core/proxy-client";
import { SurgeBinding } from "../../../common/model/local/surge-binding";
import { ClientContextService } from "../../../service/client-context.service";
import { DashboardService } from "../../../service/dashboard.service";
import { LocalFsService } from "../../../service/local-fs.service";
import { MetadataService } from "../../../service/metadata.service";
import { PreviewContextService } from "../../../service/preview-context.service";
import { WorkbenchEmpty } from "../../shared/workbench-empty/workbench-empty";

type FileRole = "main" | "rules";

@Component({
    selector: "app-dashboard-sync",
    imports: [
        WorkbenchEmpty,
    ],
    templateUrl: "./dashboard-sync.html",
    styleUrl: "./dashboard-sync.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardSync {
    private fs = inject(LocalFsService);
    private clientContext = inject(ClientContextService);
    private dashboardService = inject(DashboardService);
    private metadata = inject(MetadataService);
    private previewCtx = inject(PreviewContextService);
    private router = inject(Router);
    private route = inject(ActivatedRoute);

    readonly supported = this.fs.supported;
    readonly client = this.clientContext.client;
    readonly binding = signal<SurgeBinding | null>(null);
    readonly files = signal<string[]>([]);
    readonly error = signal<string | null>(null);
    /** 句柄存在但权限已失效（常见于浏览器重启/刷新后）：需要一次用户手势才能再弹出系统授权框 */
    readonly needsAuth = signal(false);

    readonly urlResult = toSignal(this.dashboardService.data$, { initialValue: null });
    readonly isSurge = computed(() => this.client() === ProxyClient.Surge);

    readonly canPreview = computed(() => {
        const b = this.binding();
        return this.isSurge() && !!b && !this.needsAuth() && !!b.mainProfile && !!b.rulesProfile && !!this.urlResult();
    });

    constructor() {
        void this.restore();
    }

    private async restore(): Promise<void> {
        if (!this.supported) {
            return;
        }
        try {
            const saved = await this.fs.loadBinding();
            if (!saved) {
                return;
            }
            this.binding.set(saved);
            if (await this.fs.ensurePermission(saved.dirHandle, "read")) {
                this.files.set(await this.fs.listFiles(saved.dirHandle));
            } else {
                this.needsAuth.set(true);
            }
        } catch (e) {
            this.error.set(this.message(e));
        }
    }

    /** 重新授权按钮：点击本身就是用户手势，浏览器才会真正弹出系统权限框 */
    async reauthorize(): Promise<void> {
        const b = this.binding();
        if (!b) {
            return;
        }
        this.error.set(null);
        try {
            if (await this.fs.ensurePermission(b.dirHandle, "readwrite")) {
                this.needsAuth.set(false);
                this.files.set(await this.fs.listFiles(b.dirHandle));
            }
        } catch (e) {
            this.error.set(this.message(e));
        }
    }

    async pickDir(): Promise<void> {
        this.error.set(null);
        try {
            const dir = await this.fs.pickDirectory();
            const files = await this.fs.listFiles(dir);
            const binding: SurgeBinding = {
                dirHandle: dir,
                dirName: dir.name,
                client: this.client(),
                mainProfile: this.guess(files, [".conf"]),
                rulesProfile: this.guess(files, [".dconf", "rule"]),
            };
            this.needsAuth.set(false);
            this.files.set(files);
            this.binding.set(binding);
            await this.fs.saveBinding(binding);
        } catch (e) {
            if ((e as DOMException)?.name !== "AbortError") {
                this.error.set(this.message(e));
            }
        }
    }

    roleOf(name: string): FileRole | null {
        const b = this.binding();
        if (!b) {
            return null;
        }
        if (b.mainProfile === name) {
            return "main";
        }
        if (b.rulesProfile === name) {
            return "rules";
        }
        return null;
    }

    /** 点击分段按钮：已是该角色则取消，否则设为该角色（同角色会从别的文件移走） */
    toggleRole(name: string, role: FileRole): void {
        void this.setRole(name, this.roleOf(name) === role ? null : role);
    }

    private async setRole(name: string, role: FileRole | null): Promise<void> {
        const b = this.binding();
        if (!b) {
            return;
        }
        const next: SurgeBinding = { ...b };
        if (next.mainProfile === name) {
            next.mainProfile = null;
        }
        if (next.rulesProfile === name) {
            next.rulesProfile = null;
        }
        if (role === "main") {
            next.mainProfile = name;
        }
        if (role === "rules") {
            next.rulesProfile = name;
        }
        this.binding.set(next);
        await this.fs.saveBinding(next);
    }

    preview(): void {
        const binding = this.binding();
        const urlResult = this.urlResult();
        if (!binding || !urlResult || !binding.mainProfile || !binding.rulesProfile) {
            return;
        }
        this.previewCtx.set({ binding, urlResult, version: this.metadata.version });
        void this.router.navigate(["preview"], { relativeTo: this.route.parent });
    }

    isEligible(name: string): boolean {
        const lower = name.toLowerCase();
        return lower.endsWith(".conf") || lower.endsWith(".dconf");
    }

    fileSizeLabel(_name: string): string {
        return "—";
    }

    private guess(files: string[], hints: string[]): string | null {
        return files.find(f => hints.some(h => f.toLowerCase().includes(h))) ?? null;
    }

    private message(e: unknown): string {
        return e instanceof Error ? e.message : String(e);
    }
}
