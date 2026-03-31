import { HttpErrorResponse } from "@angular/common/http";
import {
    catchError,
    defer,
    map,
    OperatorFunction,
    throwError,
} from "rxjs";
import { ResponseBody } from "../response/response";

export class LatencyWrapper<T> {
    time: number;
    value: number;

    constructor(
        public sentAt: number,
        public latency: number,
        public response?: ResponseBody<T>,
        public error?: HttpErrorResponse,
    ) {
        this.time = this.sentAt;
        this.value = Math.ceil(this.latency);
    }
}

export function withLatency<T extends ResponseBody<R>, R>(
    onSample?: (response: LatencyWrapper<R>) => void,
): OperatorFunction<T, LatencyWrapper<R>> {
    return (source$) => defer(() => {
        const sentAt = Date.now();
        const start = typeof performance !== "undefined" ? performance.now() : sentAt;

        return source$.pipe(
            map((resp) => {
                const now = Date.now();
                const end = typeof performance !== "undefined" ? performance.now() : now;
                const latencyResp: LatencyWrapper<R> = new LatencyWrapper(sentAt, end - start, resp);
                onSample?.(latencyResp);
                return latencyResp;
            }),
            catchError((err: HttpErrorResponse) => {
                const now = Date.now();
                const end = typeof performance !== "undefined" ? performance.now() : now;
                const latencyResp: LatencyWrapper<R> = new LatencyWrapper(sentAt, end - start, undefined, err);
                onSample?.(latencyResp);
                return throwError(() => err);
            }),
        );
    });
}
