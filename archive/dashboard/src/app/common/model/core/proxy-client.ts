import * as z from "zod";

export enum ProxyClient {
    Surge = "Surge",
    Clash = "Clash",
}

export const ProxyClientSchema = z.preprocess(
    (input) => {
        if (typeof input !== "string") {
            return input;
        }

        switch (input.trim().toLowerCase()) {
            case "surge":
                return ProxyClient.Surge;
            case "clash":
                return ProxyClient.Clash;
            default:
                return input;
        }
    },
    z.enum(ProxyClient, {
        error: (issue) => `Unknown ProxyClient: ${String(issue.input)}`,
    }),
);

export namespace ProxyClientSerde {
    export function deserialize(json: unknown): ProxyClient {
        return ProxyClientSchema.parse(json);
    }

    export function safeDeserialize(json: unknown) {
        return ProxyClientSchema.safeParse(json);
    }
}
