import { ChangeDetectionStrategy, Component, computed, inject, signal } from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { ConvUrl } from "../../../common/model/core/conv-url";
import { DashboardService } from "../../../service/dashboard.service";
import { CopyAction } from "../../shared/copy-action/copy-action";
import { WorkbenchEmpty } from "../../shared/workbench-empty/workbench-empty";
import { WorkbenchSection } from "../../shared/workbench-section/workbench-section";

interface LinkRow {
    label: string;
    url: string;
    tag?: string;
    primary?: boolean;
}

interface ProviderGroup {
    label: string;
    items: LinkRow[];
}

@Component({
    selector: "app-dashboard-subs",
    imports: [CopyAction, WorkbenchEmpty, WorkbenchSection],
    templateUrl: "./dashboard-subs.html",
    styleUrl: "./dashboard-subs.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardSubs {
    private dashboardService = inject(DashboardService);
    private readonly result = toSignal(this.dashboardService.data$, { initialValue: null });

    readonly hasResult = computed(() => !!this.result());

    readonly rows = computed<LinkRow[]>(() => {
        const r = this.result();
        if (!r) {
            return [];
        }
        return [
            { label: "原始订阅配置", tag: "RAW", url: r.raw_url.toString() },
            { label: "转换前订阅配置", tag: "ORIGINAL", url: r.original_url.toString() },
            { label: "转换后订阅配置", tag: "PROFILE", url: r.profile_url.toString(), primary: true },
        ];
    });

    readonly groups = computed<ProviderGroup[]>(() => {
        const r = this.result();
        if (!r) {
            return [];
        }
        const groups: ProviderGroup[] = [];
        if (r.rule_provider_urls.length) {
            groups.push({
                label: "Rule Provider",
                items: r.rule_provider_urls.map(u => ({ label: this.policyName(u), url: u.toString() })),
            });
        }
        if (r.proxy_provider_urls.length) {
            groups.push({
                label: "Proxy Provider",
                items: r.proxy_provider_urls.map(u => ({ label: u.query?.proxy_provider_name ?? "proxy", url: u.toString() })),
            });
        }
        return groups;
    });

    readonly collapsed = signal<ReadonlySet<string>>(new Set());

    isCollapsed(label: string): boolean {
        return this.collapsed().has(label);
    }

    toggleGroup(label: string): void {
        const next = new Set(this.collapsed());
        if (next.has(label)) {
            next.delete(label);
        } else {
            next.add(label);
        }
        this.collapsed.set(next);
    }

    private policyName(u: ConvUrl): string {
        const policy = u.query?.policy;
        if (!policy) {
            return "—";
        }
        return policy.option ? `${policy.name} / ${policy.option}` : policy.name;
    }
}
