import { Injectable } from "@angular/core";
import ConvQuery from "../common/model/core/conv-query";
import { ProxyClientSerde } from "../common/model/core/proxy-client";
import { EncryptorService } from "./encrypt.service";
import { EnvService } from "./env.service";


@Injectable({
    providedIn: "root",
})
export class UrlService {
    constructor(
        public crypto: EncryptorService,
        public env: EnvService,
    ) {
    }

    public buildSubscriptionQuery(params: UrlParams): ConvQuery {
        const { secret, url, client, interval, strict } = params;
        const sub_url = this.crypto.encrypt(secret, url);
        const server = `${window.location.protocol}//${this.env.host.value}`;
        const proxyClient = ProxyClientSerde.deserialize(client);
        return new ConvQuery(
            server,
            sub_url,
            proxyClient,
            interval,
            strict,
            null,
            null,
        );
    }
}

export interface UrlParams {
    secret: string;

    url: string;

    client: string;

    interval: number;

    strict: boolean;
}
