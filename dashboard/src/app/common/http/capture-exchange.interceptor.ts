import {
    HttpErrorResponse,
    HttpInterceptorFn,
    HttpResponse,
} from "@angular/common/http";
import { tap } from "rxjs";
import {
    CAPTURED_EXCHANGE,
    CapturedExchange,
} from "./captured-exchange";

/**
 * 拦截器：自动捕获每次 HTTP 请求/响应的完整信息。
 *
 * 工作原理：
 *   1. 从 HttpRequest 创建 CapturedExchange（记录 method / url / request headers）
 *   2. 将 exchange 写入 HttpContext holder（通过 CAPTURED_EXCHANGE token）
 *   3. 在响应/错误回调中补充 response 信息（status / headers）
 *
 * service 层只需在请求前创建 holder 并通过 HttpContext 传入，
 * 请求完成后即可从 holder 中读取完整的交换记录。
 */
export const captureExchangeInterceptor: HttpInterceptorFn = (req, next) => {
    const exchange = CapturedExchange.fromHttpRequest(req);
    const holder = req.context.get(CAPTURED_EXCHANGE);
    holder.exchange = exchange;

    return next(req).pipe(
        tap({
            next: (event) => {
                if (event instanceof HttpResponse) {
                    exchange.captureResponse(event);
                }
            },
            error: (err) => {
                if (err instanceof HttpErrorResponse) {
                    exchange.captureError(err);
                }
            },
        }),
    );
};
