import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable, map, catchError, of } from "rxjs";
import { BackendStatus } from "../common/model/api/backend-status";
import {
    ResponseBody,
    ResponseBodyScheme,
} from "../common/response/response";

@Injectable({ providedIn: "root" })
export class StatusService {
    private static readonly STATUS_ENDPOINT = `/actuator/status`;
    private static readonly HEALTH_ENDPOINT = `/actuator/healthy`;

    constructor(private http: HttpClient) {
    }

    getStatus(): Observable<BackendStatus | null> {
        return this.http.get(StatusService.STATUS_ENDPOINT).pipe(
            map(response => {
                const body = ResponseBody.deserialize(ResponseBodyScheme.parse(response), BackendStatus);
                return body?.data ?? null;
            }),
            catchError(() => of(null)),
        );
    }

    healthCheck(): Observable<boolean> {
        return this.http.get(StatusService.HEALTH_ENDPOINT).pipe(
            map(response => {
                const body = ResponseBody.deserialize(ResponseBodyScheme.parse(response), undefined);
                return body?.isOk() ?? false;
            }),
            catchError(() => of(false)),
        );
    }
}
