import qs from "qs";
import * as z from "zod";
import Cloneable from "../../base/cloneable";
import Equatable from "../../base/equals";
import Serializable from "../../base/serializable";
import {
    Policy,
    PolicySchema,
} from "./policy";
import {
    ProxyClient,
    ProxyClientSchema,
} from "./proxy-client";

export const ConvQuerySchema = z.object({
    server: z.string(),
    sub_url: z.string(),
    client: ProxyClientSchema,
    interval: z.number(),
    strict: z.boolean().nullable(),
    proxy_provider_name: z.string().nullable().optional(),
    policy: PolicySchema.nullable().optional(),
});

export type ConvQueryJson = z.infer<typeof ConvQuerySchema>;

export default class ConvQuery implements Cloneable<ConvQuery>, Equatable<ConvQuery>, Serializable {

    public constructor(
        // common
        public server: string,
        public sub_url: string,
        public client: ProxyClient,
        public interval: number,
        // profile
        public strict: boolean | null,
        // proxy provider
        public proxy_provider_name: string | null,
        // rule provider
        public policy: Policy | null,
    ) {
    }

    public toString(): string {
        const params: Record<string, unknown> = {
            server: this.server,
            sub_url: this.sub_url,
            client: this.client.toLowerCase(),
            interval: this.interval,
        };
        if (this.strict !== null) {
            params["strict"] = this.strict;
        }
        if (this.proxy_provider_name !== null) {
            params["proxy_provider_name"] = this.proxy_provider_name;
        }
        if (this.policy !== null) {
            params["policy"] = this.policy.serialize();
        }
        return qs.stringify(params);
    }

    // --- Cloneable / Equatable / Serializable ---

    public clone(): ConvQuery {
        return new ConvQuery(
            this.server,
            this.sub_url,
            this.client,
            this.interval,
            this.strict,
            this.proxy_provider_name,
            this.policy?.clone() ?? null,
        );
    }

    public equals(other?: ConvQuery): boolean {
        if (!other) {
            return false;
        }
        return this.server === other.server
            && this.sub_url === other.sub_url
            && this.client === other.client
            && this.interval === other.interval
            && this.strict === other.strict
            && this.proxy_provider_name === other.proxy_provider_name
            && (this.policy?.equals(other.policy ?? undefined) ?? other.policy === null);
    }

    public serialize(): any {
        return {
            server: this.server,
            sub_url: this.sub_url,
            client: this.client,
            interval: this.interval,
            strict: this.strict,
            proxy_provider_name: this.proxy_provider_name,
            policy: this.policy?.serialize() ?? null,
        };
    }

    public static fromSearch(search: string): ConvQuery {
        const raw = qs.parse(search.startsWith("?") ? search.slice(1) : search);
        const clientRaw = String(raw["client"] ?? "").toLowerCase();
        const client = Object.values(ProxyClient).find(v => v.toLowerCase() === clientRaw)
            ?? ProxyClient.Clash;
        const policyRaw = raw["policy"];
        const policy = policyRaw && typeof policyRaw === "object" && !Array.isArray(policyRaw)
                       ? Policy.deserialize(PolicySchema.parse(policyRaw))
                       : null;
        return new ConvQuery(
            String(raw["server"] ?? ""),
            String(raw["sub_url"] ?? ""),
            client,
            Number(raw["interval"] ?? 3600),
            raw["strict"] != null ? String(raw["strict"]) === "true" : null,
            raw["proxy_provider_name"] != null ? String(raw["proxy_provider_name"]) : null,
            policy,
        );
    }

    public static parse(json: unknown): ConvQueryJson {
        return ConvQuerySchema.parse(json);
    }

    public static deserialize(conv_query: ConvQueryJson): ConvQuery {
        return new ConvQuery(
            conv_query.server,
            conv_query.sub_url,
            conv_query.client,
            conv_query.interval,
            conv_query.strict,
            !!conv_query.proxy_provider_name ? conv_query.proxy_provider_name : null,
            !!conv_query.policy ? Policy.deserialize(conv_query.policy) : null,
        );
    }
}

