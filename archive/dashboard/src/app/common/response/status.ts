import * as z from "zod";
import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";

export default class AppStatus implements Equatable<AppStatus>, Cloneable<AppStatus>, Serializable {

    constructor(
        public readonly code: number,
        public readonly status: string,
    ) {
    }

    public isOk(): boolean {
        return this.code === 0;
    }

    public isError(): boolean {
        return this.code === -1;
    }

    public is(code: number): boolean {
        return this.code === code;
    }

    serialize() {
        return {
            code: this.code,
            status: this.status,
        };
    }

    clone(): AppStatus {
        return new AppStatus(this.code, this.status.toString());
    }

    equals(other?: AppStatus | undefined): boolean {
        if (other === undefined) {
            return false;
        }

        if (this === other) {
            return true;
        }

        return this.code === other.code
            && this.status === other.status;
    }

    public static deserialize(json: unknown): AppStatus {
        const status = AppStatusSchema.parse(json);

        return new AppStatus(status.code, status.status);
    }

}

export const AppStatusSchema = z.object({
    code: z.number(),
    status: z.string().trim(),
});
