import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, catchError, map, throwError } from 'rxjs';
import {
  BuildPlanUrlRequest,
  BuildPlanUrlResponse,
  DecodePlanUrlRequest,
  DecodePlanUrlResponse,
  EvaluatePlanRequest,
  EvaluatePlanResponse,
  LoadSourceRequest,
  PlanBoardApiError,
} from '../common/model/api/plan-board';
import { SourceProfile } from '../common/model/core/evaluation';
import { ResponseBody } from '../common/response/response';

@Injectable({ providedIn: 'root' })
/** HTTP boundary for PlanBoard. It never logs Plan or subscription content. */
export class PlanBoardApiService {
  constructor(private readonly http: HttpClient) {}

  loadSource(request: LoadSourceRequest): Observable<SourceProfile> {
    return this.post('/api/load-source', request.serialize(), SourceProfile.deserialize);
  }

  evaluatePlan(request: EvaluatePlanRequest): Observable<EvaluatePlanResponse> {
    return this.post('/api/evaluate-plan', request.serialize(), EvaluatePlanResponse.deserialize);
  }

  buildUrl(request: BuildPlanUrlRequest): Observable<BuildPlanUrlResponse> {
    return this.post('/api/build-url', request.serialize(), BuildPlanUrlResponse.deserialize);
  }

  decodePlan(request: DecodePlanUrlRequest): Observable<DecodePlanUrlResponse> {
    return this.post('/api/decode-plan', request.serialize(), DecodePlanUrlResponse.deserialize);
  }

  private post<T>(
    path: string,
    body: unknown,
    deserialize: (value: unknown, path?: string) => T,
  ): Observable<T> {
    return this.http.post(path, body).pipe(
      map((value) => {
        const response = ResponseBody.deserialize(value, deserialize);
        if (!response.isOk() || response.data === null) {
          throw new PlanBoardApiError(response.messages, response.status.code);
        }
        return response.data;
      }),
      catchError((error) => {
        if (error instanceof PlanBoardApiError) return throwError(() => error);
        if (error instanceof HttpErrorResponse) {
          try {
            const response = ResponseBody.deserialize<unknown>(error.error);
            return throwError(() => new PlanBoardApiError(response.messages, error.status));
          } catch {
            return throwError(() => new PlanBoardApiError([error.message], error.status));
          }
        }
        return throwError(() => error);
      }),
    );
  }
}
