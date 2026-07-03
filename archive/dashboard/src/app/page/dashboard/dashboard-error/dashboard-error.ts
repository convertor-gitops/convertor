import { AsyncPipe } from "@angular/common";
import {
    ChangeDetectionStrategy,
    Component,
    inject,
    model,
} from "@angular/core";
import {
    MatCardContent,
    MatCardHeader,
    MatCardTitle,
} from "@angular/material/card";
import { MatChip } from "@angular/material/chips";
import { MatDivider } from "@angular/material/divider";
import {
    MatAccordion,
    MatExpansionPanel,
    MatExpansionPanelDescription,
    MatExpansionPanelHeader,
    MatExpansionPanelTitle,
} from "@angular/material/expansion";
import {
    filter,
    map,
    shareReplay,
} from "rxjs";
import { DashboardErrorResponse } from "../../../common/error/dashboard-http.error";
import { ResponseBody } from "../../../common/response/response";
import { DashboardService } from "../../../service/dashboard.service";
import { DashboardPanel } from "../dashboard-panel/dashboard-panel";

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

    /** 非 null 的错误对象流 */
    dashboardError$ = this.dashboardService.error$.pipe(
        filter((e): e is DashboardErrorResponse => e != null),
        shareReplay(1),
    );

    /** 客户端请求视角（ResponseBody：status=HTTP状态, request=客户端RequestBody） */
    clientRequest$ = this.dashboardError$.pipe(
        map((e) => e.clientRequest),
    );

    /** 服务端响应（ResponseBody：status=业务状态, request=服务端RequestBody, messages=错误链） */
    responseBody$ = this.dashboardError$.pipe(
        map((e) => e.responseBody),
    );

    /** 请求对比列表：客户端 vs 服务端 */
    requests$ = this.dashboardError$.pipe(
        map((e) => <{ desc: string, body: ResponseBody<unknown> | null; }[]>[
            { desc: "客户端发出请求", body: e.clientRequest },
            { desc: "服务端收到请求", body: e.responseBody },
        ]),
    );

    /** 错误链主消息 */
    mainMessage$ = this.dashboardError$.pipe(
        map((e) => e.responseBody?.messages[0] ?? ""),
    );

    /** 错误链 cause */
    causeMessages$ = this.dashboardError$.pipe(
        map((e) => e.responseBody?.messages.slice(1) ?? []),
    );

    // ui
    clientRequestCollapsed = model(false);

    afterCollapse() {
        this.clientRequestCollapsed.set(true);
    }

    afterExpand() {
        this.clientRequestCollapsed.set(false);
    }
}
