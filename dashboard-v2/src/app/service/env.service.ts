import { isPlatformBrowser } from "@angular/common";
import { DOCUMENT, Inject, Injectable, PLATFORM_ID } from "@angular/core";
import { BehaviorSubject } from "rxjs";

@Injectable({ providedIn: "root" })
export class EnvService {
    public host: BehaviorSubject<string>;
    public userAgent: BehaviorSubject<string>;

    constructor(
        @Inject(DOCUMENT) private document: Document,
        @Inject(PLATFORM_ID) private platformId: Object,
    ) {
        this.host = new BehaviorSubject<string>(this.getHost() ?? "");
        this.userAgent = new BehaviorSubject<string>(this.getUserAgent() ?? "");
    }

    getHost(): string | null {
        return isPlatformBrowser(this.platformId) ? this.document.location.host : null;
    }

    getUserAgent(): string | null {
        return isPlatformBrowser(this.platformId) ? navigator.userAgent : null;
    }
}
