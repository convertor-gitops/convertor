export default class ConvertorQuery {

    public static API_SUBSCRIPTION = "api/build-url";

    public constructor(
        public client: string,
        public interval: number,
        public strict: boolean,
        public sub_url: string,
        public server: string,
    ) {
    }

    public subscriptionPath(): string {
        return `/${ConvertorQuery.API_SUBSCRIPTION}?${this}`;
    }

    public toString(): string {
        const params = new URLSearchParams();
        params.set("client", this.client);
        params.set("interval", this.interval.toString());
        params.set("strict", this.strict ? "true" : "false");
        params.set("sub_url", this.sub_url);
        params.set("server", this.server);
        return params.toString()
    }
}
