import { asObject, asString, childPath, required } from '../deserialize';
import Cloneable from '../base/cloneable';
import Equatable from '../base/equals';
import Serializable from '../base/serializable';

export class RequestBody implements Equatable<RequestBody>, Cloneable<RequestBody>, Serializable {
  constructor(
    public method: string,
    public scheme: string,
    public host: string,
    public uri: string,
    public headers: Map<string, string>,
  ) {}

  serialize(): unknown {
    return {
      method: this.method,
      scheme: this.scheme,
      host: this.host,
      uri: this.uri,
      headers: Object.fromEntries(this.headers),
    };
  }

  clone(): RequestBody {
    return new RequestBody(this.method, this.scheme, this.host, this.uri, new Map(this.headers));
  }

  equals(other?: RequestBody): boolean {
    return (
      other instanceof RequestBody &&
      JSON.stringify(this.serialize()) === JSON.stringify(other.serialize())
    );
  }

  static deserialize(value: unknown, path = '$'): RequestBody | null {
    if (value === null || value === undefined) {
      return null;
    }
    const object = asObject(value, path);
    const headersObject = asObject(required(object, 'headers', path), childPath(path, 'headers'));
    return new RequestBody(
      asString(required(object, 'method', path), childPath(path, 'method')),
      asString(required(object, 'scheme', path), childPath(path, 'scheme')),
      asString(required(object, 'host', path), childPath(path, 'host')),
      asString(required(object, 'uri', path), childPath(path, 'uri')),
      new Map(
        Object.entries(headersObject).map(([key, header]) => [
          key,
          asString(header, childPath(path, `headers.${key}`)),
        ]),
      ),
    );
  }

  url(): string {
    return `${this.scheme}://${this.host}${this.uri}`;
  }
}
