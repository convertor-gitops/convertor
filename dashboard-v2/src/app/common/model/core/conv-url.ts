import * as z from "zod";
import Cloneable from "../../base/cloneable";
import Equatable from "../../base/equals";
import Serializable from "../../base/serializable";
import ConvQuery, { ConvQuerySchema } from "./conv-query";

export enum UrlTypeVariant {
    Original = "Original",
    Raw = "Raw",
    Profile = "Profile",
    ProxyProvider = "ProxyProvider",
    RuleProvider = "RuleProvider",
}

const UrlTypeVariantSchema = z.enum(UrlTypeVariant, {
    error: (issue) => `Unknown UrlType: ${String(issue.input)}`,
});

export const ConvUrlSchema = z.object({
    type: UrlTypeVariantSchema,
    server: z.string(),
    query: ConvQuerySchema.nullable().optional(),
});

export type ConvUrlJson = z.infer<typeof ConvUrlSchema>;

export class ConvUrl implements Cloneable<ConvUrl>, Equatable<ConvUrl>, Serializable {
    private constructor(
        public readonly type: UrlType,
        public readonly server: string,
        public readonly query: ConvQuery | null,
    ) {
    }

    // --- Static constructors ---

    public static new(type: UrlType, server: string, query: ConvQuery | null): ConvUrl {
        return new ConvUrl(type, server, query);
    }

    public static empty(): ConvUrl {
        return new ConvUrl(UrlType.Original, "http://example.com", null);
    }

    public static original(server: string): ConvUrl {
        return new ConvUrl(UrlType.Original, server, null);
    }

    /** FromStr: parse a full URL string into ConvUrl */
    public static fromStr(s: string): ConvUrl {
        const url = new URL(s);
        const type = UrlType.fromPath(url.pathname);
        let query: ConvQuery | null = null;
        if (type !== UrlType.Original && url.search) {
            query = ConvQuery.fromSearch(url.search);
        }
        const server = `${url.protocol}//${url.host}`;
        return new ConvUrl(type, server, query);
    }

    // --- Instance methods ---

    /** path_and_query */
    public pathAndQuery(): string {
        const path = this.type.path();
        if (this.query === null) {
            return path;
        }
        return `${path}?${this.query.toString()}`;
    }

    /** takeQuery: consumes the query (returns it, leaving null conceptually) */
    public takeQuery(): ConvQuery {
        if (this.query === null) {
            throw new Error("MissingConvQuery");
        }
        return this.query;
    }

    /** TryFrom<ConvUrl> for URL */
    public toUrl(): URL {
        const url = new URL(this.server);
        if (this.type === UrlType.Original) {
            return url;
        }
        url.pathname = this.type.path();
        if (this.query !== null) {
            url.search = this.query.toString();
        }
        return url;
    }

    /** Display */
    public toString(): string {
        return this.toUrl().toString();
    }

    public desc(): string {
        return this.type.label;
    }

    // --- Cloneable / Equatable / Serializable ---

    public clone(): ConvUrl {
        return new ConvUrl(this.type.clone(), this.server, this.query);
    }

    public equals(other?: ConvUrl): boolean {
        if (!other) {
            return false;
        }
        return this.type.equals(other.type) && this.server === other.server;
    }

    public serialize(): any {
        return {
            type: this.type.serialize(),
            server: this.server,
            query: this.query ?? null,
        };
    }

    public static parse(json: unknown): ConvUrlJson {
        return ConvUrlSchema.parse(json);
    }

    public static deserialize(conv_url: ConvUrlJson): ConvUrl {
        return new ConvUrl(
            UrlType.deserialize(conv_url.type),
            conv_url.server,
            !!conv_url.query ? ConvQuery.deserialize(conv_url.query) : null,
        );
    }
}

export class UrlType implements Cloneable<UrlType>, Equatable<UrlType>, Serializable {
    public static readonly PREFIX = "/subscription";

    public static readonly Original = new UrlType(UrlTypeVariant.Original, "/original", "订阅商原始订阅配置", "original");
    public static readonly Raw = new UrlType(UrlTypeVariant.Raw, "/raw", "转换前订阅配置", "raw_profile");
    public static readonly Profile = new UrlType(UrlTypeVariant.Profile, "/profile", "转换后订阅配置", "profile");
    public static readonly ProxyProvider = new UrlType(UrlTypeVariant.ProxyProvider, "/proxy-provider", "代理提供者", "proxy_provider");
    public static readonly RuleProvider = new UrlType(UrlTypeVariant.RuleProvider, "/rule-provider", "规则提供者", "rule_provider");

    private constructor(
        public readonly variant: UrlTypeVariant,
        public readonly subPath: string,
        public readonly label: string,
        private readonly displayName: string,
    ) {
    }

    public static prefix(): string {
        return UrlType.PREFIX;
    }

    public path(): string {
        return UrlType.PREFIX + this.subPath;
    }

    public static variants(): UrlType[] {
        return [
            UrlType.Original,
            UrlType.Raw,
            UrlType.Profile,
            UrlType.ProxyProvider,
            UrlType.RuleProvider,
        ];
    }

    public static fromPath(path: string): UrlType {
        return UrlType.variants().find(v => v.path() === path) ?? UrlType.Original;
    }

    public toString(): string {
        return this.displayName;
    }

    // --- Cloneable / Equatable / Serializable ---

    public clone(): UrlType {
        return this; // 不可变单例，直接返回自身
    }

    public equals(other?: UrlType): boolean {
        if (!other) {
            return false;
        }
        return this.variant === other.variant;
    }

    public serialize(): any {
        return this.variant;
    }

    public static deserialize(value: UrlTypeVariant): UrlType {
        const type = UrlType.variants().find(v => v.variant === value);
        if (type === undefined) {
            throw new Error(`Unknown UrlType: ${String(value)}`);
        }
        return type;
    }
}

