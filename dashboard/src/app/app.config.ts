import { provideHttpClient, withFetch, withInterceptors } from "@angular/common/http";
import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from "@angular/core";
import { captureExchangeInterceptor } from "./common/http/capture-exchange.interceptor";

export const appConfig: ApplicationConfig = {
    providers: [
        provideBrowserGlobalErrorListeners(),
        provideZonelessChangeDetection(),
        // provideRouter(routes),
        provideHttpClient(
            withFetch(),
            withInterceptors([captureExchangeInterceptor]),
        ),
    ],
};
