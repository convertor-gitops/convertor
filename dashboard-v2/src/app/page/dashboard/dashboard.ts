import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { BreakpointObserver } from "@angular/cdk/layout";
import { toSignal } from "@angular/core/rxjs-interop";
import { map } from "rxjs";
import { DashboardService } from "../../service/dashboard.service";
import { MetadataService } from "../../service/metadata.service";
import { DashboardError } from "./dashboard-error/dashboard-error";
import { DashboardLocal } from "./dashboard-local/dashboard-local";
import { DashboardParam } from "./dashboard-param/dashboard-param";
import { DashboardStatus } from "./dashboard-status/dashboard-status";
import { DashboardSubs } from "./dashboard-subs/dashboard-subs";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardParam,
        DashboardSubs,
        DashboardLocal,
        DashboardError,
        DashboardStatus,
        AsyncPipe,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        DashboardService,
    ],
})
export class Dashboard {
    private metadata = inject(MetadataService);
    private breakpointObserver = inject(BreakpointObserver);
    dashboardService: DashboardService = inject(DashboardService);

    readonly version = this.metadata.version;
    readonly isMobile = toSignal(
        this.breakpointObserver.observe("(max-width: 760px)").pipe(
            map(state => state.matches),
        ),
        { initialValue: false },
    );
    error$ = this.dashboardService.error$;
}
