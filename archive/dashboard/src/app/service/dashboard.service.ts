import {
    HttpClient,
    HttpContext,
    HttpErrorResponse,
} from "@angular/common/http";
import { Injectable } from "@angular/core";
import {
    BehaviorSubject,
    catchError,
    finalize,
    map,
    Observable,
    tap,
} from "rxjs";
import { DashboardErrorResponse } from "../common/error/dashboard-http.error";
import {
    CAPTURED_EXCHANGE,
    CapturedExchangeHolder,
} from "../common/http/captured-exchange";
import { UrlResult } from "../common/model/api/url_result";
import ConvQuery from "../common/model/core/conv-query";
import {
    ResponseBody,
    ResponseBodyScheme,
} from "../common/response/response";

@Injectable()
export class DashboardService {
    public static readonly HEALTH_ENDPOINT = `/actuator/healthy`;
    public static readonly REDIS_ENDPOINT = `/actuator/redis`;
    public static readonly BUILD_URL = `/api/build-url`;

    loading: BehaviorSubject<boolean> = new BehaviorSubject<boolean>(false);
    loading$ = this.loading.asObservable();

    error: BehaviorSubject<DashboardErrorResponse | null> = new BehaviorSubject<DashboardErrorResponse | null>(null);
    error$ = this.error.asObservable();

    data = new BehaviorSubject<UrlResult | null>(null);
    data$ = this.data.asObservable();

    public constructor(
        private http: HttpClient,
    ) {
    }

    public healthCheck(): Observable<ResponseBody | null> {
        return this.http.get<ResponseBody>(DashboardService.HEALTH_ENDPOINT)
                   .pipe(
                       map(response => ResponseBody.deserialize(ResponseBodyScheme.parse(response), undefined)),
                   );
    }

    public redisCheck(): Observable<ResponseBody<string> | null> {
        return this.http.get<ResponseBody<string>>(DashboardService.REDIS_ENDPOINT)
                   .pipe(
                       map(response => ResponseBody.deserialize(ResponseBodyScheme.parse(response), undefined)),
                   );
    }

    public getSubscription(query: ConvQuery): Observable<ResponseBody<UrlResult>> {
        const path = `${DashboardService.BUILD_URL}?${query.toString()}`;
        console.log("请求", query, path);
        this.loading.next(true);
        const holder: CapturedExchangeHolder = { exchange: null };
        const context = new HttpContext().set(CAPTURED_EXCHANGE, holder);
        return this.http.get(path, { context }).pipe(
            map(response => {
                const body = ResponseBody.deserialize(ResponseBodyScheme.parse(response), UrlResult);
                if (body === null) {
                    throw new Error("Unexpected null: failed to deserialize ResponseBody<UrlResult>");
                }
                return body;
            }),
            // 请求成功时清除错误信息
            tap(response => {
                console.log("请求成功", response);
                if (response.isOk()) {
                    this.error.next(null);
                    this.data.next(response.data!);
                } else {
                    console.error("业务错误", response);
                    this.data.next(null);
                    this.error.next(DashboardErrorResponse.fromBiz(response, holder));
                }
            }),
            // 错误只在 HTTP 内部处理，吞掉，不打断主流
            catchError((err: HttpErrorResponse) => {
                console.error("请求失败", err);
                const dashboardError = DashboardErrorResponse.fromHttp(err, holder);
                this.error.next(dashboardError);
                throw dashboardError;
            }),
            // 结束（成功/失败/取消）：关 loading
            finalize(() => {
                this.loading.next(false);
            }),
        );
    }
}
