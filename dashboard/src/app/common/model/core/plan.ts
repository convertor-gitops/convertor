import {
  DeserializeError,
  JsonObject,
  asArray,
  asBoolean,
  asEnum,
  asInteger,
  asObject,
  asString,
  asStringArray,
  childPath,
  nullable,
  required,
} from '../../deserialize';
import { ProxyClient, ProxyClientSerde } from './proxy-client';
import { Rule } from './profile';

/**
 * Versioned orchestration domain model shared with `convertor::core::plan`.
 * Rust tagged enums are represented by abstract bases plus concrete variants,
 * so consumers never receive plain objects pretending to be domain instances.
 */

type Model = { serialize(): unknown };
type ModelDeserializer<T> = (value: unknown, path?: string) => T;

function field<T>(
  object: JsonObject,
  key: string,
  path: string,
  deserialize: (value: unknown, path: string) => T,
): T {
  return deserialize(required(object, key, path), childPath(path, key));
}

function nullableField<T>(
  object: JsonObject,
  key: string,
  path: string,
  deserialize: (value: unknown, path: string) => T,
): T | null {
  return nullable(required(object, key, path), deserialize, childPath(path, key));
}

function externallyTagged(value: unknown, path: string): [string, unknown] {
  if (typeof value === 'string') return [value, undefined];
  const object = asObject(value, path);
  const entries = Object.entries(object);
  if (entries.length !== 1)
    throw new DeserializeError(path, 'expected one externally tagged enum variant');
  return entries[0];
}

export abstract class SourceInput {
  abstract readonly kind: 'remote' | 'inline';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): SourceInput {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'remote':
        return new RemoteSourceInput(field(object, 'url', path, asString));
      case 'inline':
        return new InlineSourceInput(field(object, 'content', path, asString));
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown SourceInput variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class RemoteSourceInput extends SourceInput {
  readonly kind = 'remote' as const;
  constructor(public url: string) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, url: this.url };
  }
}
export class InlineSourceInput extends SourceInput {
  readonly kind = 'inline' as const;
  constructor(public content: string) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, content: this.content };
  }
}

export abstract class StringMatch {
  abstract readonly op: string;
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): StringMatch {
    const object = asObject(value, path);
    const op = field(object, 'op', path, asString);
    const raw = required(object, 'value', path);
    switch (op) {
      case 'equals':
        return new EqualsStringMatch(asString(raw, childPath(path, 'value')));
      case 'one_of':
        return new OneOfStringMatch(asStringArray(raw, childPath(path, 'value')));
      case 'contains':
        return new ContainsStringMatch(asString(raw, childPath(path, 'value')));
      case 'starts_with':
        return new StartsWithStringMatch(asString(raw, childPath(path, 'value')));
      case 'ends_with':
        return new EndsWithStringMatch(asString(raw, childPath(path, 'value')));
      case 'regex': {
        const regex = asObject(raw, childPath(path, 'value'));
        return new RegexStringMatch(
          field(regex, 'pattern', childPath(path, 'value'), asString),
          field(regex, 'case_insensitive', childPath(path, 'value'), asBoolean),
        );
      }
      default:
        throw new DeserializeError(
          childPath(path, 'op'),
          `unknown StringMatch variant ${JSON.stringify(op)}`,
        );
    }
  }
}
export class EqualsStringMatch extends StringMatch {
  readonly op = 'equals';
  constructor(public value: string) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, value: this.value };
  }
}
export class OneOfStringMatch extends StringMatch {
  readonly op = 'one_of';
  constructor(public value: string[]) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, value: [...this.value] };
  }
}
export class ContainsStringMatch extends StringMatch {
  readonly op = 'contains';
  constructor(public value: string) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, value: this.value };
  }
}
export class StartsWithStringMatch extends StringMatch {
  readonly op = 'starts_with';
  constructor(public value: string) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, value: this.value };
  }
}
export class EndsWithStringMatch extends StringMatch {
  readonly op = 'ends_with';
  constructor(public value: string) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, value: this.value };
  }
}
export class RegexStringMatch extends StringMatch {
  readonly op = 'regex';
  constructor(
    public pattern: string,
    public case_insensitive: boolean,
  ) {
    super();
  }
  serialize(): unknown {
    return {
      op: this.op,
      value: { pattern: this.pattern, case_insensitive: this.case_insensitive },
    };
  }
}

export abstract class NodePredicate implements Model {
  abstract readonly field: 'name' | 'protocol' | 'server' | 'port' | 'has_tag';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): NodePredicate {
    const object = asObject(value, path);
    const kind = field(object, 'field', path, asString);
    const test = required(object, 'test', path);
    switch (kind) {
      case 'name':
        return new NodeNamePredicate(StringMatch.deserialize(test, childPath(path, 'test')));
      case 'protocol':
        return new NodeProtocolPredicate(StringMatch.deserialize(test, childPath(path, 'test')));
      case 'server':
        return new NodeServerPredicate(StringMatch.deserialize(test, childPath(path, 'test')));
      case 'port':
        return new NodePortPredicate(asInteger(test, childPath(path, 'test')));
      case 'has_tag':
        return new NodeHasTagPredicate(asString(test, childPath(path, 'test')));
      default:
        throw new DeserializeError(
          childPath(path, 'field'),
          `unknown NodePredicate variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
abstract class StringNodePredicate extends NodePredicate {
  constructor(public test: StringMatch) {
    super();
  }
  serialize(): unknown {
    return { field: this.field, test: this.test.serialize() };
  }
}
export class NodeNamePredicate extends StringNodePredicate {
  readonly field = 'name' as const;
}
export class NodeProtocolPredicate extends StringNodePredicate {
  readonly field = 'protocol' as const;
}
export class NodeServerPredicate extends StringNodePredicate {
  readonly field = 'server' as const;
}
export class NodePortPredicate extends NodePredicate {
  readonly field = 'port' as const;
  constructor(public test: number) {
    super();
  }
  serialize(): unknown {
    return { field: this.field, test: this.test };
  }
}
export class NodeHasTagPredicate extends NodePredicate {
  readonly field = 'has_tag' as const;
  constructor(public test: string) {
    super();
  }
  serialize(): unknown {
    return { field: this.field, test: this.test };
  }
}

export abstract class GroupPredicate implements Model {
  abstract readonly field: 'name' | 'kind';
  constructor(public test: StringMatch) {}
  serialize(): unknown {
    return { field: this.field, test: this.test.serialize() };
  }
  static deserialize(value: unknown, path = '$'): GroupPredicate {
    const object = asObject(value, path);
    const kind = field(object, 'field', path, asString);
    const test = StringMatch.deserialize(required(object, 'test', path), childPath(path, 'test'));
    switch (kind) {
      case 'name':
        return new GroupNamePredicate(test);
      case 'kind':
        return new GroupKindPredicate(test);
      default:
        throw new DeserializeError(
          childPath(path, 'field'),
          `unknown GroupPredicate variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class GroupNamePredicate extends GroupPredicate {
  readonly field = 'name' as const;
}
export class GroupKindPredicate extends GroupPredicate {
  readonly field = 'kind' as const;
}

export abstract class RulePredicate implements Model {
  abstract readonly field: 'kind' | 'value' | 'original_target_name';
  constructor(public test: StringMatch) {}
  serialize(): unknown {
    return { field: this.field, test: this.test.serialize() };
  }
  static deserialize(value: unknown, path = '$'): RulePredicate {
    const object = asObject(value, path);
    const kind = field(object, 'field', path, asString);
    const test = StringMatch.deserialize(required(object, 'test', path), childPath(path, 'test'));
    switch (kind) {
      case 'kind':
        return new RuleKindPredicate(test);
      case 'value':
        return new RuleValuePredicate(test);
      case 'original_target_name':
        return new OriginalTargetNamePredicate(test);
      default:
        throw new DeserializeError(
          childPath(path, 'field'),
          `unknown RulePredicate variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class RuleKindPredicate extends RulePredicate {
  readonly field = 'kind' as const;
}
export class RuleValuePredicate extends RulePredicate {
  readonly field = 'value' as const;
}
export class OriginalTargetNamePredicate extends RulePredicate {
  readonly field = 'original_target_name' as const;
}

export abstract class Predicate<P extends Model> implements Model {
  abstract readonly op: 'all' | 'any' | 'not' | 'atom';
  abstract serialize(): unknown;
  static deserialize<P extends Model>(
    value: unknown,
    atom: ModelDeserializer<P>,
    path = '$',
  ): Predicate<P> {
    const object = asObject(value, path);
    const op = field(object, 'op', path, asString);
    const args = required(object, 'args', path);
    switch (op) {
      case 'all':
        return new AllPredicate(
          asArray(
            args,
            (item, itemPath) => Predicate.deserialize(item, atom, itemPath),
            childPath(path, 'args'),
          ),
        );
      case 'any':
        return new AnyPredicate(
          asArray(
            args,
            (item, itemPath) => Predicate.deserialize(item, atom, itemPath),
            childPath(path, 'args'),
          ),
        );
      case 'not':
        return new NotPredicate(Predicate.deserialize(args, atom, childPath(path, 'args')));
      case 'atom':
        return new AtomPredicate(atom(args, childPath(path, 'args')));
      default:
        throw new DeserializeError(
          childPath(path, 'op'),
          `unknown Predicate variant ${JSON.stringify(op)}`,
        );
    }
  }
}
export class AllPredicate<P extends Model> extends Predicate<P> {
  readonly op = 'all' as const;
  constructor(public args: Predicate<P>[]) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, args: this.args.map((item) => item.serialize()) };
  }
}
export class AnyPredicate<P extends Model> extends Predicate<P> {
  readonly op = 'any' as const;
  constructor(public args: Predicate<P>[]) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, args: this.args.map((item) => item.serialize()) };
  }
}
export class NotPredicate<P extends Model> extends Predicate<P> {
  readonly op = 'not' as const;
  constructor(public args: Predicate<P>) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, args: this.args.serialize() };
  }
}
export class AtomPredicate<P extends Model> extends Predicate<P> {
  readonly op = 'atom' as const;
  constructor(public args: P) {
    super();
  }
  serialize(): unknown {
    return { op: this.op, args: this.args.serialize() };
  }
}

export class NodeAnnotation {
  constructor(
    public when: Predicate<NodePredicate>,
    public add_tags: string[],
  ) {}
  static deserialize(value: unknown, path = '$'): NodeAnnotation {
    const object = asObject(value, path);
    return new NodeAnnotation(
      field(object, 'when', path, (item, itemPath) =>
        Predicate.deserialize(item, NodePredicate.deserialize, itemPath),
      ),
      field(object, 'add_tags', path, asStringArray),
    );
  }
  serialize(): unknown {
    return { when: this.when.serialize(), add_tags: [...this.add_tags] };
  }
}

export class Source {
  constructor(
    public id: number,
    public name: string,
    public input: SourceInput,
    public annotations: NodeAnnotation[],
    public node_filter: Predicate<NodePredicate> | null,
  ) {}
  static deserialize(value: unknown, path = '$'): Source {
    const object = asObject(value, path);
    return new Source(
      field(object, 'id', path, asInteger),
      field(object, 'name', path, asString),
      field(object, 'input', path, SourceInput.deserialize),
      field(object, 'annotations', path, (item, itemPath) =>
        asArray(item, NodeAnnotation.deserialize, itemPath),
      ),
      nullableField(object, 'node_filter', path, (item, itemPath) =>
        Predicate.deserialize(item, NodePredicate.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      id: this.id,
      name: this.name,
      input: this.input.serialize(),
      annotations: this.annotations.map((item) => item.serialize()),
      node_filter: this.node_filter?.serialize() ?? null,
    };
  }
}

export abstract class NodeDimension implements Model {
  abstract readonly kind: 'region' | 'source' | 'protocol' | 'has_tag';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): NodeDimension {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'region':
        return new RegionDimension();
      case 'source':
        return new SourceDimension();
      case 'protocol':
        return new ProtocolDimension();
      case 'has_tag':
        return new HasTagDimension(field(object, 'tag', path, asString));
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown NodeDimension variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class RegionDimension extends NodeDimension {
  readonly kind = 'region' as const;
  serialize(): unknown {
    return { kind: this.kind };
  }
}
export class SourceDimension extends NodeDimension {
  readonly kind = 'source' as const;
  serialize(): unknown {
    return { kind: this.kind };
  }
}
export class ProtocolDimension extends NodeDimension {
  readonly kind = 'protocol' as const;
  serialize(): unknown {
    return { kind: this.kind };
  }
}
export class HasTagDimension extends NodeDimension {
  readonly kind = 'has_tag' as const;
  constructor(public tag: string) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, tag: this.tag };
  }
}

export abstract class GroupStrategy implements Model {
  abstract readonly kind: 'select' | 'url_test';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): GroupStrategy {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'select':
        return new SelectGroupStrategy();
      case 'url_test':
        return new UrlTestGroupStrategy(
          field(object, 'url', path, asString),
          field(object, 'interval_secs', path, asInteger),
          field(object, 'tolerance_ms', path, asInteger),
        );
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown GroupStrategy variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class SelectGroupStrategy extends GroupStrategy {
  readonly kind = 'select' as const;
  serialize(): unknown {
    return { kind: this.kind };
  }
}
export class UrlTestGroupStrategy extends GroupStrategy {
  readonly kind = 'url_test' as const;
  constructor(
    public url: string,
    public interval_secs: number,
    public tolerance_ms: number,
  ) {
    super();
  }
  serialize(): unknown {
    return {
      kind: this.kind,
      url: this.url,
      interval_secs: this.interval_secs,
      tolerance_ms: this.tolerance_ms,
    };
  }
}

export class GroupingPolicy {
  constructor(
    public id: number,
    public group_by: NodeDimension[],
    public strategy: GroupStrategy,
  ) {}
  static deserialize(value: unknown, path = '$'): GroupingPolicy {
    const object = asObject(value, path);
    return new GroupingPolicy(
      field(object, 'id', path, asInteger),
      field(object, 'group_by', path, (item, itemPath) =>
        asArray(item, NodeDimension.deserialize, itemPath),
      ),
      field(object, 'strategy', path, GroupStrategy.deserialize),
    );
  }
  serialize(): unknown {
    return {
      id: this.id,
      group_by: this.group_by.map((item) => item.serialize()),
      strategy: this.strategy.serialize(),
    };
  }
}

export type BuiltinValue = 'Direct' | 'Reject';
const BUILTINS: readonly BuiltinValue[] = ['Direct', 'Reject'];

export abstract class Target implements Model {
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): Target {
    const [kind, raw] = externallyTagged(value, path);
    switch (kind) {
      case 'Group':
        return new GroupTarget(asInteger(raw, childPath(path, kind)));
      case 'Builtin':
        return new BuiltinTarget(asEnum(raw, BUILTINS, childPath(path, kind)));
      default:
        throw new DeserializeError(path, `unknown Target variant ${JSON.stringify(kind)}`);
    }
  }
}
export class GroupTarget extends Target {
  constructor(public group: number) {
    super();
  }
  serialize(): unknown {
    return { Group: this.group };
  }
}
export class BuiltinTarget extends Target {
  constructor(public builtin: BuiltinValue) {
    super();
  }
  serialize(): unknown {
    return { Builtin: this.builtin };
  }
}

export class NodeSelection {
  constructor(
    public source: number,
    public predicate: Predicate<NodePredicate>,
  ) {}
  static deserialize(value: unknown, path = '$'): NodeSelection {
    const object = asObject(value, path);
    return new NodeSelection(
      field(object, 'source', path, asInteger),
      field(object, 'predicate', path, (item, itemPath) =>
        Predicate.deserialize(item, NodePredicate.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return { source: this.source, predicate: this.predicate.serialize() };
  }
}
export class SourceGroupSelection {
  constructor(
    public source: number,
    public predicate: Predicate<GroupPredicate>,
  ) {}
  static deserialize(value: unknown, path = '$'): SourceGroupSelection {
    const object = asObject(value, path);
    return new SourceGroupSelection(
      field(object, 'source', path, asInteger),
      field(object, 'predicate', path, (item, itemPath) =>
        Predicate.deserialize(item, GroupPredicate.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return { source: this.source, predicate: this.predicate.serialize() };
  }
}

export type GroupScope = 'Roots' | 'All';
const GROUP_SCOPES: readonly GroupScope[] = ['Roots', 'All'];

export abstract class BaseGroupPredicate implements Model {
  abstract readonly kind: 'name' | 'depth' | 'dimension';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): BaseGroupPredicate {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const raw = required(object, 'value', path);
    switch (kind) {
      case 'name':
        return new BaseGroupNamePredicate(StringMatch.deserialize(raw, childPath(path, 'value')));
      case 'depth':
        return new BaseGroupDepthPredicate(asInteger(raw, childPath(path, 'value')));
      case 'dimension': {
        const dimension = asObject(raw, childPath(path, 'value'));
        return new BaseGroupDimensionPredicate(
          field(dimension, 'dimension', childPath(path, 'value'), NodeDimension.deserialize),
          field(dimension, 'value', childPath(path, 'value'), asString),
        );
      }
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown BaseGroupPredicate variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class BaseGroupNamePredicate extends BaseGroupPredicate {
  readonly kind = 'name' as const;
  constructor(public test: StringMatch) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.test.serialize() };
  }
}
export class BaseGroupDepthPredicate extends BaseGroupPredicate {
  readonly kind = 'depth' as const;
  constructor(public depth: number) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.depth };
  }
}
export class BaseGroupDimensionPredicate extends BaseGroupPredicate {
  readonly kind = 'dimension' as const;
  constructor(
    public dimension: NodeDimension,
    public value: string,
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: { dimension: this.dimension.serialize(), value: this.value } };
  }
}

export class BaseGroupSelection {
  constructor(
    public policy: number | null,
    public scope: GroupScope,
    public predicate: Predicate<BaseGroupPredicate>,
  ) {}
  static deserialize(value: unknown, path = '$'): BaseGroupSelection {
    const object = asObject(value, path);
    return new BaseGroupSelection(
      Object.hasOwn(object, 'policy') ? nullableField(object, 'policy', path, asInteger) : null,
      field(object, 'scope', path, (item, itemPath) => asEnum(item, GROUP_SCOPES, itemPath)),
      field(object, 'predicate', path, (item, itemPath) =>
        Predicate.deserialize(item, BaseGroupPredicate.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      ...(this.policy === null ? {} : { policy: this.policy }),
      scope: this.scope,
      predicate: this.predicate.serialize(),
    };
  }
}

export abstract class MemberSelector implements Model {
  abstract readonly kind: string;
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): MemberSelector {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const raw = required(object, 'value', path);
    switch (kind) {
      case 'nodes':
        return new NodesMemberSelector(NodeSelection.deserialize(raw, childPath(path, 'value')));
      case 'nodes_from_groups': {
        const data = asObject(raw, childPath(path, 'value'));
        return new NodesFromGroupsMemberSelector(
          field(data, 'selection', childPath(path, 'value'), SourceGroupSelection.deserialize),
          field(data, 'depth', childPath(path, 'value'), (item, itemPath) =>
            asEnum(item, ['Direct', 'Recursive'] as const, itemPath),
          ),
        );
      }
      case 'import_groups':
        return new ImportGroupsMemberSelector(
          SourceGroupSelection.deserialize(raw, childPath(path, 'value')),
        );
      case 'base_groups':
        return new BaseGroupsMemberSelector(
          BaseGroupSelection.deserialize(raw, childPath(path, 'value')),
        );
      case 'group':
        return new GroupMemberSelector(asInteger(raw, childPath(path, 'value')));
      case 'builtin':
        return new BuiltinMemberSelector(asEnum(raw, BUILTINS, childPath(path, 'value')));
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown MemberSelector variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class NodesMemberSelector extends MemberSelector {
  readonly kind = 'nodes';
  constructor(public selection: NodeSelection) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.selection.serialize() };
  }
}
export class NodesFromGroupsMemberSelector extends MemberSelector {
  readonly kind = 'nodes_from_groups';
  constructor(
    public selection: SourceGroupSelection,
    public depth: 'Direct' | 'Recursive',
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: { selection: this.selection.serialize(), depth: this.depth } };
  }
}
export class ImportGroupsMemberSelector extends MemberSelector {
  readonly kind = 'import_groups';
  constructor(public selection: SourceGroupSelection) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.selection.serialize() };
  }
}
export class BaseGroupsMemberSelector extends MemberSelector {
  readonly kind = 'base_groups';
  constructor(public selection: BaseGroupSelection) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.selection.serialize() };
  }
}
export class GroupMemberSelector extends MemberSelector {
  readonly kind = 'group';
  constructor(public group: number) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.group };
  }
}
export class BuiltinMemberSelector extends MemberSelector {
  readonly kind = 'builtin';
  constructor(public builtin: BuiltinValue) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.builtin };
  }
}

export class CustomGroup {
  constructor(
    public id: number,
    public name: string,
    public strategy: GroupStrategy,
    public member_selectors: MemberSelector[],
  ) {}
  static deserialize(value: unknown, path = '$'): CustomGroup {
    const object = asObject(value, path);
    return new CustomGroup(
      field(object, 'id', path, asInteger),
      field(object, 'name', path, asString),
      field(object, 'strategy', path, GroupStrategy.deserialize),
      field(object, 'member_selectors', path, (item, itemPath) =>
        asArray(item, MemberSelector.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      id: this.id,
      name: this.name,
      strategy: this.strategy.serialize(),
      member_selectors: this.member_selectors.map((item) => item.serialize()),
    };
  }
}

export class ManualRule {
  constructor(
    public rule: Rule,
    public target: Target,
  ) {}
  static deserialize(value: unknown, path = '$'): ManualRule {
    const object = asObject(value, path);
    return new ManualRule(
      field(object, 'rule', path, Rule.deserialize),
      field(object, 'target', path, Target.deserialize),
    );
  }
  serialize(): unknown {
    return { rule: this.rule.serialize(), target: this.target.serialize() };
  }
}
export class TargetMapping {
  constructor(
    public when: Predicate<RulePredicate>,
    public target: Target,
  ) {}
  static deserialize(value: unknown, path = '$'): TargetMapping {
    const object = asObject(value, path);
    return new TargetMapping(
      field(object, 'when', path, (item, itemPath) =>
        Predicate.deserialize(item, RulePredicate.deserialize, itemPath),
      ),
      field(object, 'target', path, Target.deserialize),
    );
  }
  serialize(): unknown {
    return { when: this.when.serialize(), target: this.target.serialize() };
  }
}

export abstract class UnmappedTargetPolicy implements Model {
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): UnmappedTargetPolicy {
    const [kind, raw] = externallyTagged(value, path);
    switch (kind) {
      case 'Error':
        return new ErrorUnmappedTargetPolicy();
      case 'Drop':
        return new DropUnmappedTargetPolicy();
      case 'Preserve':
        return new PreserveUnmappedTargetPolicy();
      case 'Use':
        return new UseUnmappedTargetPolicy(Target.deserialize(raw, childPath(path, kind)));
      default:
        throw new DeserializeError(
          path,
          `unknown UnmappedTargetPolicy variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class ErrorUnmappedTargetPolicy extends UnmappedTargetPolicy {
  serialize(): unknown {
    return 'Error';
  }
}
export class DropUnmappedTargetPolicy extends UnmappedTargetPolicy {
  serialize(): unknown {
    return 'Drop';
  }
}
export class PreserveUnmappedTargetPolicy extends UnmappedTargetPolicy {
  serialize(): unknown {
    return 'Preserve';
  }
}
export class UseUnmappedTargetPolicy extends UnmappedTargetPolicy {
  constructor(public target: Target) {
    super();
  }
  serialize(): unknown {
    return { Use: this.target.serialize() };
  }
}

export abstract class TargetBinding implements Model {
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): TargetBinding {
    const [kind, raw] = externallyTagged(value, path);
    switch (kind) {
      case 'Replace':
        return new ReplaceTargetBinding(Target.deserialize(raw, childPath(path, kind)));
      case 'Preserve':
        return new PreserveTargetBinding();
      case 'Map': {
        const map = asObject(raw, childPath(path, kind));
        return new MapTargetBinding(
          field(map, 'cases', childPath(path, kind), (item, itemPath) =>
            asArray(item, TargetMapping.deserialize, itemPath),
          ),
          field(map, 'unmatched', childPath(path, kind), UnmappedTargetPolicy.deserialize),
        );
      }
      default:
        throw new DeserializeError(path, `unknown TargetBinding variant ${JSON.stringify(kind)}`);
    }
  }
}
export class ReplaceTargetBinding extends TargetBinding {
  constructor(public target: Target) {
    super();
  }
  serialize(): unknown {
    return { Replace: this.target.serialize() };
  }
}
export class PreserveTargetBinding extends TargetBinding {
  serialize(): unknown {
    return 'Preserve';
  }
}
export class MapTargetBinding extends TargetBinding {
  constructor(
    public cases: TargetMapping[],
    public unmatched: UnmappedTargetPolicy,
  ) {
    super();
  }
  serialize(): unknown {
    return {
      Map: {
        cases: this.cases.map((item) => item.serialize()),
        unmatched: this.unmatched.serialize(),
      },
    };
  }
}

export abstract class RuleBlock implements Model {
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): RuleBlock {
    const [kind, raw] = externallyTagged(value, path);
    const data = asObject(raw, childPath(path, kind));
    switch (kind) {
      case 'Emit':
        return new EmitRuleBlock(
          field(data, 'rules', childPath(path, kind), (item, itemPath) =>
            asArray(item, ManualRule.deserialize, itemPath),
          ),
        );
      case 'Take':
        return new TakeRuleBlock(
          field(data, 'source', childPath(path, kind), asInteger),
          field(data, 'predicate', childPath(path, kind), (item, itemPath) =>
            Predicate.deserialize(item, RulePredicate.deserialize, itemPath),
          ),
          field(data, 'targets', childPath(path, kind), TargetBinding.deserialize),
        );
      default:
        throw new DeserializeError(path, `unknown RuleBlock variant ${JSON.stringify(kind)}`);
    }
  }
}
export class EmitRuleBlock extends RuleBlock {
  constructor(public rules: ManualRule[]) {
    super();
  }
  serialize(): unknown {
    return { Emit: { rules: this.rules.map((item) => item.serialize()) } };
  }
}
export class TakeRuleBlock extends RuleBlock {
  constructor(
    public source: number,
    public predicate: Predicate<RulePredicate>,
    public targets: TargetBinding,
  ) {
    super();
  }
  serialize(): unknown {
    return {
      Take: {
        source: this.source,
        predicate: this.predicate.serialize(),
        targets: this.targets.serialize(),
      },
    };
  }
}

export class Output {
  constructor(
    public roots: number[],
    public extra_nodes: NodeSelection[],
    public fallback: Target,
    public settings_source: number,
  ) {}
  static deserialize(value: unknown, path = '$'): Output {
    const object = asObject(value, path);
    return new Output(
      field(object, 'roots', path, (item, itemPath) => asArray(item, asInteger, itemPath)),
      field(object, 'extra_nodes', path, (item, itemPath) =>
        asArray(item, NodeSelection.deserialize, itemPath),
      ),
      field(object, 'fallback', path, Target.deserialize),
      field(object, 'settings_source', path, asInteger),
    );
  }
  serialize(): unknown {
    return {
      roots: [...this.roots],
      extra_nodes: this.extra_nodes.map((item) => item.serialize()),
      fallback: this.fallback.serialize(),
      settings_source: this.settings_source,
    };
  }
}

export class Plan {
  constructor(
    public version: number,
    public client: ProxyClient,
    public sources: Source[],
    public grouping_policies: GroupingPolicy[],
    public groups: CustomGroup[],
    public rules: RuleBlock[],
    public output: Output,
  ) {}

  static empty(client = ProxyClient.Surge): Plan {
    return new Plan(1, client, [], [], [], [], new Output([], [], new BuiltinTarget('Direct'), 0));
  }

  clone(): Plan {
    return Plan.deserialize(this.serialize());
  }

  static deserialize(value: unknown, path = '$'): Plan {
    const object = asObject(value, path);
    return new Plan(
      field(object, 'version', path, asInteger),
      field(object, 'client', path, ProxyClientSerde.deserialize),
      field(object, 'sources', path, (item, itemPath) =>
        asArray(item, Source.deserialize, itemPath),
      ),
      field(object, 'grouping_policies', path, (item, itemPath) =>
        asArray(item, GroupingPolicy.deserialize, itemPath),
      ),
      field(object, 'groups', path, (item, itemPath) =>
        asArray(item, CustomGroup.deserialize, itemPath),
      ),
      field(object, 'rules', path, (item, itemPath) =>
        asArray(item, RuleBlock.deserialize, itemPath),
      ),
      field(object, 'output', path, Output.deserialize),
    );
  }

  serialize(): unknown {
    return {
      version: this.version,
      client: ProxyClientSerde.serialize(this.client),
      sources: this.sources.map((item) => item.serialize()),
      grouping_policies: this.grouping_policies.map((item) => item.serialize()),
      groups: this.groups.map((item) => item.serialize()),
      rules: this.rules.map((item) => item.serialize()),
      output: this.output.serialize(),
    };
  }
}
