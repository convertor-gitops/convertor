import { asInteger, asObject, asString, childPath, required } from '../deserialize';
import Cloneable from '../base/cloneable';
import Equatable from '../base/equals';
import Serializable from '../base/serializable';

export default class AppStatus implements Equatable<AppStatus>, Cloneable<AppStatus>, Serializable {
  constructor(
    public readonly code: number,
    public readonly status: string,
  ) {}

  isOk(): boolean {
    return this.code === 0;
  }

  isError(): boolean {
    return this.code === -1;
  }

  is(code: number): boolean {
    return this.code === code;
  }

  serialize(): unknown {
    return { code: this.code, status: this.status };
  }

  clone(): AppStatus {
    return new AppStatus(this.code, this.status);
  }

  equals(other?: AppStatus): boolean {
    return other instanceof AppStatus && this.code === other.code && this.status === other.status;
  }

  static deserialize(value: unknown, path = '$'): AppStatus {
    const object = asObject(value, path);
    return new AppStatus(
      asInteger(required(object, 'code', path), childPath(path, 'code')),
      asString(required(object, 'status', path), childPath(path, 'status')).trim(),
    );
  }
}
