import { describe, expect, it } from 'vitest';
import { EvaluationReport, SourceProfile } from './evaluation';
import * as EvaluationModel from './evaluation';
import { ItemEntry, Profile, Proxy } from './profile';
import * as ProfileModel from './profile';

const PROXY = {
  name: '香港 01',
  protocol: 'trojan',
  server: 'hk.example.com',
  port: 443,
  password: 'secret',
  cipher: null,
  sni: null,
  udp: true,
  tfo: null,
  skip_cert_verify: false,
  tags: ['premium'],
  extra: { custom: 'kept' },
  comment: null,
};
const PROFILE = {
  proxies: [{ kind: 'item', value: PROXY }],
  proxy_providers: [],
  proxy_groups: [],
  rule_providers: [],
  rules: [],
};

describe('Profile and evaluation serde', () => {
  it('preserves Profile extras and creates item instances', () => {
    const profile = Profile.deserialize(PROFILE);
    expect(profile).toBeInstanceOf(Profile);
    expect(profile.proxies[0]).toBeInstanceOf(ItemEntry);
    expect((profile.proxies[0] as ItemEntry<Proxy>).value).toBeInstanceOf(Proxy);
    expect((profile.proxies[0] as ItemEntry<Proxy>).value.extra).toEqual({ custom: 'kept' });
    expect(profile.serialize()).toEqual(PROFILE);
  });

  it('maps SourceProfile and EvaluationReport recursively', () => {
    const source = SourceProfile.deserialize({
      source_id: 7,
      client: 'clash',
      input_fingerprint: 'input',
      fingerprint: 'all',
      loaded_at: 42,
      content: 'proxies: []',
      profile: PROFILE,
      dependencies: { nodes: [{ source: 7, key: 'provider', nodes: [PROXY] }], rules: [] },
      diagnostics: [{ severity: 'warning', code: 'example', path: 'source/7', message: 'example' }],
      input_nodes: [
        {
          identity: 's7/n0',
          source: 7,
          proxy: PROXY,
          origins: [{ kind: 'direct', index: 0 }],
          region: '🇭🇰 香港',
        },
      ],
      input_diagnostics: [],
    });
    expect(source).toBeInstanceOf(SourceProfile);
    expect(source.profile).toBeInstanceOf(Profile);
    expect(source.dependencies.nodes[0].nodes[0]).toBeInstanceOf(Proxy);
    expect(source.input_nodes[0]).toBeInstanceOf(EvaluationModel.InputNode);
    expect(source.serialize()).not.toHaveProperty('input_nodes');
    expect(source.serialize()).not.toHaveProperty('input_diagnostics');

    const compatible = SourceProfile.deserialize({
      source_id: 7,
      client: 'clash',
      input_fingerprint: 'input',
      fingerprint: 'all',
      loaded_at: 42,
      content: 'proxies: []',
      profile: PROFILE,
      dependencies: { nodes: [], rules: [] },
      diagnostics: [],
    });
    expect(compatible.input_nodes).toEqual([]);
    expect(compatible.input_diagnostics).toEqual([]);

    const report = EvaluationReport.deserialize({
      nodes: [
        {
          identity: 'source:7/node:0',
          source: 7,
          origins: [{ kind: 'direct', index: 0 }],
          proxy: PROXY,
          original_tags: ['premium'],
          effective_tags: ['premium'],
          kept: true,
          reachable: true,
          output_name: '香港 01',
        },
      ],
      groups: [],
      base_groups: [
        {
          identity: 'policy:1/region:hk',
          output_name: '香港',
          policy: 1,
          name: '香港',
          depth: 1,
          parent: null,
          dimensions: [{ dimension: { kind: 'region' }, value: '香港' }],
        },
      ],
      diagnostics: [],
      trace: [{ path: 'group/1', resources: ['source:7/node:0'] }],
      profile: PROFILE,
    });
    expect(report).toBeInstanceOf(EvaluationReport);
    expect(report.nodes[0].proxy).toBeInstanceOf(Proxy);
    expect(report.profile).toBeInstanceOf(Profile);
    expect(report.hasErrors).toBe(false);
  });

  it('dispatches Profile and Evaluation tagged enums to real variants', () => {
    expect(ProfileModel.ExternalResource.deserialize({ kind: 'http', value: 'https://x' })).toBeInstanceOf(
      ProfileModel.HttpResource,
    );
    expect(ProfileModel.ExternalResource.deserialize({ kind: 'file', value: './x' })).toBeInstanceOf(
      ProfileModel.FileResource,
    );
    expect(ProfileModel.PolicyRef.deserialize({ kind: 'named', value: 'group' })).toBeInstanceOf(
      ProfileModel.NamedPolicyRef,
    );
    expect(ProfileModel.PolicyRef.deserialize({ kind: 'built_in', value: 'DIRECT' })).toBeInstanceOf(
      ProfileModel.BuiltInPolicyRef,
    );
    expect(ProfileModel.ProviderSource.deserialize({ kind: 'inline' })).toBeInstanceOf(
      ProfileModel.InlineProviderSource,
    );
    expect(
      ProfileModel.ProviderSource.deserialize({
        kind: 'external',
        value: { kind: 'http', value: 'https://x' },
      }),
    ).toBeInstanceOf(ProfileModel.ExternalProviderSource);

    const payloads = [
      { kind: 'classical', value: [] },
      { kind: 'domain', value: ['example.com'] },
      { kind: 'ip_cidr', value: ['192.0.2.0/24'] },
    ].map((value) => ProfileModel.RuleProviderPayload.deserialize(value));
    expect(new Set(payloads.map((value) => value.constructor.name)).size).toBe(3);

    const origins = [
      { kind: 'direct', index: 0 },
      { kind: 'proxy_provider', name: 'p', index: 0 },
      { kind: 'policy_path', group_index: 0, index: 0 },
    ].map((value) => EvaluationModel.NodeOrigin.deserialize(value));
    expect(new Set(origins.map((value) => value.constructor.name)).size).toBe(3);

    const groupKinds = [
      { kind: 'base', policy: 1, depth: 1 },
      { kind: 'custom', id: 1 },
      { kind: 'imported', source: 1, index: 0 },
    ].map((value) => EvaluationModel.EvaluatedGroupKind.deserialize(value));
    expect(new Set(groupKinds.map((value) => value.constructor.name)).size).toBe(3);

    const members = ['node', 'group', 'builtin'].map((kind) =>
      EvaluationModel.EvaluatedMemberRef.deserialize({ kind, value: 'x' }),
    );
    expect(new Set(members.map((value) => value.constructor.name)).size).toBe(3);
  });
});
