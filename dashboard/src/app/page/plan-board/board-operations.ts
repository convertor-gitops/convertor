import { EvaluationReport, SourceProfile } from '../../common/model/core/evaluation';
import {
  AtomPredicate,
  CustomGroup,
  EqualsStringMatch,
  NodeNamePredicate,
  NodeSelection,
  NodesMemberSelector,
  Plan,
} from '../../common/model/core/plan';
import { Proxy } from '../../common/model/core/profile';

/** Snapshot-scoped selection key; evaluator identity alone may be reused after refresh. */
export interface InputNodeRow {
  key: string;
  identity: string;
  source: number;
  sourceName: string;
  proxy: Proxy;
  region: string;
}

export interface BrowseDimension {
  kind: 'source' | 'protocol' | 'region' | 'has_tag';
  tag?: string;
}

export interface BrowseBranch {
  key: string;
  label: string;
  count: number;
  children: BrowseBranch[];
  nodes: InputNodeRow[];
}

export function inputNodes(
  plan: Plan,
  profiles: ReadonlyMap<number, SourceProfile>,
): InputNodeRow[] {
  return plan.sources.flatMap((source) => {
    const snapshot = profiles.get(source.id);
    return (
      snapshot?.input_nodes.map((node) => ({
        key: JSON.stringify([source.id, snapshot.fingerprint, node.identity]),
        identity: node.identity,
        source: source.id,
        sourceName: source.name,
        proxy: node.proxy,
        region: node.region,
      })) ?? []
    );
  });
}

/** Partition inside each parent bucket; dimensions are ordered, never a composite key. */
export function groupInputNodes(
  nodes: InputNodeRow[],
  dimensions: BrowseDimension[],
  path = '',
): BrowseBranch[] {
  if (!dimensions.length)
    return [{ key: path || 'all', label: '全部节点', count: nodes.length, children: [], nodes }];
  const [dimension, ...rest] = dimensions;
  const buckets = new Map<string, { label: string; nodes: InputNodeRow[] }>();
  for (const node of nodes) {
    let value: string;
    let label: string;
    switch (dimension.kind) {
      case 'source':
        value = String(node.source);
        label = node.sourceName;
        break;
      case 'protocol':
        value = label = node.proxy.protocol;
        break;
      case 'region':
        value = label = node.region || '未识别地区';
        break;
      case 'has_tag':
        value = String(node.proxy.tags.includes(dimension.tag ?? ''));
        label = value === 'true' ? `有标签「${dimension.tag}」` : `无标签「${dimension.tag}」`;
        break;
    }
    if (!buckets.has(value)) buckets.set(value, { label, nodes: [] });
    buckets.get(value)!.nodes.push(node);
  }
  return [...buckets].map(([value, bucket]) => {
    const key = `${path}/${JSON.stringify([dimension.kind, dimension.tag ?? '', value])}`;
    return {
      key,
      label: bucket.label,
      count: bucket.nodes.length,
      children: rest.length ? groupInputNodes(bucket.nodes, rest, key) : [],
      nodes: rest.length ? [] : bucket.nodes,
    };
  });
}

export function acceptsPickedNodes(group: CustomGroup): boolean {
  return group.member_selectors.every(
    (selector) =>
      selector instanceof NodesMemberSelector &&
      selector.selection.predicate instanceof AtomPredicate &&
      selector.selection.predicate.args instanceof NodeNamePredicate &&
      selector.selection.predicate.args.test instanceof EqualsStringMatch,
  );
}

/** Gate is checked at mutation time as well as by the drag target. Inputs are never removed. */
export function appendPickedNodes(plan: Plan, groupId: number, nodes: InputNodeRow[]): Plan {
  const next = plan.clone();
  const group = next.groups.find((item) => item.id === groupId);
  if (!group || !acceptsPickedNodes(group))
    throw new Error('该组包含条件或组引用，只能通过条件编辑器修改成员。');
  const keys = new Set(
    group.member_selectors.map((selector) => {
      const selection = (selector as NodesMemberSelector).selection;
      const atom = selection.predicate as AtomPredicate<NodeNamePredicate>;
      return JSON.stringify([selection.source, (atom.args.test as EqualsStringMatch).value]);
    }),
  );
  for (const node of nodes) {
    if (!next.sources.some((source) => source.id === node.source))
      throw new Error('节点来源已移除，请重新选择节点。');
    const key = JSON.stringify([node.source, node.proxy.name]);
    if (keys.has(key)) continue;
    keys.add(key);
    group.member_selectors.push(
      new NodesMemberSelector(
        new NodeSelection(
          node.source,
          new AtomPredicate(new NodeNamePredicate(new EqualsStringMatch(node.proxy.name))),
        ),
      ),
    );
  }
  return next;
}

/** Expand graph references for linkage, stopping cycles and deduplicating resource identities. */
export function referencedNodes(
  report: EvaluationReport,
  resources: Iterable<string>,
): Set<string> {
  const groups = new Map(report.groups.map((group) => [group.identity, group]));
  const nodes = new Set(report.nodes.map((node) => node.identity));
  const visited = new Set<string>();
  const result = new Set<string>();
  const visit = (identity: string): void => {
    if (visited.has(identity)) return;
    visited.add(identity);
    if (nodes.has(identity)) result.add(identity);
    else
      groups.get(identity)?.members.forEach((member) => {
        if (member.kind !== 'builtin') visit(member.value);
      });
  };
  for (const resource of resources) visit(resource);
  return result;
}
