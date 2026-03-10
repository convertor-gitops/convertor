import { RequestBody } from "./request";

export class ResponseBody<T = void> {
    constructor(
        public status: string,
        public messages: string[],
        public request: RequestBody | null,
        public data: T | null,
    ) {
    }

    public static deserialize<T>(json: ResponseBody<T> | any, ctor?: {
        new(...args: any[]): T;
        deserialize(json: T): T;
    }): ResponseBody<T> | null {
        if (json == null) {
            return null;
        }
        if (!Object.hasOwn(json, "status") || !Object.hasOwn(json, "messages")) {
            return null;
        }
        return new ResponseBody<T>(
            json.status,
            json.messages,
            RequestBody.deserialize(json.request),
            ctor?.deserialize(json.data) ?? json.data,
        );
    }

    public isOk(): boolean {
        return this.status === "ok";
    }
}
