import { CdkCopyToClipboard } from "@angular/cdk/clipboard";
import { AsyncPipe } from "@angular/common";
import {
    Component,
    inject,
} from "@angular/core";
import { ReactiveFormsModule } from "@angular/forms";
import {
    MatCard,
    MatCardContent,
    MatCardHeader,
    MatCardTitle,
} from "@angular/material/card";
import { MatDivider } from "@angular/material/divider";
import {
    filter,
    map,
    Observable,
} from "rxjs";
import { UrlResult } from "../../../common/model/api/url_result";
import { ConvUrl } from "../../../common/model/core/conv-url";
import { DashboardService } from "../../../service/dashboard.service";
import { IconButton } from "../../shared/icon-button/icon-button";
import { DashboardPanel } from "../dashboard-panel/dashboard-panel";

@Component({
    selector: "app-dashboard-subs",
    imports: [
        ReactiveFormsModule,
        DashboardPanel,
        MatCardHeader,
        MatCardTitle,
        MatCardContent,
        MatCard,
        AsyncPipe,
        MatDivider,
        IconButton,
        CdkCopyToClipboard,
    ],
    templateUrl: "./dashboard-subs.html",
    styleUrl: "./dashboard-subs.scss",
})
export class DashboardSubs {
    dashboardService: DashboardService = inject(DashboardService);

    urlResult$ = this.dashboardService.data$;
    urls$: Observable<ConvUrl[]> = this.urlResult$.pipe(
        filter((urlResult): urlResult is UrlResult => !!urlResult),
        map(urlResult => [
            urlResult.original_url,
            urlResult.raw_url,
            urlResult.profile_url,
            ...urlResult.proxy_provider_urls,
            ...urlResult.rule_provider_urls,
        ]),
    );
}
