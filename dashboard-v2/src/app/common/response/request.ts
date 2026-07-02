import * as z from "zod";
import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";

export class RequestBody implements Equatable<RequestBody>, Cloneable<RequestBody>, Serializable {
    constructor(
        public method: string,
        public scheme: string,
        public host: string,
        public uri: string,
        public headers: Map<string, string>,
    ) {
    }

    serialize() {
        return {
            method: this.method,
            host: this.host,
            uri: this.uri,
            headers: this.headers,
        };
    }

    clone() {
        return new RequestBody(
            this.method,
            this.scheme,
            this.host,
            this.uri,
            this.headers,
        );
    }

    equals(other?: RequestBody): boolean {
        if (other === undefined || other === null) {
            return false;
        }

        if (this === other) {
            return true;
        }

        return this.method === other.method
            && this.scheme === other.scheme
            && this.host === other.host
            && this.uri === other.uri
            && this.headers === other.headers;
    }

    public static deserialize(json: RequestBody | any): RequestBody | null {
        if (json == null) {
            return null;
        }
        let isRequestBody = Object.hasOwn(json, "method")
            && Object.hasOwn(json, "scheme")
            && Object.hasOwn(json, "host")
            && Object.hasOwn(json, "uri")
            && Object.hasOwn(json, "headers");
        if (!isRequestBody) {
            return null;
        }
        return new RequestBody(
            json.method,
            json.scheme,
            json.host,
            json.uri,
            new Map(Object.entries(json.headers)),
        );
    }

    public url(): string {
        return `${this.scheme}://${this.host}${this.uri}`;
    }
}

export const RequestBodySchema = z.object({
    method: z.string(),
    scheme: z.string(),
    host: z.string(),
    uri: z.string(),
    headers: z.record(z.string(), z.string()),
});
