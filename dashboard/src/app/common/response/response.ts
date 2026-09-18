import { asArray, asObject, asString, childPath, required } from '../deserialize';
import { RequestBody } from './request';
import AppStatus from './status';

export type Deserializer<T> = (value: unknown, path?: string) => T;

export class ResponseBody<T = void> {
  constructor(
    public status: AppStatus,
    public messages: string[],
    public request: RequestBody | null,
    public data: T | null,
  ) {}

  isOk(): boolean {
    return this.status.isOk();
  }

  static deserialize<T>(value: unknown, deserializeData?: Deserializer<T>): ResponseBody<T> {
    const object = asObject(value);
    const rawData = object['data'];
    return new ResponseBody(
      AppStatus.deserialize(required(object, 'status'), '$.status'),
      asArray(required(object, 'messages'), asString, '$.messages'),
      RequestBody.deserialize(object['request'], '$.request'),
      rawData === null || rawData === undefined
        ? null
        : deserializeData
          ? deserializeData(rawData, childPath('$', 'data'))
          : (rawData as T),
    );
  }
}
