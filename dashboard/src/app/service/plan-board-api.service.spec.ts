import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it, beforeEach } from 'vitest';
import { LoadSourceRequest, PlanBoardApiError } from '../common/model/api/plan-board';
import { SourceProfile } from '../common/model/core/evaluation';
import { RemoteSourceInput } from '../common/model/core/plan';
import { ProxyClient } from '../common/model/core/proxy-client';
import { PlanBoardApiService } from './plan-board-api.service';

const SOURCE_PROFILE = {
  source_id: 1,
  client: 'surge',
  input_fingerprint: 'input',
  fingerprint: 'profile',
  loaded_at: 1,
  content: '[Proxy]\n',
  profile: { proxies: [], proxy_providers: [], proxy_groups: [], rule_providers: [], rules: [] },
  dependencies: { nodes: [], rules: [] },
  diagnostics: [],
};

describe('PlanBoardApiService', () => {
  let service: PlanBoardApiService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(PlanBoardApiService);
    http = TestBed.inject(HttpTestingController);
  });

  it('maps successful responses to real classes', () => {
    let result: SourceProfile | undefined;
    service
      .loadSource(
        new LoadSourceRequest(
          1,
          ProxyClient.Surge,
          new RemoteSourceInput('https://example.com/sub'),
          'use',
        ),
      )
      .subscribe((value) => (result = value));
    const request = http.expectOne('/api/load-source');
    expect(request.request.body).toEqual({
      source_id: 1,
      client: 'surge',
      input: { kind: 'remote', url: 'https://example.com/sub' },
      cache: 'use',
    });
    request.flush({ status: { code: 0, status: 'OK' }, messages: [], data: SOURCE_PROFILE });
    expect(result).toBeInstanceOf(SourceProfile);
  });

  it('turns business and HTTP errors into PlanBoardApiError', () => {
    let business: unknown;
    service
      .loadSource(new LoadSourceRequest(1, ProxyClient.Surge, new RemoteSourceInput('bad'), 'use'))
      .subscribe({ error: (error) => (business = error) });
    http
      .expectOne('/api/load-source')
      .flush({
        status: { code: 100, status: 'SOURCE_LOAD_ERROR' },
        messages: ['invalid source'],
        data: null,
      });
    expect(business).toBeInstanceOf(PlanBoardApiError);
    expect((business as Error).message).toContain('invalid source');

    let transport: unknown;
    service
      .loadSource(new LoadSourceRequest(1, ProxyClient.Surge, new RemoteSourceInput('bad'), 'use'))
      .subscribe({ error: (error) => (transport = error) });
    http
      .expectOne('/api/load-source')
      .flush('upstream down', { status: 502, statusText: 'Bad Gateway' });
    expect(transport).toBeInstanceOf(PlanBoardApiError);
  });
});
