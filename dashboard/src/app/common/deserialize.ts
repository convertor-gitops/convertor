/** An error raised while mapping untrusted JSON into a domain class. */
export class DeserializeError extends Error {
  constructor(
    public readonly path: string,
    message: string,
  ) {
    super(`${path}: ${message}`);
    this.name = 'DeserializeError';
  }
}

export type JsonObject = Record<string, unknown>;

export function childPath(path: string, key: string | number): string {
  return typeof key === 'number' ? `${path}[${key}]` : `${path}.${key}`;
}

export function asObject(value: unknown, path = '$'): JsonObject {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new DeserializeError(path, 'expected an object');
  }
  return value as JsonObject;
}

export function required(object: JsonObject, key: string, path = '$'): unknown {
  if (!Object.hasOwn(object, key)) {
    throw new DeserializeError(childPath(path, key), 'required field is missing');
  }
  return object[key];
}

export function asString(value: unknown, path = '$'): string {
  if (typeof value !== 'string') {
    throw new DeserializeError(path, 'expected a string');
  }
  return value;
}

export function asBoolean(value: unknown, path = '$'): boolean {
  if (typeof value !== 'boolean') {
    throw new DeserializeError(path, 'expected a boolean');
  }
  return value;
}

export function asNumber(value: unknown, path = '$'): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new DeserializeError(path, 'expected a finite number');
  }
  return value;
}

export function asInteger(value: unknown, path = '$'): number {
  const number = asNumber(value, path);
  if (!Number.isSafeInteger(number)) {
    throw new DeserializeError(path, 'expected a safe integer');
  }
  return number;
}

export function asArray<T>(
  value: unknown,
  deserialize: (item: unknown, path: string) => T,
  path = '$',
): T[] {
  if (!Array.isArray(value)) {
    throw new DeserializeError(path, 'expected an array');
  }
  return value.map((item, index) => deserialize(item, childPath(path, index)));
}

export function asStringArray(value: unknown, path = '$'): string[] {
  return asArray(value, asString, path);
}

export function optional<T>(
  value: unknown,
  deserialize: (item: unknown, path: string) => T,
  path = '$',
): T | undefined {
  return value === undefined ? undefined : deserialize(value, path);
}

export function nullable<T>(
  value: unknown,
  deserialize: (item: unknown, path: string) => T,
  path = '$',
): T | null {
  return value === null ? null : deserialize(value, path);
}

export function optionalNullable<T>(
  value: unknown,
  deserialize: (item: unknown, path: string) => T,
  path = '$',
): T | null | undefined {
  return value === undefined ? undefined : nullable(value, deserialize, path);
}

export function asEnum<T extends string>(value: unknown, values: readonly T[], path = '$'): T {
  const string = asString(value, path);
  if (!values.includes(string as T)) {
    throw new DeserializeError(path, `unknown enum variant ${JSON.stringify(string)}`);
  }
  return string as T;
}

export function asRecord(value: unknown, path = '$'): Record<string, unknown> {
  return { ...asObject(value, path) };
}

export function serializeOptional<T>(
  value: T | undefined,
  serialize: (item: T) => unknown = (item) => item,
): unknown {
  return value === undefined ? undefined : serialize(value);
}

export function compactObject(entries: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(entries).filter(([, value]) => value !== undefined));
}
