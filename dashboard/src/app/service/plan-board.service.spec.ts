import { TestBed } from '@angular/core/testing';
import { Observable, Subject } from 'rxjs';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  BuildPlanUrlRequest,
  BuildPlanUrlResponse,
  DecodePlanUrlRequest,
  DecodePlanUrlResponse,
  EvaluatePlanRequest,
  EvaluatePlanResponse,
  LoadSourceRequest,
} from '../common/model/api/plan-board';
import {
  EvaluationReport,
  ResolvedDependencies,
  SourceProfile,
} from '../common/model/core/evaluation';
import { Plan, RemoteSourceInput } from '../common/model/core/plan';
import { Profile } from '../common/model/core/profile';
import { ProxyClient } from '../common/model/core/proxy-client';
import { PlanBoardApiService } from './plan-board-api.service';
import { PlanBoardService } from './plan-board.service';

class ControlledApi {
  readonly loadCalls: Array<{ request: LoadSourceRequest; response: Subject<SourceProfile> }> = [];
  readonly evaluateCalls: Array<{
    request: EvaluatePlanRequest;
    response: Subject<EvaluatePlanResponse>;
  }> = [];
  readonly buildCalls: Array<{
    request: BuildPlanUrlRequest;
    response: Subject<BuildPlanUrlResponse>;
  }> = [];
  readonly decodeCalls: Array<{
    request: DecodePlanUrlRequest;
    response: Subject<DecodePlanUrlResponse>;
  }> = [];
  loadSource(request: LoadSourceRequest): Observable<SourceProfile> {
    const response = new Subject<SourceProfile>();
    this.loadCalls.push({ request, response });
    return response;
  }
  evaluatePlan(request: EvaluatePlanRequest): Observable<EvaluatePlanResponse> {
    const response = new Subject<EvaluatePlanResponse>();
    this.evaluateCalls.push({ request, response });
    return response;
  }
  buildUrl(request: BuildPlanUrlRequest): Observable<BuildPlanUrlResponse> {
    const response = new Subject<BuildPlanUrlResponse>();
    this.buildCalls.push({ request, response });
    return response;
  }
  decodePlan(request: DecodePlanUrlRequest): Observable<DecodePlanUrlResponse> {
    const response = new Subject<DecodePlanUrlResponse>();
    this.decodeCalls.push({ request, response });
    return response;
  }
}

describe('PlanBoardService request lifecycle', () => {
  let service: PlanBoardService;
  let api: ControlledApi;
  beforeEach(() => {
    vi.useFakeTimers();
    localStorage.clear();
    api = new ControlledApi();
    TestBed.configureTestingModule({
      providers: [PlanBoardService, { provide: PlanBoardApiService, useValue: api }],
    });
    service = TestBed.inject(PlanBoardService);
  });

  it('fences a source response after its input changes or the source is removed', () => {
    const source = addReadySource('https://example.com/a');
    service.loadSource(source.id);
    const oldRequest = api.loadCalls.at(-1)!.response;
    service.updateSource(
      source.id,
      (item) => (item.input = new RemoteSourceInput('https://example.com/b')),
    );
    oldRequest.next(profile(source.id, 'old'));
    expect(service.sourceProfiles().has(source.id)).toBe(false);
    expect(service.loadingSourceIds().has(source.id)).toBe(false);
    service.loadSource(source.id);
    const replacementRequest = api.loadCalls.at(-1)!.response;
    service.removeSource(source.id);
    replacementRequest.next(profile(source.id, 'removed'));
    expect(service.sourceProfiles().has(source.id)).toBe(false);
  });

  it('fences source responses when the client changes', () => {
    const source = addReadySource('https://example.com/a');
    service.loadSource(source.id);
    const request = api.loadCalls.at(-1)!.response;
    service.setClient(nextClient(service.plan().client));
    request.next(profile(source.id, 'wrong-client'));
    expect(service.sourceProfiles().size).toBe(0);
    expect(service.loadingSourceIds().size).toBe(0);
  });

  it('reloads sources independently and retains successful results', () => {
    const first = addReadySource('https://example.com/a');
    const second = addReadySource('https://example.com/b');
    service.reloadAllSources(true);
    const [firstCall, secondCall] = api.loadCalls.slice(-2);
    firstCall.response.next(profile(first.id, 'fresh'));
    firstCall.response.complete();
    secondCall.response.error(new Error('source unavailable'));
    expect(service.sourceProfiles().get(first.id)?.fingerprint).toBe('fresh');
    expect(service.sourceProfiles().has(second.id)).toBe(false);
    expect(service.sourceErrors().get(second.id)).toBe('source unavailable');
    expect(service.loadingSourceIds().size).toBe(0);
  });

  it('keeps the previous preview visible but stale while refresh and evaluation run', () => {
    const source = addReadySource('https://example.com/a');
    completeLoadAndEvaluation(source.id, 'initial');
    expect(service.previewStale()).toBe(false);
    expect(service.renderedContent()).toBe('rendered-initial');
    service.loadSource(source.id, true);
    expect(service.previewStale()).toBe(true);
    expect(service.renderedContent()).toBe('rendered-initial');
    expect(service.canBuildUrl()).toBe(false);
    api.loadCalls.at(-1)!.response.next(profile(source.id, 'refreshed'));
    vi.advanceTimersByTime(300);
    expect(service.evaluating()).toBe(true);
    expect(service.canBuildUrl()).toBe(false);
    api.evaluateCalls.at(-1)!.response.next(evaluation('refreshed'));
    expect(service.previewStale()).toBe(false);
    expect(service.renderedContent()).toBe('rendered-refreshed');
  });

  it('debounces evaluation and ignores an evaluation completed after a later edit', () => {
    const source = addReadySource('https://example.com/a');
    service.loadSource(source.id);
    api.loadCalls.at(-1)!.response.next(profile(source.id, 'loaded'));
    vi.advanceTimersByTime(300);
    const evaluationRequest = api.evaluateCalls.at(-1)!.response;
    service.updateSource(source.id, (item) => (item.name = 'renamed'));
    evaluationRequest.next(evaluation('obsolete'));
    expect(service.renderedContent()).toBeNull();
    vi.advanceTimersByTime(299);
    expect(api.evaluateCalls).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(api.evaluateCalls).toHaveLength(2);
  });

  it('fences build and decode responses after later plan edits', () => {
    const source = addReadySource('https://example.com/a');
    completeLoadAndEvaluation(source.id, 'ready');
    service.buildUrl();
    const build = api.buildCalls.at(-1)!.response;
    service.updateSource(source.id, (item) => (item.name = 'after build'));
    build.next(new BuildPlanUrlResponse('https://example.com/stale', 'fp', 1, []));
    expect(service.subscriptionUrl()).toBeNull();
    expect(service.building()).toBe(false);
    service.restoreFromUrl('https://example.com/encoded');
    const decode = api.decodeCalls.at(-1)!.response;
    service.updateSource(source.id, (item) => (item.name = 'after decode'));
    decode.next(new DecodePlanUrlResponse(Plan.empty(), 'decoded', 1));
    expect(service.plan().sources).toHaveLength(1);
    expect(service.plan().sources[0].name).toBe('after decode');
    expect(service.restoring()).toBe(false);
  });

  it('does not evaluate retained snapshots after a refresh fails', () => {
    const source = addReadySource('https://example.com/a');
    completeLoadAndEvaluation(source.id, 'initial');
    service.loadSource(source.id, true);
    api.loadCalls.at(-1)!.response.error(new Error('refresh failed'));
    service.updateSource(source.id, (item) => (item.name = 'renamed'));
    vi.advanceTimersByTime(300);
    service.evaluateNow();
    expect(api.evaluateCalls).toHaveLength(1);
    expect(service.previewStale()).toBe(true);
    expect(service.canBuildUrl()).toBe(false);
    expect(service.sourceProfiles().get(source.id)?.fingerprint).toBe('initial');
  });

  it('invalidates an in-flight export when refreshing snapshots without a Plan edit', () => {
    const source = addReadySource('https://example.com/a');
    completeLoadAndEvaluation(source.id, 'initial');
    service.buildUrl();
    const response = api.buildCalls.at(-1)!.response;
    service.loadSource(source.id, true);
    response.next(new BuildPlanUrlResponse('https://example.com/stale', 'fp', 1, []));
    expect(service.subscriptionUrl()).toBeNull();
    expect(service.building()).toBe(false);
  });

  it('unsubscribes outstanding work on destroy', () => {
    const source = addReadySource('https://example.com/a');
    service.loadSource(source.id);
    const response = api.loadCalls.at(-1)!.response;
    expect(response.observed).toBe(true);
    service.ngOnDestroy();
    expect(response.observed).toBe(false);
    response.next(profile(source.id, 'late'));
    expect(service.sourceProfiles().size).toBe(0);
  });

  function addReadySource(url: string) {
    const source = service.addSource('remote');
    service.updateSource(source.id, (item) => (item.input = new RemoteSourceInput(url)));
    return source;
  }
  function completeLoadAndEvaluation(sourceId: number, fingerprint: string): void {
    service.loadSource(sourceId);
    api.loadCalls.at(-1)!.response.next(profile(sourceId, fingerprint));
    vi.advanceTimersByTime(300);
    api.evaluateCalls.at(-1)!.response.next(evaluation(fingerprint));
  }
});

function profile(sourceId: number, fingerprint: string): SourceProfile {
  return new SourceProfile(
    sourceId,
    ProxyClient.Clash,
    'input',
    fingerprint,
    1,
    '[Proxy]\n',
    Profile.empty(),
    new ResolvedDependencies([], []),
    [],
  );
}
function evaluation(fingerprint: string): EvaluatePlanResponse {
  return new EvaluatePlanResponse(
    fingerprint,
    new Map(),
    new EvaluationReport([], [], [], [], [], Profile.empty()),
    `rendered-${fingerprint}`,
  );
}
function nextClient(client: ProxyClient): ProxyClient {
  return client === ProxyClient.Clash ? ProxyClient.Surge : ProxyClient.Clash;
}
