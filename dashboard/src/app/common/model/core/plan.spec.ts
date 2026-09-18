import { describe, expect, it } from 'vitest';
import { DeserializeError } from '../../deserialize';
import * as PlanModel from './plan';
import {
  BaseGroupsMemberSelector,
  BuiltinTarget,
  Plan,
  RegionDimension,
  SelectGroupStrategy,
} from './plan';

const PLAN_FIXTURE = {
  version: 1,
  client: 'surge',
  sources: [
    {
      id: 1,
      name: 'source-a',
      input: { kind: 'inline', content: '[Proxy]\n' },
      annotations: [
        {
          when: { op: 'atom', args: { field: 'name', test: { op: 'contains', value: '家宽' } } },
          add_tags: ['家宽'],
        },
      ],
      node_filter: {
        op: 'not',
        args: { op: 'atom', args: { field: 'protocol', test: { op: 'equals', value: 'http' } } },
      },
    },
  ],
  grouping_policies: [
    { id: 1, group_by: [{ kind: 'region' }, { kind: 'source' }], strategy: { kind: 'select' } },
  ],
  groups: [
    {
      id: 1,
      name: 'all',
      strategy: {
        kind: 'url_test',
        url: 'https://example.com/204',
        interval_secs: 600,
        tolerance_ms: 50,
      },
      member_selectors: [
        {
          kind: 'base_groups',
          value: { policy: 1, scope: 'Roots', predicate: { op: 'all', args: [] } },
        },
      ],
      on_empty: { Use: { Builtin: 'Direct' } },
    },
  ],
  rules: [
    {
      Emit: {
        rules: [
          {
            rule: {
              rule_type: 'DOMAIN-SUFFIX',
              value: 'example.com',
              target: null,
              options: [],
              comment: null,
            },
            target: { Group: 1 },
          },
        ],
      },
    },
    {
      Take: {
        source: 1,
        predicate: { op: 'all', args: [] },
        targets: { Map: { cases: [], unmatched: { Use: { Builtin: 'Reject' } } } },
      },
    },
  ],
  output: { roots: [1], extra_nodes: [], fallback: { Group: 1 }, settings_source: 1 },
};

describe('Plan hand-written serde', () => {
  it('creates real nested instances and serializes to the Rust fixture exactly', () => {
    const plan = Plan.deserialize(PLAN_FIXTURE);
    expect(plan).toBeInstanceOf(Plan);
    expect(plan.grouping_policies[0].group_by[0]).toBeInstanceOf(RegionDimension);
    expect(plan.grouping_policies[0].strategy).toBeInstanceOf(SelectGroupStrategy);
    expect(plan.groups[0].member_selectors[0]).toBeInstanceOf(BaseGroupsMemberSelector);
    expect((plan.groups[0].on_empty.serialize() as { Use: unknown }).Use).toEqual({
      Builtin: 'Direct',
    });
    expect(plan.serialize()).toEqual(PLAN_FIXTURE);
  });

  it('reports a precise path for unknown variants', () => {
    const invalid = structuredClone(PLAN_FIXTURE);
    invalid.sources[0].input = { kind: 'database', content: '' };
    expect(() => Plan.deserialize(invalid)).toThrowError(DeserializeError);
    expect(() => Plan.deserialize(invalid)).toThrow('$.sources[0].input.kind');
  });

  it('retains concrete target variants after cloning', () => {
    const cloned = Plan.deserialize(PLAN_FIXTURE).clone();
    expect(cloned.output.fallback.serialize()).toEqual({ Group: 1 });
    const target = new BuiltinTarget('Reject');
    expect(target.serialize()).toEqual({ Builtin: 'Reject' });
  });

  it('dispatches every Plan tagged-enum variant to a concrete class', () => {
    const stringMatches = [
      { op: 'equals', value: 'x' },
      { op: 'one_of', value: ['x'] },
      { op: 'contains', value: 'x' },
      { op: 'starts_with', value: 'x' },
      { op: 'ends_with', value: 'x' },
      { op: 'regex', value: { pattern: 'x', case_insensitive: true } },
    ].map((value) => PlanModel.StringMatch.deserialize(value));
    expect(new Set(stringMatches.map((value) => value.constructor.name)).size).toBe(6);

    const nodePredicates = [
      { field: 'name', test: { op: 'equals', value: 'x' } },
      { field: 'protocol', test: { op: 'equals', value: 'x' } },
      { field: 'server', test: { op: 'equals', value: 'x' } },
      { field: 'port', test: 443 },
      { field: 'has_tag', test: 'home' },
    ].map((value) => PlanModel.NodePredicate.deserialize(value));
    expect(new Set(nodePredicates.map((value) => value.constructor.name)).size).toBe(5);

    const dimensions = [
      { kind: 'region' },
      { kind: 'source' },
      { kind: 'protocol' },
      { kind: 'has_tag', tag: 'home' },
    ].map((value) => PlanModel.NodeDimension.deserialize(value));
    expect(new Set(dimensions.map((value) => value.constructor.name)).size).toBe(4);

    const selectors = [
      { kind: 'nodes', value: { source: 1, predicate: { op: 'all', args: [] } } },
      {
        kind: 'nodes_from_groups',
        value: {
          selection: { source: 1, predicate: { op: 'all', args: [] } },
          depth: 'Recursive',
        },
      },
      {
        kind: 'import_groups',
        value: { source: 1, predicate: { op: 'all', args: [] } },
      },
      {
        kind: 'base_groups',
        value: { policy: 1, scope: 'All', predicate: { op: 'all', args: [] } },
      },
      { kind: 'group', value: 1 },
      { kind: 'builtin', value: 'Direct' },
    ].map((value) => PlanModel.MemberSelector.deserialize(value));
    expect(new Set(selectors.map((value) => value.constructor.name)).size).toBe(6);

    const bindings = [
      { Replace: { Builtin: 'Direct' } },
      'Preserve',
      { Map: { cases: [], unmatched: 'Drop' } },
    ].map((value) => PlanModel.TargetBinding.deserialize(value));
    expect(new Set(bindings.map((value) => value.constructor.name)).size).toBe(3);

    const blocks = [
      { Emit: { rules: [] } },
      {
        Take: {
          source: 1,
          predicate: { op: 'all', args: [] },
          targets: 'Preserve',
        },
      },
    ].map((value) => PlanModel.RuleBlock.deserialize(value));
    expect(new Set(blocks.map((value) => value.constructor.name)).size).toBe(2);
  });
});
