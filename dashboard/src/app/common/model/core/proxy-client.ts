import { DeserializeError, asString } from '../../deserialize';

export enum ProxyClient {
  Surge = 'Surge',
  Clash = 'Clash',
}

export namespace ProxyClientSerde {
  export function deserialize(value: unknown, path = '$'): ProxyClient {
    switch (asString(value, path).trim().toLowerCase()) {
      case 'surge':
        return ProxyClient.Surge;
      case 'clash':
        return ProxyClient.Clash;
      default:
        throw new DeserializeError(path, `unknown ProxyClient ${JSON.stringify(value)}`);
    }
  }

  export function serialize(value: ProxyClient): string {
    return value.toLowerCase();
  }

  export function safeDeserialize(
    value: unknown,
  ): { success: true; data: ProxyClient } | { success: false; error: unknown } {
    try {
      return { success: true, data: deserialize(value) };
    } catch (error) {
      return { success: false, error };
    }
  }
}
