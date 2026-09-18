import {
  asArray,
  asBoolean,
  asObject,
  asString,
  childPath,
  optional,
  required,
} from '../../deserialize';

export class ServiceStatus {
  constructor(
    public readonly name: string,
    public readonly healthy: boolean,
    public readonly message?: string,
  ) {}

  static deserialize(value: unknown, path = '$'): ServiceStatus {
    const object = asObject(value, path);
    return new ServiceStatus(
      asString(required(object, 'name', path), childPath(path, 'name')),
      asBoolean(required(object, 'healthy', path), childPath(path, 'healthy')),
      optional(object['message'], asString, childPath(path, 'message')),
    );
  }

  serialize(): unknown {
    return { name: this.name, healthy: this.healthy, message: this.message };
  }
}

export class BackendStatus {
  constructor(
    public readonly version: string,
    public readonly services: ServiceStatus[],
  ) {}

  get healthy(): boolean {
    return this.services.every((service) => service.healthy);
  }

  static deserialize(value: unknown, path = '$'): BackendStatus {
    const object = asObject(value, path);
    return new BackendStatus(
      asString(required(object, 'version', path), childPath(path, 'version')),
      asArray(
        required(object, 'services', path),
        ServiceStatus.deserialize,
        childPath(path, 'services'),
      ),
    );
  }

  serialize(): unknown {
    return { version: this.version, services: this.services.map((service) => service.serialize()) };
  }
}
