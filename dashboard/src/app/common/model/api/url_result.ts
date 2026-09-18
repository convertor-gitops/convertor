import { asArray, asObject, childPath, required } from '../../deserialize';
import Cloneable from '../../base/cloneable';
import Equatable from '../../base/equals';
import Serializable from '../../base/serializable';
import { ConvUrl } from '../core/conv-url';

export class UrlResult implements Cloneable<UrlResult>, Equatable<UrlResult>, Serializable {
  public constructor(
    public original_url: ConvUrl,
    public raw_url: ConvUrl,
    public profile_url: ConvUrl,
    public proxy_provider_urls: ConvUrl[],
    public rule_provider_urls: ConvUrl[],
  ) {}

  public clone(): UrlResult {
    return new UrlResult(
      this.original_url.clone(),
      this.raw_url.clone(),
      this.profile_url.clone(),
      this.proxy_provider_urls.map((u) => u.clone()),
      this.rule_provider_urls.map((u) => u.clone()),
    );
  }

  public equals(other?: UrlResult): boolean {
    if (!other) {
      return false;
    }
    return (
      this.original_url.equals(other.original_url) &&
      this.raw_url.equals(other.raw_url) &&
      this.profile_url.equals(other.profile_url) &&
      this.proxy_provider_urls.length === other.proxy_provider_urls.length &&
      this.proxy_provider_urls.every((u, i) => u.equals(other.proxy_provider_urls[i])) &&
      this.rule_provider_urls.length === other.rule_provider_urls.length &&
      this.rule_provider_urls.every((u, i) => u.equals(other.rule_provider_urls[i]))
    );
  }

  public serialize(): any {
    return {
      original_url: this.original_url.serialize(),
      raw_url: this.raw_url.serialize(),
      profile_url: this.profile_url.serialize(),
      proxy_provider_urls: this.proxy_provider_urls.map((u) => u.serialize()),
      rule_provider_urls: this.rule_provider_urls.map((u) => u.serialize()),
    };
  }

  public static deserialize(value: unknown, path = '$'): UrlResult {
    const json = asObject(value, path);
    return new UrlResult(
      ConvUrl.deserialize(required(json, 'original_url', path), childPath(path, 'original_url')),
      ConvUrl.deserialize(required(json, 'raw_url', path), childPath(path, 'raw_url')),
      ConvUrl.deserialize(required(json, 'profile_url', path), childPath(path, 'profile_url')),
      asArray(
        required(json, 'proxy_provider_urls', path),
        ConvUrl.deserialize,
        childPath(path, 'proxy_provider_urls'),
      ),
      asArray(
        required(json, 'rule_provider_urls', path),
        ConvUrl.deserialize,
        childPath(path, 'rule_provider_urls'),
      ),
    );
  }
}
