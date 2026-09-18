import qs from 'qs';
import {
  asBoolean,
  asInteger,
  asObject,
  asString,
  childPath,
  optionalNullable,
  required,
} from '../../deserialize';
import Cloneable from '../../base/cloneable';
import Equatable from '../../base/equals';
import Serializable from '../../base/serializable';
import { Policy } from './policy';
import { ProxyClient, ProxyClientSerde } from './proxy-client';

export default class ConvQuery implements Cloneable<ConvQuery>, Equatable<ConvQuery>, Serializable {
  public constructor(
    // common
    public server: string,
    public sub_url: string,
    public client: ProxyClient,
    public interval: number,
    // profile
    public strict: boolean | null,
    // proxy provider
    public proxy_provider_name: string | null,
    // rule provider
    public policy: Policy | null,
  ) {}

  public toString(): string {
    const params: Record<string, unknown> = {
      server: this.server,
      sub_url: this.sub_url,
      client: this.client.toLowerCase(),
      interval: this.interval,
    };
    if (this.strict !== null) {
      params['strict'] = this.strict;
    }
    if (this.proxy_provider_name !== null) {
      params['proxy_provider_name'] = this.proxy_provider_name;
    }
    if (this.policy !== null) {
      params['policy'] = this.policy.serialize();
    }
    return qs.stringify(params);
  }

  // --- Cloneable / Equatable / Serializable ---

  public clone(): ConvQuery {
    return new ConvQuery(
      this.server,
      this.sub_url,
      this.client,
      this.interval,
      this.strict,
      this.proxy_provider_name,
      this.policy?.clone() ?? null,
    );
  }

  public equals(other?: ConvQuery): boolean {
    if (!other) {
      return false;
    }
    return (
      this.server === other.server &&
      this.sub_url === other.sub_url &&
      this.client === other.client &&
      this.interval === other.interval &&
      this.strict === other.strict &&
      this.proxy_provider_name === other.proxy_provider_name &&
      (this.policy?.equals(other.policy ?? undefined) ?? other.policy === null)
    );
  }

  public serialize(): any {
    return {
      server: this.server,
      sub_url: this.sub_url,
      client: ProxyClientSerde.serialize(this.client),
      interval: this.interval,
      strict: this.strict,
      proxy_provider_name: this.proxy_provider_name,
      policy: this.policy?.serialize() ?? null,
    };
  }

  public static fromSearch(search: string): ConvQuery {
    const raw = qs.parse(search.startsWith('?') ? search.slice(1) : search);
    const clientRaw = String(raw['client'] ?? '').toLowerCase();
    const client =
      Object.values(ProxyClient).find((v) => v.toLowerCase() === clientRaw) ?? ProxyClient.Clash;
    const policyRaw = raw['policy'];
    const policy =
      policyRaw && typeof policyRaw === 'object' && !Array.isArray(policyRaw)
        ? Policy.deserialize(policyRaw)
        : null;
    return new ConvQuery(
      String(raw['server'] ?? ''),
      String(raw['sub_url'] ?? ''),
      client,
      Number(raw['interval'] ?? 3600),
      raw['strict'] != null ? String(raw['strict']) === 'true' : null,
      raw['proxy_provider_name'] != null ? String(raw['proxy_provider_name']) : null,
      policy,
    );
  }

  public static deserialize(value: unknown, path = '$'): ConvQuery {
    const conv_query = asObject(value, path);
    return new ConvQuery(
      asString(required(conv_query, 'server', path), childPath(path, 'server')),
      asString(required(conv_query, 'sub_url', path), childPath(path, 'sub_url')),
      ProxyClientSerde.deserialize(required(conv_query, 'client', path), childPath(path, 'client')),
      asInteger(required(conv_query, 'interval', path), childPath(path, 'interval')),
      optionalNullable(conv_query['strict'], asBoolean, childPath(path, 'strict')) ?? null,
      optionalNullable(
        conv_query['proxy_provider_name'],
        asString,
        childPath(path, 'proxy_provider_name'),
      ) ?? null,
      optionalNullable(conv_query['policy'], Policy.deserialize, childPath(path, 'policy')) ?? null,
    );
  }
}
