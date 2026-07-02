import { provideHttpClient, withFetch, withInterceptors } from "@angular/common/http";
import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from "@angular/core";
import { provideRouter } from "@angular/router";
import { captureExchangeInterceptor } from "./common/http/capture-exchange.interceptor";
import { routes } from "./app.routes";

export const appConfig: ApplicationConfig = {
    providers: [
        provideBrowserGlobalErrorListeners(),
        provideZonelessChangeDetection(),
        provideRouter(routes),
        provideHttpClient(
            withFetch(),
            withInterceptors([captureExchangeInterceptor]),
        ),
    ],
};
