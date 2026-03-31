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
}
