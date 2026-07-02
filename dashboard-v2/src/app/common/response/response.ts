import * as z from "zod";
import {
    RequestBody,
    RequestBodySchema,
} from "./request";
import AppStatus, { AppStatusSchema } from "./status";

export const ResponseBodyScheme = z.object({
    status: AppStatusSchema,
    messages: z.array(z.string()),
    request: RequestBodySchema.nullable().optional(),
    data: z.unknown().nullable().optional(),
});

export type ResponseBodyJson = z.infer<typeof ResponseBodyScheme>;

export class ResponseBody<T = void> {
    constructor(
        public status: AppStatus,
        public messages: string[],
        public request: RequestBody | null,
        public data: T | null,
    ) {
    }

    public isOk(): boolean {
        return this.status.isOk();
    }

    public static deserialize<T>(json: ResponseBodyJson, ctor?: {
        new(...args: any[]): T;
        parse(json: unknown): unknown;
        deserialize(json: unknown): T;
    }): ResponseBody<T> {

        let data: T | null = null;

        if (!!json.data) {
            if (!!ctor) {
                data = ctor.deserialize(ctor.parse(json.data!));
            } else {
                data = json.data as T;
            }
        }

        return new ResponseBody<T>(
            AppStatus.deserialize(json.status),
            json.messages,
            RequestBody.deserialize(json.request),
            data,
        );
    }
}
