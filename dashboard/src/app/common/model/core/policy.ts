import {
  asBoolean,
  asObject,
  asString,
  childPath,
  optionalNullable,
  required,
} from '../../deserialize';
import Cloneable from '../../base/cloneable';
import Equatable from '../../base/equals';
import Serializable from '../../base/serializable';

export class Policy implements Cloneable<Policy>, Equatable<Policy>, Serializable {
  constructor(
    public name: string,
    public is_subscription: boolean,
    public option?: string,
  ) {}

  clone(): Policy {
    return Policy.deserialize(this.serialize());
  }

  equals(other?: Policy): boolean {
    return (
      other instanceof Policy &&
      this.name === other.name &&
      this.is_subscription === other.is_subscription &&
      this.option === other.option
    );
  }

  serialize(): unknown {
    return { name: this.name, is_subscription: this.is_subscription, option: this.option ?? null };
  }

  static deserialize(value: unknown, path = '$'): Policy {
    const object = asObject(value, path);
    return new Policy(
      asString(required(object, 'name', path), childPath(path, 'name')),
      asBoolean(required(object, 'is_subscription', path), childPath(path, 'is_subscription')),
      optionalNullable(object['option'], asString, childPath(path, 'option')) ?? undefined,
    );
  }
}
