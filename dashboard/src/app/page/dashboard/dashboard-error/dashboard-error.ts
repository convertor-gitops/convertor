import { AsyncPipe } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, Component, effect, inject, model } from "@angular/core";
import { MatCardContent, MatCardHeader, MatCardTitle } from "@angular/material/card";
import { MatChip } from "@angular/material/chips";
import { MatDivider } from "@angular/material/divider";
import {
    MatAccordion,
    MatExpansionPanel,
    MatExpansionPanelDescription,
    MatExpansionPanelHeader,
    MatExpansionPanelTitle,
} from "@angular/material/expansion";
import { combineLatest, filter, map, Observable, shareReplay, startWith, tap, withLatestFrom } from "rxjs";
import { RequestSnapshot } from "../../../common/response/request";
import { DashboardService } from "../../../service/dashboard.service";
import { DashboardPanel } from "../dashboard-panel/dashboard-panel";
import { ApiResponse } from "../../../common/response/response";

@Component({
    selector: "app-dashboard-error",
    imports: [
        DashboardPanel,
        MatCardHeader,
        MatCardContent,
        MatAccordion,
        MatExpansionPanel,
        MatExpansionPanelHeader,
        MatExpansionPanelTitle,
        MatExpansionPanelDescription,
        AsyncPipe,
        MatDivider,
        MatChip,
        MatCardTitle,

    ],
    templateUrl: "./dashboard-error.html",
    styleUrl: "./dashboard-error.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardError {
    dashboardService: DashboardService = inject(DashboardService);

    dashboardHttpError$ = this.dashboardService.error$;

    httpErrorResponse$ = this.dashboardHttpError$.pipe(
        map((error) => error?.cause),
    );

    apiResponse$ = this.httpErrorResponse$.pipe(
        map((error) => error?.error),
        filter((error) => !!error),
        map((error) => ApiResponse.deserialize(error)),
        filter((response) => response != null),
        shareReplay(1),
    );

    clientRequest$ = this.httpErrorResponse$.pipe(
        filter((errorResponse) => !!(errorResponse?.url)),
        map((errorResponse) => <[HttpErrorResponse, string]>[errorResponse!, errorResponse!.url!]),
        withLatestFrom(this.dashboardHttpError$.pipe(
            filter((error) => !!(error?.method)),
            map((error) => error!.method!),
        )),
        map(([[errorResponse, url], method]) => {
            const parsedUrl = new URL(url);
            const headers = new Map<string, string>();
            errorResponse.headers.keys().forEach(key => {
                headers.set(key, errorResponse.headers.get(key) ?? "");
            });
            return new RequestSnapshot(
                method,
                parsedUrl.protocol,
                parsedUrl.host,
                parsedUrl.pathname + parsedUrl.search,
                headers,
            );
        }),
    );
    serverRequest$ = this.apiResponse$.pipe(
        map((response) => response.request),
    );
    requests$: Observable<Record<string, RequestSnapshot | null>> = combineLatest([
        this.clientRequest$.pipe(startWith(null)),
        this.serverRequest$.pipe(startWith(null)),
    ]).pipe(
        map(([client, server]) => {
            return {
                "客户端发出请求": client,
                "服务端收到请求": server,
            };
        }),
    );

    errorMessage$ = this.httpErrorResponse$.pipe(
        map((error) => error?.message ?? ""),
    );

    // response messages
    mainMessage$ = this.apiResponse$.pipe(
        map((response) => response.messages[0] ?? ""),
        tap(console.log),
    );
    causeMessages$ = this.apiResponse$.pipe(
        map((response) => response.messages.slice(1)),
    );

    // ui
    clientRequestCollapsed = model(false);
    protected readonly Object = Object;

    constructor() {
        effect(() => {
            console.log(this.clientRequestCollapsed());
        });
    }

    afterCollapse() {
        console.log("afterCollapse");
        this.clientRequestCollapsed.set(true);
    }

    afterExpand() {
        console.log("afterExpand");
        this.clientRequestCollapsed.set(false);
    }
}
