import { describe, expect, it } from 'vitest';
import {
  acceptsPickedNodes,
  appendPickedNodes,
  groupInputNodes,
  InputNodeRow,
} from './board-operations';
import {
  AllPredicate,
  AtomPredicate,
  BuiltinMemberSelector,
  CustomGroup,
  EqualsStringMatch,
  ErrorEmptyGroupPolicy,
  InlineSourceInput,
  NodeNamePredicate,
  NodeSelection,
  NodesMemberSelector,
  OneOfStringMatch,
  Plan,
  SelectGroupStrategy,
  Source,
} from '../../common/model/core/plan';
import { Proxy } from '../../common/model/core/profile';

function node(
  source: number,
  name: string,
  protocol = 'trojan',
  tags: string[] = [],
): InputNodeRow {
  return {
    key: `${source}/${name}`,
    identity: `s${source}/${name}`,
    source,
    sourceName: `源${source}`,
    region: '香港',
    proxy: Proxy.deserialize({
      name,
      protocol,
      server: 'example.com',
      port: 443,
      password: 'x',
      cipher: null,
      sni: null,
      udp: null,
      tfo: null,
      skip_cert_verify: null,
      tags,
      extra: {},
      comment: null,
    }),
  };
}
function plan(): Plan {
  const plan = Plan.empty();
  plan.sources = [1, 2].map(
    (id) => new Source(id, `源${id}`, new InlineSourceInput('content'), [], null),
  );
  plan.groups = [
    new CustomGroup(1, '手选组', new SelectGroupStrategy(), [], new ErrorEmptyGroupPolicy()),
  ];
  return plan;
}

describe('input browsing and explicit node group edits', () => {
  it('groups recursively within each parent with independent branch keys', () => {
    const nodes = [node(1, 'a'), node(1, 'b', 'ss'), node(2, 'a')];
    const tree = groupInputNodes(nodes, [{ kind: 'source' }, { kind: 'protocol' }]);
    expect(tree.map((branch) => [branch.label, branch.count])).toEqual([
      ['源1', 2],
      ['源2', 1],
    ]);
    expect(tree[0].nodes).toEqual([]);
    expect(tree[0].children.map((branch) => branch.label)).toEqual(['trojan', 'ss']);
    expect(tree[1].children[0].nodes).toEqual([nodes[2]]);
    expect(tree[0].children[0].key).not.toBe(tree[1].children[0].key);
    expect(
      groupInputNodes(nodes, [{ kind: 'protocol' }, { kind: 'source' }])[0].children,
    ).toHaveLength(2);
  });
  it('uses original proxy tags without dropping nonmatching nodes', () => {
    const tree = groupInputNodes(
      [node(1, 'a', 'ss', ['原始']), node(1, 'b')],
      [{ kind: 'has_tag', tag: '原始' }],
    );
    expect(tree.map((branch) => branch.label)).toEqual(['有标签「原始」', '无标签「原始」']);
    expect(tree.map((branch) => branch.count)).toEqual([1, 1]);
  });
  it('deduplicates by source and name, preserving inputs and original Plan', () => {
    const original = plan();
    const rows = [node(1, '同名'), node(1, '同名'), node(2, '同名')];
    const next = appendPickedNodes(original, 1, rows);
    expect(original.groups[0].member_selectors).toEqual([]);
    expect(rows).toHaveLength(3);
    expect(next.groups[0].member_selectors).toHaveLength(2);
    expect(acceptsPickedNodes(next.groups[0])).toBe(true);
    expect(appendPickedNodes(next, 1, rows).groups[0].member_selectors).toHaveLength(2);
    expect(
      next.groups[0].member_selectors.map(
        (selector) => (selector as NodesMemberSelector).selection.source,
      ),
    ).toEqual([1, 2]);
  });
  it('rejects OneOf, compound predicates and mixed builtin groups at mutation time', () => {
    const forbidden = [
      new NodesMemberSelector(
        new NodeSelection(1, new AtomPredicate(new NodeNamePredicate(new OneOfStringMatch(['a'])))),
      ),
      new NodesMemberSelector(
        new NodeSelection(
          1,
          new AllPredicate([new AtomPredicate(new NodeNamePredicate(new EqualsStringMatch('a')))]),
        ),
      ),
      new BuiltinMemberSelector('Direct'),
    ];
    for (const selector of forbidden) {
      const current = appendPickedNodes(plan(), 1, [node(1, 'a')]);
      current.groups[0].member_selectors.push(selector);
      expect(acceptsPickedNodes(current.groups[0])).toBe(false);
      expect(() => appendPickedNodes(current, 1, [node(1, 'b')])).toThrow();
    }
    expect(() => appendPickedNodes(plan(), 1, [node(9, 'removed')])).toThrow();
  });
});
