import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject, OnInit } from "@angular/core";
import { MatIcon } from "@angular/material/icon";
import { MatTooltip } from "@angular/material/tooltip";
import { BehaviorSubject, forkJoin } from "rxjs";
import { BackendStatus } from "../../../common/model/api/backend-status";
import { StatusService } from "../../../service/status.service";

@Component({
    selector: "app-dashboard-status",
    imports: [AsyncPipe, MatIcon, MatTooltip],
    templateUrl: "./dashboard-status.html",
    styleUrl: "./dashboard-status.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardStatus implements OnInit {
    private statusService = inject(StatusService);

    health$ = new BehaviorSubject<boolean | null>(null);
    backendStatus$ = new BehaviorSubject<BackendStatus | null>(null);
    error$ = new BehaviorSubject<string | null>(null);

    ngOnInit(): void {
        this.refresh();
    }

    refresh(): void {
        this.error$.next(null);
        this.health$.next(null);
        this.backendStatus$.next(null);
        forkJoin({
            health: this.statusService.healthCheck(),
            status: this.statusService.getStatus(),
        }).subscribe({
            next: ({ health, status }) => {
                this.health$.next(health);
                this.backendStatus$.next(status);
                if (!status) {
                    this.error$.next("无法连接状态接口");
                }
            },
            error: () => this.error$.next("无法连接后端"),
        });
    }

    healthTip(healthy: boolean): string {
        return healthy ? "/actuator/healthy 正常" : "/actuator/healthy 异常";
    }

    /** hover 详情：API 版本 + 各服务一行（healthy 打勾，未配置/异常给原因） */
    tooltip(status: BackendStatus): string {
        const lines = [`API v${status.version}`];
        for (const svc of status.services) {
            lines.push(`${svc.healthy ? "✓" : "•"} ${svc.healthy ? svc.name : (svc.message ?? svc.name)}`);
        }
        return lines.join("\n");
    }
}
