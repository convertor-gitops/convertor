import { HttpErrorResponse } from "@angular/common/http";
import { CapturedExchangeHolder } from "../http/captured-exchange";
import { RequestBody } from "../response/request";
import {
    ResponseBody,
    ResponseBodyScheme,
} from "../response/response";
import AppStatus from "../response/status";

export class DashboardErrorResponse {
    private constructor(
        // response body 中的 status 用来表示 http status，request 则是 client request，不存在 response
        public readonly clientRequest: ResponseBody<unknown> | null,
        // 这是服务端返回的完整 response body
        public readonly responseBody: ResponseBody<unknown> | null,
    ) {
    }

    static fromBiz(body: ResponseBody<unknown>, holder: CapturedExchangeHolder): DashboardErrorResponse {
        return new DashboardErrorResponse(
            new ResponseBody(
                new AppStatus(200, "OK"),
                [],
                DashboardErrorResponse.toClientRequest(holder),
                undefined,
            ),
            body,
        );
    }

    static fromHttp(httpError: HttpErrorResponse, holder: CapturedExchangeHolder): DashboardErrorResponse {
        return new DashboardErrorResponse(
            new ResponseBody(
                new AppStatus(200, "OK"),
                [],
                DashboardErrorResponse.toClientRequest(holder),
                undefined,
            ),
            DashboardErrorResponse.parseResponseBody(httpError),
        );
    }

    private static parseResponseBody(httpError: HttpErrorResponse): ResponseBody<unknown> | null {
        if (!httpError.error || typeof httpError.error !== "object") {
            return null;
        }

        try {
            return ResponseBody.deserialize(ResponseBodyScheme.parse(httpError.error), undefined);
        } catch {
            return null;
        }
    }

    private static toClientRequest(holder: CapturedExchangeHolder): RequestBody | null {
        if (!holder.exchange) {
            return null;
        }

        const req = holder.exchange.request;
        const parsedUrl = new URL(req.url, "http://placeholder");
        return new RequestBody(
            req.method,
            parsedUrl.protocol,
            parsedUrl.host === "placeholder" ? location.host : parsedUrl.host,
            parsedUrl.pathname + parsedUrl.search,
            req.headers,
        );
    }
}
