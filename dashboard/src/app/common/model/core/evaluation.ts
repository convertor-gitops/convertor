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
import { NodeDimension } from './plan';
import { Profile, Proxy, Rule } from './profile';
import { ProxyClient, ProxyClientSerde } from './proxy-client';

/** Loaded source state and evaluator report returned by the workbench API. */

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

export type DiagnosticSeverity = 'warning' | 'error';
const DIAGNOSTIC_SEVERITIES: readonly DiagnosticSeverity[] = ['warning', 'error'];

export class Diagnostic {
  constructor(
    public severity: DiagnosticSeverity,
    public code: string,
    public path: string,
    public message: string,
  ) {}
  static deserialize(value: unknown, path = '$'): Diagnostic {
    const object = asObject(value, path);
    return new Diagnostic(
      field(object, 'severity', path, (item, itemPath) =>
        asEnum(item, DIAGNOSTIC_SEVERITIES, itemPath),
      ),
      field(object, 'code', path, asString),
      field(object, 'path', path, asString),
      field(object, 'message', path, asString),
    );
  }
  serialize(): unknown {
    return { severity: this.severity, code: this.code, path: this.path, message: this.message };
  }
}

export class NodeDependency {
  constructor(
    public source: number,
    public key: string,
    public nodes: Proxy[],
  ) {}
  static deserialize(value: unknown, path = '$'): NodeDependency {
    const object = asObject(value, path);
    return new NodeDependency(
      field(object, 'source', path, asInteger),
      field(object, 'key', path, asString),
      field(object, 'nodes', path, (item, itemPath) => asArray(item, Proxy.deserialize, itemPath)),
    );
  }
  serialize(): unknown {
    return {
      source: this.source,
      key: this.key,
      nodes: this.nodes.map((node) => node.serialize()),
    };
  }
}
export class RuleDependency {
  constructor(
    public source: number,
    public key: string,
    public rules: Rule[],
  ) {}
  static deserialize(value: unknown, path = '$'): RuleDependency {
    const object = asObject(value, path);
    return new RuleDependency(
      field(object, 'source', path, asInteger),
      field(object, 'key', path, asString),
      field(object, 'rules', path, (item, itemPath) => asArray(item, Rule.deserialize, itemPath)),
    );
  }
  serialize(): unknown {
    return {
      source: this.source,
      key: this.key,
      rules: this.rules.map((rule) => rule.serialize()),
    };
  }
}
export class ResolvedDependencies {
  constructor(
    public nodes: NodeDependency[],
    public rules: RuleDependency[],
  ) {}
  static deserialize(value: unknown, path = '$'): ResolvedDependencies {
    const object = asObject(value, path);
    return new ResolvedDependencies(
      field(object, 'nodes', path, (item, itemPath) =>
        asArray(item, NodeDependency.deserialize, itemPath),
      ),
      field(object, 'rules', path, (item, itemPath) =>
        asArray(item, RuleDependency.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      nodes: this.nodes.map((item) => item.serialize()),
      rules: this.rules.map((item) => item.serialize()),
    };
  }
}

export class SourceProfile {
  constructor(
    public source_id: number,
    public client: ProxyClient,
    public input_fingerprint: string,
    public fingerprint: string,
    public loaded_at: number,
    public content: string,
    public profile: Profile,
    public dependencies: ResolvedDependencies,
    public diagnostics: Diagnostic[],
    public input_nodes: InputNode[] = [],
    public input_diagnostics: Diagnostic[] = [],
  ) {}
  static deserialize(value: unknown, path = '$'): SourceProfile {
    const object = asObject(value, path);
    return new SourceProfile(
      field(object, 'source_id', path, asInteger),
      field(object, 'client', path, ProxyClientSerde.deserialize),
      field(object, 'input_fingerprint', path, asString),
      field(object, 'fingerprint', path, asString),
      field(object, 'loaded_at', path, asInteger),
      field(object, 'content', path, asString),
      field(object, 'profile', path, Profile.deserialize),
      field(object, 'dependencies', path, ResolvedDependencies.deserialize),
      field(object, 'diagnostics', path, (item, itemPath) =>
        asArray(item, Diagnostic.deserialize, itemPath),
      ),
      object['input_nodes'] === undefined
        ? []
        : asArray(object['input_nodes'], InputNode.deserialize, childPath(path, 'input_nodes')),
      object['input_diagnostics'] === undefined
        ? []
        : asArray(object['input_diagnostics'], Diagnostic.deserialize, childPath(path, 'input_diagnostics')),
    );
  }
  serialize(): unknown {
    return {
      source_id: this.source_id,
      client: ProxyClientSerde.serialize(this.client),
      input_fingerprint: this.input_fingerprint,
      fingerprint: this.fingerprint,
      loaded_at: this.loaded_at,
      content: this.content,
      profile: this.profile.serialize(),
      dependencies: this.dependencies.serialize(),
      diagnostics: this.diagnostics.map((item) => item.serialize()),
    };
  }
}

export abstract class NodeOrigin {
  abstract readonly kind: 'direct' | 'proxy_provider' | 'policy_path';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): NodeOrigin {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'direct':
        return new DirectNodeOrigin(field(object, 'index', path, asInteger));
      case 'proxy_provider':
        return new ProxyProviderNodeOrigin(
          field(object, 'name', path, asString),
          field(object, 'index', path, asInteger),
        );
      case 'policy_path':
        return new PolicyPathNodeOrigin(
          field(object, 'group_index', path, asInteger),
          field(object, 'index', path, asInteger),
        );
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown NodeOrigin variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class DirectNodeOrigin extends NodeOrigin {
  readonly kind = 'direct' as const;
  constructor(public index: number) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, index: this.index };
  }
}
export class ProxyProviderNodeOrigin extends NodeOrigin {
  readonly kind = 'proxy_provider' as const;
  constructor(
    public name: string,
    public index: number,
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, name: this.name, index: this.index };
  }
}
export class PolicyPathNodeOrigin extends NodeOrigin {
  readonly kind = 'policy_path' as const;
  constructor(
    public group_index: number,
    public index: number,
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, group_index: this.group_index, index: this.index };
  }
}

export class InputNode {
  constructor(
    public identity: string,
    public source: number,
    public proxy: Proxy,
    public origins: NodeOrigin[],
    public region: string,
  ) {}
  static deserialize(value: unknown, path = '$'): InputNode {
    const object = asObject(value, path);
    return new InputNode(
      field(object, 'identity', path, asString),
      field(object, 'source', path, asInteger),
      field(object, 'proxy', path, Proxy.deserialize),
      field(object, 'origins', path, (item, itemPath) =>
        asArray(item, NodeOrigin.deserialize, itemPath),
      ),
      field(object, 'region', path, asString),
    );
  }
}

export class EvaluatedNode {
  constructor(
    public identity: string,
    public source: number,
    public origins: NodeOrigin[],
    public proxy: Proxy,
    public original_tags: string[],
    public effective_tags: string[],
    public kept: boolean,
    public reachable: boolean,
    public output_name: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): EvaluatedNode {
    const object = asObject(value, path);
    return new EvaluatedNode(
      field(object, 'identity', path, asString),
      field(object, 'source', path, asInteger),
      field(object, 'origins', path, (item, itemPath) =>
        asArray(item, NodeOrigin.deserialize, itemPath),
      ),
      field(object, 'proxy', path, Proxy.deserialize),
      field(object, 'original_tags', path, asStringArray),
      field(object, 'effective_tags', path, asStringArray),
      field(object, 'kept', path, asBoolean),
      field(object, 'reachable', path, asBoolean),
      nullableField(object, 'output_name', path, asString),
    );
  }
  serialize(): unknown {
    return {
      identity: this.identity,
      source: this.source,
      origins: this.origins.map((item) => item.serialize()),
      proxy: this.proxy.serialize(),
      original_tags: [...this.original_tags],
      effective_tags: [...this.effective_tags],
      kept: this.kept,
      reachable: this.reachable,
      output_name: this.output_name,
    };
  }
}

export abstract class EvaluatedGroupKind {
  abstract readonly kind: 'base' | 'custom' | 'imported';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): EvaluatedGroupKind {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'base':
        return new BaseEvaluatedGroupKind(
          field(object, 'policy', path, asInteger),
          field(object, 'depth', path, asInteger),
        );
      case 'custom':
        return new CustomEvaluatedGroupKind(field(object, 'id', path, asInteger));
      case 'imported':
        return new ImportedEvaluatedGroupKind(
          field(object, 'source', path, asInteger),
          field(object, 'index', path, asInteger),
        );
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown EvaluatedGroupKind variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class BaseEvaluatedGroupKind extends EvaluatedGroupKind {
  readonly kind = 'base' as const;
  constructor(
    public policy: number,
    public depth: number,
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, policy: this.policy, depth: this.depth };
  }
}
export class CustomEvaluatedGroupKind extends EvaluatedGroupKind {
  readonly kind = 'custom' as const;
  constructor(public id: number) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, id: this.id };
  }
}
export class ImportedEvaluatedGroupKind extends EvaluatedGroupKind {
  readonly kind = 'imported' as const;
  constructor(
    public source: number,
    public index: number,
  ) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, source: this.source, index: this.index };
  }
}

export abstract class EvaluatedMemberRef {
  abstract readonly kind: 'node' | 'group' | 'builtin';
  constructor(public value: string) {}
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): EvaluatedMemberRef {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const name = field(object, 'value', path, asString);
    switch (kind) {
      case 'node':
        return new EvaluatedNodeRef(name);
      case 'group':
        return new EvaluatedGroupRef(name);
      case 'builtin':
        return new EvaluatedBuiltinRef(name);
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown EvaluatedMemberRef variant ${JSON.stringify(kind)}`,
        );
    }
  }
}
export class EvaluatedNodeRef extends EvaluatedMemberRef {
  readonly kind = 'node' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}
export class EvaluatedGroupRef extends EvaluatedMemberRef {
  readonly kind = 'group' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}
export class EvaluatedBuiltinRef extends EvaluatedMemberRef {
  readonly kind = 'builtin' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export class EvaluatedGroup {
  constructor(
    public identity: string,
    public kind: EvaluatedGroupKind,
    public name: string,
    public members: EvaluatedMemberRef[],
    public valid: boolean,
    public reachable: boolean,
    public output_name: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): EvaluatedGroup {
    const object = asObject(value, path);
    return new EvaluatedGroup(
      field(object, 'identity', path, asString),
      field(object, 'kind', path, EvaluatedGroupKind.deserialize),
      field(object, 'name', path, asString),
      field(object, 'members', path, (item, itemPath) =>
        asArray(item, EvaluatedMemberRef.deserialize, itemPath),
      ),
      field(object, 'valid', path, asBoolean),
      field(object, 'reachable', path, asBoolean),
      nullableField(object, 'output_name', path, asString),
    );
  }
  serialize(): unknown {
    return {
      identity: this.identity,
      kind: this.kind.serialize(),
      name: this.name,
      members: this.members.map((item) => item.serialize()),
      valid: this.valid,
      reachable: this.reachable,
      output_name: this.output_name,
    };
  }
}

export class Trace {
  constructor(
    public path: string,
    public resources: string[],
  ) {}
  static deserialize(value: unknown, path = '$'): Trace {
    const object = asObject(value, path);
    return new Trace(
      field(object, 'path', path, asString),
      field(object, 'resources', path, asStringArray),
    );
  }
  serialize(): unknown {
    return { path: this.path, resources: [...this.resources] };
  }
}
export class DimensionValue {
  constructor(
    public dimension: NodeDimension,
    public value: string,
  ) {}
  static deserialize(value: unknown, path = '$'): DimensionValue {
    const object = asObject(value, path);
    return new DimensionValue(
      field(object, 'dimension', path, NodeDimension.deserialize),
      field(object, 'value', path, asString),
    );
  }
  serialize(): unknown {
    return { dimension: this.dimension.serialize(), value: this.value };
  }
}
export class BaseGroup {
  constructor(
    public identity: string,
    public output_name: string | null,
    public policy: number,
    public name: string,
    public depth: number,
    public parent: string | null,
    public dimensions: DimensionValue[],
  ) {}
  static deserialize(value: unknown, path = '$'): BaseGroup {
    const object = asObject(value, path);
    return new BaseGroup(
      field(object, 'identity', path, asString),
      nullableField(object, 'output_name', path, asString),
      field(object, 'policy', path, asInteger),
      field(object, 'name', path, asString),
      field(object, 'depth', path, asInteger),
      nullableField(object, 'parent', path, asString),
      field(object, 'dimensions', path, (item, itemPath) =>
        asArray(item, DimensionValue.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      identity: this.identity,
      output_name: this.output_name,
      policy: this.policy,
      name: this.name,
      depth: this.depth,
      parent: this.parent,
      dimensions: this.dimensions.map((item) => item.serialize()),
    };
  }
}

export class EvaluationReport {
  constructor(
    public nodes: EvaluatedNode[],
    public groups: EvaluatedGroup[],
    public base_groups: BaseGroup[],
    public diagnostics: Diagnostic[],
    public trace: Trace[],
    public profile: Profile | null,
  ) {}
  get hasErrors(): boolean {
    return this.diagnostics.some((item) => item.severity === 'error');
  }
  static deserialize(value: unknown, path = '$'): EvaluationReport {
    const object = asObject(value, path);
    return new EvaluationReport(
      field(object, 'nodes', path, (item, itemPath) =>
        asArray(item, EvaluatedNode.deserialize, itemPath),
      ),
      field(object, 'groups', path, (item, itemPath) =>
        asArray(item, EvaluatedGroup.deserialize, itemPath),
      ),
      field(object, 'base_groups', path, (item, itemPath) =>
        asArray(item, BaseGroup.deserialize, itemPath),
      ),
      field(object, 'diagnostics', path, (item, itemPath) =>
        asArray(item, Diagnostic.deserialize, itemPath),
      ),
      field(object, 'trace', path, (item, itemPath) => asArray(item, Trace.deserialize, itemPath)),
      nullableField(object, 'profile', path, Profile.deserialize),
    );
  }
  serialize(): unknown {
    return {
      nodes: this.nodes.map((item) => item.serialize()),
      groups: this.groups.map((item) => item.serialize()),
      base_groups: this.base_groups.map((item) => item.serialize()),
      diagnostics: this.diagnostics.map((item) => item.serialize()),
      trace: this.trace.map((item) => item.serialize()),
      profile: this.profile?.serialize() ?? null,
    };
  }
}
