import {
  JsonObject,
  asArray,
  asEnum,
  asInteger,
  asObject,
  asString,
  childPath,
  required,
} from '../../deserialize';
import { Diagnostic, EvaluationReport, SourceProfile } from '../core/evaluation';
import { Plan, SourceInput } from '../core/plan';
import { ProxyClient, ProxyClientSerde } from '../core/proxy-client';

function field<T>(
  object: JsonObject,
  key: string,
  path: string,
  deserialize: (value: unknown, path: string) => T,
): T {
  return deserialize(required(object, key, path), childPath(path, key));
}

export type SourceCacheMode = 'use' | 'refresh';

export class LoadSourceRequest {
  constructor(
    public source_id: number,
    public client: ProxyClient,
    public input: SourceInput,
    public cache: SourceCacheMode,
  ) {}
  static deserialize(value: unknown, path = '$'): LoadSourceRequest {
    const object = asObject(value, path);
    return new LoadSourceRequest(
      field(object, 'source_id', path, asInteger),
      field(object, 'client', path, ProxyClientSerde.deserialize),
      field(object, 'input', path, SourceInput.deserialize),
      field(object, 'cache', path, (item, itemPath) =>
        asEnum(item, ['use', 'refresh'] as const, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      source_id: this.source_id,
      client: ProxyClientSerde.serialize(this.client),
      input: this.input.serialize(),
      cache: this.cache,
    };
  }
}

export class EvaluatePlanRequest {
  constructor(
    public plan: Plan,
    public source_profiles: SourceProfile[],
  ) {}
  static deserialize(value: unknown, path = '$'): EvaluatePlanRequest {
    const object = asObject(value, path);
    return new EvaluatePlanRequest(
      field(object, 'plan', path, Plan.deserialize),
      field(object, 'source_profiles', path, (item, itemPath) =>
        asArray(item, SourceProfile.deserialize, itemPath),
      ),
    );
  }
  serialize(): unknown {
    return {
      plan: this.plan.serialize(),
      source_profiles: this.source_profiles.map((item) => item.serialize()),
    };
  }
}

export class EvaluatePlanResponse {
  constructor(
    public plan_fingerprint: string,
    public source_profile_fingerprints: Map<number, string>,
    public report: EvaluationReport,
    public rendered_content: string | null,
  ) {}
  static deserialize(value: unknown, path = '$'): EvaluatePlanResponse {
    const object = asObject(value, path);
    const fingerprints = asObject(
      required(object, 'source_profile_fingerprints', path),
      childPath(path, 'source_profile_fingerprints'),
    );
    const rendered = object['rendered_content'];
    return new EvaluatePlanResponse(
      field(object, 'plan_fingerprint', path, asString),
      new Map(
        Object.entries(fingerprints).map(([source, fingerprint]) => [
          asInteger(Number(source), childPath(path, `source_profile_fingerprints.${source}`)),
          asString(fingerprint, childPath(path, `source_profile_fingerprints.${source}`)),
        ]),
      ),
      field(object, 'report', path, EvaluationReport.deserialize),
      rendered === null
        ? null
        : asString(required(object, 'rendered_content', path), childPath(path, 'rendered_content')),
    );
  }
  serialize(): unknown {
    return {
      plan_fingerprint: this.plan_fingerprint,
      source_profile_fingerprints: Object.fromEntries(this.source_profile_fingerprints),
      report: this.report.serialize(),
      rendered_content: this.rendered_content,
    };
  }
}

export class BuildPlanUrlRequest {
  constructor(public plan: Plan) {}
  static deserialize(value: unknown, path = '$'): BuildPlanUrlRequest {
    const object = asObject(value, path);
    return new BuildPlanUrlRequest(field(object, 'plan', path, Plan.deserialize));
  }
  serialize(): unknown {
    return { plan: this.plan.serialize() };
  }
}
export class BuildPlanUrlResponse {
  constructor(
    public url: string,
    public plan_fingerprint: string,
    public encoded_length: number,
    public warnings: string[],
  ) {}
  static deserialize(value: unknown, path = '$'): BuildPlanUrlResponse {
    const object = asObject(value, path);
    return new BuildPlanUrlResponse(
      field(object, 'url', path, asString),
      field(object, 'plan_fingerprint', path, asString),
      field(object, 'encoded_length', path, asInteger),
      field(object, 'warnings', path, (item, itemPath) => asArray(item, asString, itemPath)),
    );
  }
  serialize(): unknown {
    return {
      url: this.url,
      plan_fingerprint: this.plan_fingerprint,
      encoded_length: this.encoded_length,
      warnings: [...this.warnings],
    };
  }
}

export class DecodePlanUrlRequest {
  constructor(public url: string) {}
  static deserialize(value: unknown, path = '$'): DecodePlanUrlRequest {
    const object = asObject(value, path);
    return new DecodePlanUrlRequest(field(object, 'url', path, asString));
  }
  serialize(): unknown {
    return { url: this.url };
  }
}
export class DecodePlanUrlResponse {
  constructor(
    public plan: Plan,
    public plan_fingerprint: string,
    public encoded_length: number,
  ) {}
  static deserialize(value: unknown, path = '$'): DecodePlanUrlResponse {
    const object = asObject(value, path);
    return new DecodePlanUrlResponse(
      field(object, 'plan', path, Plan.deserialize),
      field(object, 'plan_fingerprint', path, asString),
      field(object, 'encoded_length', path, asInteger),
    );
  }
  serialize(): unknown {
    return {
      plan: this.plan.serialize(),
      plan_fingerprint: this.plan_fingerprint,
      encoded_length: this.encoded_length,
    };
  }
}

/** Business diagnostics are returned as valid preview data, not as this error. */
export class PlanBoardApiError extends Error {
  constructor(
    public readonly messages: string[],
    public readonly status?: number,
  ) {
    super(messages.join('; ') || '请求失败');
    this.name = 'PlanBoardApiError';
  }
}

export { Diagnostic };
