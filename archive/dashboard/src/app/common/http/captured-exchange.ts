import {
    HttpContextToken,
    HttpErrorResponse,
    HttpRequest,
    HttpResponse,
} from "@angular/common/http";

// ─── 捕获的请求详情 ───────────────────────────────────────────

export interface CapturedRequest {
    readonly method: string;
    readonly url: string;
    readonly headers: Map<string, string>;
}

// ─── 捕获的响应详情 ───────────────────────────────────────────

export interface CapturedResponse {
    readonly status: number;
    readonly statusText: string;
    readonly headers: Map<string, string>;
}

// ─── 完整的请求/响应交换记录 ──────────────────────────────────

export class CapturedExchange {
    readonly request: CapturedRequest;
    response: CapturedResponse | null = null;

    private constructor(request: CapturedRequest) {
        this.request = request;
    }

    static fromHttpRequest(req: HttpRequest<unknown>): CapturedExchange {
        const headers = new Map<string, string>();
        for (const key of req.headers.keys()) {
            headers.set(key, req.headers.get(key) ?? "");
        }
        return new CapturedExchange({
            method: req.method,
            url: req.urlWithParams,
            headers,
        });
    }

    captureResponse(res: HttpResponse<unknown>): void {
        this.response = CapturedExchange.extractResponseHeaders(res);
    }

    captureError(err: HttpErrorResponse): void {
        this.response = CapturedExchange.extractResponseHeaders(err);
    }

    private static extractResponseHeaders(
        res: HttpResponse<unknown> | HttpErrorResponse,
    ): CapturedResponse {
        const headers = new Map<string, string>();
        for (const key of res.headers.keys()) {
            headers.set(key, res.headers.get(key) ?? "");
        }
        return {
            status: res.status,
            statusText: res.statusText,
            headers,
        };
    }
}

// ─── HttpContext Token ────────────────────────────────────────
//
// 使用可变 holder 对象，拦截器写入、service 层读取。
// 用法：
//   const holder = { exchange: null };
//   const context = new HttpContext().set(CAPTURED_EXCHANGE, holder);
//   this.http.get(url, { context });
//   // 请求完成后：holder.exchange 已被拦截器填充

export interface CapturedExchangeHolder {
    exchange: CapturedExchange | null;
}

export const CAPTURED_EXCHANGE = new HttpContextToken<CapturedExchangeHolder>(
    () => ({ exchange: null }),
);
