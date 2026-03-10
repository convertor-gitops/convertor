import { Injectable } from "@angular/core";
import ConvertorQuery from "../common/model/convertor-query";
import { Crypto_xchachaService } from "./crypto_xchacha.service";
import { EnvService } from "./env.service";


@Injectable({
    providedIn: "root",
})
export class UrlService {
    constructor(
        public crypto: Crypto_xchachaService,
        public env: EnvService,
    ) {
    }

    public buildSubscriptionQuery(params: UrlParams): ConvertorQuery {
        const {secret, url, client, interval, strict} = params;
        const sub_url = this.crypto.encrypt(secret, url);
        const server = `${window.location.protocol}//${this.env.host.value}`;
        return new ConvertorQuery(
            client,
            interval,
            strict,
            sub_url,
            server,
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
