import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { DashboardService } from "../../service/dashboard.service";
import { MetadataService } from "../../service/metadata.service";
import { NoContent } from "../shared/no-content/no-content";
import { DashboardError } from "./dashboard-error/dashboard-error";
import { DashboardParam } from "./dashboard-param/dashboard-param";
import { DashboardSubs } from "./dashboard-subs/dashboard-subs";
import { DashboardStatus } from "./dashboard-status/dashboard-status";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardSubs,
        DashboardParam,
        AsyncPipe,
        NoContent,
        DashboardError,
        DashboardStatus,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        DashboardService,
    ],
})
export class Dashboard {
    dashboardService: DashboardService = inject(DashboardService);
    metadataService: MetadataService = inject(MetadataService);
    version = this.metadataService.version;

    error$ = this.dashboardService.error$;
    data$ = this.dashboardService.data$;
}
