import {
  DeserializeError,
  JsonObject,
  asArray,
  asBoolean,
  asEnum,
  asInteger,
  asObject,
  asRecord,
  asString,
  asStringArray,
  childPath,
  nullable,
  required,
} from '../../deserialize';

/**
 * TypeScript mirror of Rust's common Profile document.
 *
 * Every factory validates untrusted JSON recursively. Client-specific unknown
 * values remain in `extra`; unrelated unknown object fields follow Serde and
 * are ignored.
 */

export type ExtraFields = Record<string, unknown>;

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

export abstract class ExternalResource {
  abstract readonly kind: 'http' | 'file';
  constructor(public value: string) {}
  abstract serialize(): unknown;

  static deserialize(value: unknown, path = '$'): ExternalResource {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const resource = field(object, 'value', path, asString);
    switch (kind) {
      case 'http':
        return new HttpResource(resource);
      case 'file':
        return new FileResource(resource);
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown ExternalResource variant ${JSON.stringify(kind)}`,
        );
    }
  }
}

export class HttpResource extends ExternalResource {
  readonly kind = 'http' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export class FileResource extends ExternalResource {
  readonly kind = 'file' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export abstract class PolicyRef {
  abstract readonly kind: 'named' | 'built_in';
  constructor(public value: string) {}
  abstract serialize(): unknown;

  static deserialize(value: unknown, path = '$'): PolicyRef {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const name = field(object, 'value', path, asString);
    switch (kind) {
      case 'named':
        return new NamedPolicyRef(name);
      case 'built_in':
        return new BuiltInPolicyRef(name);
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown PolicyRef variant ${JSON.stringify(kind)}`,
        );
    }
  }
}

export class NamedPolicyRef extends PolicyRef {
  readonly kind = 'named' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export class BuiltInPolicyRef extends PolicyRef {
  readonly kind = 'built_in' as const;
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export abstract class SectionEntry<T extends { serialize(): unknown }> {
  abstract readonly kind: 'item' | 'include' | 'comment';
  abstract serialize(): unknown;

  static deserialize<T extends { serialize(): unknown }>(
    value: unknown,
    deserializeItem: (value: unknown, path?: string) => T,
    path = '$',
  ): SectionEntry<T> {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const raw = required(object, 'value', path);
    switch (kind) {
      case 'item':
        return new ItemEntry(deserializeItem(raw, childPath(path, 'value')));
      case 'include': {
        const include = asObject(raw, childPath(path, 'value'));
        return new IncludeEntry(
          field(include, 'sources', childPath(path, 'value'), (item, itemPath) =>
            asArray(item, ExternalResource.deserialize, itemPath),
          ),
          nullableField(include, 'comment', childPath(path, 'value'), asString),
        );
      }
      case 'comment':
        return new CommentEntry(field(object, 'value', path, asString));
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown SectionEntry variant ${JSON.stringify(kind)}`,
        );
    }
  }
}

export class ItemEntry<T extends { serialize(): unknown }> extends SectionEntry<T> {
  readonly kind = 'item' as const;
  constructor(public value: T) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.value.serialize() };
  }
}

export class IncludeEntry<T extends { serialize(): unknown }> extends SectionEntry<T> {
  readonly kind = 'include' as const;
  constructor(
    public sources: ExternalResource[],
    public comment: string | null,
  ) {
    super();
  }
  serialize(): unknown {
    return {
      kind: this.kind,
      value: { sources: this.sources.map((source) => source.serialize()), comment: this.comment },
    };
  }
}

export class CommentEntry<T extends { serialize(): unknown }> extends SectionEntry<T> {
  readonly kind = 'comment' as const;
  constructor(public value: string) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.value };
  }
}

export class Proxy {
  constructor(
    public name: string,
    public protocol: string,
    public server: string,
    public port: number,
    public password: string | null,
    public cipher: string | null,
    public sni: string | null,
    public udp: boolean | null,
    public tfo: boolean | null,
    public skip_cert_verify: boolean | null,
    public tags: string[],
    public extra: ExtraFields,
    public comment: string | null,
  ) {}

  static deserialize(value: unknown, path = '$'): Proxy {
    const object = asObject(value, path);
    return new Proxy(
      field(object, 'name', path, asString),
      field(object, 'protocol', path, asString),
      field(object, 'server', path, asString),
      field(object, 'port', path, asInteger),
      nullableField(object, 'password', path, asString),
      nullableField(object, 'cipher', path, asString),
      nullableField(object, 'sni', path, asString),
      nullableField(object, 'udp', path, asBoolean),
      nullableField(object, 'tfo', path, asBoolean),
      nullableField(object, 'skip_cert_verify', path, asBoolean),
      field(object, 'tags', path, asStringArray),
      field(object, 'extra', path, asRecord),
      nullableField(object, 'comment', path, asString),
    );
  }

  serialize(): unknown {
    return {
      name: this.name,
      protocol: this.protocol,
      server: this.server,
      port: this.port,
      password: this.password,
      cipher: this.cipher,
      sni: this.sni,
      udp: this.udp,
      tfo: this.tfo,
      skip_cert_verify: this.skip_cert_verify,
      tags: [...this.tags],
      extra: { ...this.extra },
      comment: this.comment,
    };
  }
}

export class HttpHeader {
  constructor(
    public name: string,
    public values: string[],
  ) {}
  static deserialize(value: unknown, path = '$'): HttpHeader {
    const object = asObject(value, path);
    return new HttpHeader(
      field(object, 'name', path, asString),
      field(object, 'values', path, asStringArray),
    );
  }
  serialize(): unknown {
    return { name: this.name, values: [...this.values] };
  }
}

export abstract class ProviderSource {
  abstract readonly kind: 'inline' | 'external';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): ProviderSource {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    switch (kind) {
      case 'inline':
        return new InlineProviderSource();
      case 'external':
        return new ExternalProviderSource(
          ExternalResource.deserialize(required(object, 'value', path), childPath(path, 'value')),
        );
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown ProviderSource variant ${JSON.stringify(kind)}`,
        );
    }
  }
}

export class InlineProviderSource extends ProviderSource {
  readonly kind = 'inline' as const;
  serialize(): unknown {
    return { kind: this.kind };
  }
}

export class ExternalProviderSource extends ProviderSource {
  readonly kind = 'external' as const;
  constructor(public resource: ExternalResource) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.resource.serialize() };
  }
}

export class HealthCheck {
  constructor(
    public enabled: boolean | null,
    public url: string | null,
    public interval: number | null,
    public timeout: number | null,
    public lazy: boolean | null,
    public expected_status: string | null,
    public extra: ExtraFields,
  ) {}
  static deserialize(value: unknown, path = '$'): HealthCheck {
    const object = asObject(value, path);
    return new HealthCheck(
      nullableField(object, 'enabled', path, asBoolean),
      nullableField(object, 'url', path, asString),
      nullableField(object, 'interval', path, asInteger),
      nullableField(object, 'timeout', path, asInteger),
      nullableField(object, 'lazy', path, asBoolean),
      nullableField(object, 'expected_status', path, asString),
      field(object, 'extra', path, asRecord),
    );
  }
  serialize(): unknown {
    return {
      enabled: this.enabled,
      url: this.url,
      interval: this.interval,
      timeout: this.timeout,
      lazy: this.lazy,
      expected_status: this.expected_status,
      extra: { ...this.extra },
    };
  }
}

export class ProxyProvider {
  constructor(
    public name: string,
    public source: ProviderSource,
    public payload: Proxy[] | null,
    public update_interval: number | null,
    public request_headers: HttpHeader[],
    public cache_path: string | null,
    public download_via: PolicyRef | null,
    public size_limit: number | null,
    public health_check: HealthCheck | null,
    public filter: string | null,
    public exclude_filter: string | null,
    public exclude_types: string[],
    public overrides: ExtraFields,
    public extra: ExtraFields,
    public comment: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): ProxyProvider {
    const object = asObject(value, path);
    return new ProxyProvider(
      field(object, 'name', path, asString),
      field(object, 'source', path, ProviderSource.deserialize),
      nullableField(object, 'payload', path, (item, itemPath) =>
        asArray(item, Proxy.deserialize, itemPath),
      ),
      nullableField(object, 'update_interval', path, asInteger),
      field(object, 'request_headers', path, (item, itemPath) =>
        asArray(item, HttpHeader.deserialize, itemPath),
      ),
      nullableField(object, 'cache_path', path, asString),
      nullableField(object, 'download_via', path, PolicyRef.deserialize),
      nullableField(object, 'size_limit', path, asInteger),
      nullableField(object, 'health_check', path, HealthCheck.deserialize),
      nullableField(object, 'filter', path, asString),
      nullableField(object, 'exclude_filter', path, asString),
      field(object, 'exclude_types', path, asStringArray),
      field(object, 'overrides', path, asRecord),
      field(object, 'extra', path, asRecord),
      nullableField(object, 'comment', path, asString),
    );
  }
  serialize(): unknown {
    return {
      name: this.name,
      source: this.source.serialize(),
      payload: this.payload?.map((proxy) => proxy.serialize()) ?? null,
      update_interval: this.update_interval,
      request_headers: this.request_headers.map((header) => header.serialize()),
      cache_path: this.cache_path,
      download_via: this.download_via?.serialize() ?? null,
      size_limit: this.size_limit,
      health_check: this.health_check?.serialize() ?? null,
      filter: this.filter,
      exclude_filter: this.exclude_filter,
      exclude_types: [...this.exclude_types],
      overrides: { ...this.overrides },
      extra: { ...this.extra },
      comment: this.comment,
    };
  }
}

export type RuleType =
  | 'DOMAIN'
  | 'DOMAIN-SUFFIX'
  | 'DOMAIN-KEYWORD'
  | 'PROCESS-NAME'
  | 'USER-AGENT'
  | 'RULE-SET'
  | 'GEOIP'
  | 'IP-CIDR'
  | 'IP-CIDR6'
  | 'FINAL'
  | 'MATCH';
const RULE_TYPES: readonly RuleType[] = [
  'DOMAIN',
  'DOMAIN-SUFFIX',
  'DOMAIN-KEYWORD',
  'PROCESS-NAME',
  'USER-AGENT',
  'RULE-SET',
  'GEOIP',
  'IP-CIDR',
  'IP-CIDR6',
  'FINAL',
  'MATCH',
];

export class Rule {
  constructor(
    public rule_type: RuleType,
    public value: string | null,
    public target: PolicyRef | null,
    public options: string[],
    public comment: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): Rule {
    const object = asObject(value, path);
    return new Rule(
      field(object, 'rule_type', path, (item, itemPath) => asEnum(item, RULE_TYPES, itemPath)),
      nullableField(object, 'value', path, asString),
      nullableField(object, 'target', path, PolicyRef.deserialize),
      field(object, 'options', path, asStringArray),
      nullableField(object, 'comment', path, asString),
    );
  }
  serialize(): unknown {
    return {
      rule_type: this.rule_type,
      value: this.value,
      target: this.target?.serialize() ?? null,
      options: [...this.options],
      comment: this.comment,
    };
  }
}

export abstract class RuleProviderPayload {
  abstract readonly kind: 'classical' | 'domain' | 'ip_cidr';
  abstract serialize(): unknown;
  static deserialize(value: unknown, path = '$'): RuleProviderPayload {
    const object = asObject(value, path);
    const kind = field(object, 'kind', path, asString);
    const raw = required(object, 'value', path);
    switch (kind) {
      case 'classical':
        return new ClassicalRulePayload(
          asArray(raw, deserializeRuleEntry, childPath(path, 'value')),
        );
      case 'domain':
        return new DomainRulePayload(asStringArray(raw, childPath(path, 'value')));
      case 'ip_cidr':
        return new IpCidrRulePayload(asStringArray(raw, childPath(path, 'value')));
      default:
        throw new DeserializeError(
          childPath(path, 'kind'),
          `unknown RuleProviderPayload variant ${JSON.stringify(kind)}`,
        );
    }
  }
}

export class ClassicalRulePayload extends RuleProviderPayload {
  readonly kind = 'classical' as const;
  constructor(public entries: RuleEntry[]) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: this.entries.map((entry) => entry.serialize()) };
  }
}
export class DomainRulePayload extends RuleProviderPayload {
  readonly kind = 'domain' as const;
  constructor(public domains: string[]) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: [...this.domains] };
  }
}
export class IpCidrRulePayload extends RuleProviderPayload {
  readonly kind = 'ip_cidr' as const;
  constructor(public cidrs: string[]) {
    super();
  }
  serialize(): unknown {
    return { kind: this.kind, value: [...this.cidrs] };
  }
}

export type RuleBehavior = 'classical' | 'domain' | 'ipcidr';
export type RuleFormat = 'yaml' | 'text' | 'mrs';
const RULE_BEHAVIORS: readonly RuleBehavior[] = ['classical', 'domain', 'ipcidr'];
const RULE_FORMATS: readonly RuleFormat[] = ['yaml', 'text', 'mrs'];

export class RuleProvider {
  constructor(
    public name: string,
    public source: ProviderSource,
    public payload: RuleProviderPayload | null,
    public update_interval: number | null,
    public request_headers: HttpHeader[],
    public cache_path: string | null,
    public download_via: PolicyRef | null,
    public size_limit: number | null,
    public behavior: RuleBehavior | null,
    public format: RuleFormat | null,
    public extra: ExtraFields,
    public comment: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): RuleProvider {
    const object = asObject(value, path);
    return new RuleProvider(
      field(object, 'name', path, asString),
      field(object, 'source', path, ProviderSource.deserialize),
      nullableField(object, 'payload', path, RuleProviderPayload.deserialize),
      nullableField(object, 'update_interval', path, asInteger),
      field(object, 'request_headers', path, (item, itemPath) =>
        asArray(item, HttpHeader.deserialize, itemPath),
      ),
      nullableField(object, 'cache_path', path, asString),
      nullableField(object, 'download_via', path, PolicyRef.deserialize),
      nullableField(object, 'size_limit', path, asInteger),
      nullableField(object, 'behavior', path, (item, itemPath) =>
        asEnum(item, RULE_BEHAVIORS, itemPath),
      ),
      nullableField(object, 'format', path, (item, itemPath) =>
        asEnum(item, RULE_FORMATS, itemPath),
      ),
      field(object, 'extra', path, asRecord),
      nullableField(object, 'comment', path, asString),
    );
  }
  serialize(): unknown {
    return {
      name: this.name,
      source: this.source.serialize(),
      payload: this.payload?.serialize() ?? null,
      update_interval: this.update_interval,
      request_headers: this.request_headers.map((header) => header.serialize()),
      cache_path: this.cache_path,
      download_via: this.download_via?.serialize() ?? null,
      size_limit: this.size_limit,
      behavior: this.behavior,
      format: this.format,
      extra: { ...this.extra },
      comment: this.comment,
    };
  }
}

export type ProxyGroupType = 'select' | 'url-test' | 'smart';
const GROUP_TYPES: readonly ProxyGroupType[] = ['select', 'url-test', 'smart'];

export class PolicyPath {
  constructor(
    public resource: ExternalResource,
    public update_interval: number | null,
  ) {}
  static deserialize(value: unknown, path = '$'): PolicyPath {
    const object = asObject(value, path);
    return new PolicyPath(
      field(object, 'resource', path, ExternalResource.deserialize),
      nullableField(object, 'update_interval', path, asInteger),
    );
  }
  serialize(): unknown {
    return { resource: this.resource.serialize(), update_interval: this.update_interval };
  }
}

export class GroupOptions {
  constructor(
    public url: string | null,
    public interval: number | null,
    public tolerance: number | null,
    public timeout: number | null,
    public lazy: boolean | null,
    public expected_status: string | null,
    public filter: string | null,
    public exclude_filter: string | null,
    public exclude_types: string[],
    public extra: ExtraFields,
  ) {}
  static deserialize(value: unknown, path = '$'): GroupOptions {
    const object = asObject(value, path);
    return new GroupOptions(
      nullableField(object, 'url', path, asString),
      nullableField(object, 'interval', path, asInteger),
      nullableField(object, 'tolerance', path, asInteger),
      nullableField(object, 'timeout', path, asInteger),
      nullableField(object, 'lazy', path, asBoolean),
      nullableField(object, 'expected_status', path, asString),
      nullableField(object, 'filter', path, asString),
      nullableField(object, 'exclude_filter', path, asString),
      field(object, 'exclude_types', path, asStringArray),
      field(object, 'extra', path, asRecord),
    );
  }
  serialize(): unknown {
    return {
      url: this.url,
      interval: this.interval,
      tolerance: this.tolerance,
      timeout: this.timeout,
      lazy: this.lazy,
      expected_status: this.expected_status,
      filter: this.filter,
      exclude_filter: this.exclude_filter,
      exclude_types: [...this.exclude_types],
      extra: { ...this.extra },
    };
  }
}

export class ProxyGroup {
  constructor(
    public name: string,
    public strategy: ProxyGroupType,
    public members: PolicyRef[],
    public providers: string[],
    public policy_path: PolicyPath | null,
    public options: GroupOptions,
    public comment: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): ProxyGroup {
    const object = asObject(value, path);
    return new ProxyGroup(
      field(object, 'name', path, asString),
      field(object, 'strategy', path, (item, itemPath) => asEnum(item, GROUP_TYPES, itemPath)),
      field(object, 'members', path, (item, itemPath) =>
        asArray(item, PolicyRef.deserialize, itemPath),
      ),
      field(object, 'providers', path, asStringArray),
      nullableField(object, 'policy_path', path, PolicyPath.deserialize),
      field(object, 'options', path, GroupOptions.deserialize),
      nullableField(object, 'comment', path, asString),
    );
  }
  serialize(): unknown {
    return {
      name: this.name,
      strategy: this.strategy,
      members: this.members.map((member) => member.serialize()),
      providers: [...this.providers],
      policy_path: this.policy_path?.serialize() ?? null,
      options: this.options.serialize(),
      comment: this.comment,
    };
  }
}

export type ProxyEntry = SectionEntry<Proxy>;
export type ProxyGroupEntry = SectionEntry<ProxyGroup>;
export type RuleEntry = SectionEntry<Rule>;
export const deserializeProxyEntry = (value: unknown, path = '$'): ProxyEntry =>
  SectionEntry.deserialize(value, Proxy.deserialize, path);
export const deserializeProxyGroupEntry = (value: unknown, path = '$'): ProxyGroupEntry =>
  SectionEntry.deserialize(value, ProxyGroup.deserialize, path);
export const deserializeRuleEntry = (value: unknown, path = '$'): RuleEntry =>
  SectionEntry.deserialize(value, Rule.deserialize, path);

export class Profile {
  constructor(
    public proxies: ProxyEntry[],
    public proxy_providers: ProxyProvider[],
    public proxy_groups: ProxyGroupEntry[],
    public rule_providers: RuleProvider[],
    public rules: RuleEntry[],
  ) {}

  static empty(): Profile {
    return new Profile([], [], [], [], []);
  }

  static deserialize(value: unknown, path = '$'): Profile {
    const object = asObject(value, path);
    return new Profile(
      field(object, 'proxies', path, (item, itemPath) =>
        asArray(item, deserializeProxyEntry, itemPath),
      ),
      field(object, 'proxy_providers', path, (item, itemPath) =>
        asArray(item, ProxyProvider.deserialize, itemPath),
      ),
      field(object, 'proxy_groups', path, (item, itemPath) =>
        asArray(item, deserializeProxyGroupEntry, itemPath),
      ),
      field(object, 'rule_providers', path, (item, itemPath) =>
        asArray(item, RuleProvider.deserialize, itemPath),
      ),
      field(object, 'rules', path, (item, itemPath) =>
        asArray(item, deserializeRuleEntry, itemPath),
      ),
    );
  }

  serialize(): unknown {
    return {
      proxies: this.proxies.map((entry) => entry.serialize()),
      proxy_providers: this.proxy_providers.map((provider) => provider.serialize()),
      proxy_groups: this.proxy_groups.map((entry) => entry.serialize()),
      rule_providers: this.rule_providers.map((provider) => provider.serialize()),
      rules: this.rules.map((entry) => entry.serialize()),
    };
  }
}
