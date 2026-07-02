import { AsyncPipe } from "@angular/common";
import {
    ChangeDetectionStrategy,
    Component,
    inject,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
    FormControl,
    FormGroup,
    ReactiveFormsModule,
    Validators,
} from "@angular/forms";
import { StorageMap } from "@ngx-pwa/local-storage";
import {
    catchError,
    debounceTime,
    defer,
    EMPTY,
    exhaustMap,
    filter,
    finalize,
    forkJoin,
    map,
    merge,
    Observable,
    Subject,
    switchMap,
    takeUntil,
} from "rxjs";
import { ProxyClient } from "../../../common/model/core/proxy-client";
import { ClientContextService } from "../../../service/client-context.service";
import { DashboardService } from "../../../service/dashboard.service";
import {
    UrlParams,
    UrlService,
} from "../../../service/url.service";
import { WorkbenchSection } from "../../shared/workbench-section/workbench-section";

@Component({
    selector: "app-dashboard-param",
    imports: [
        ReactiveFormsModule,
        AsyncPipe,
        WorkbenchSection,
    ],
    templateUrl: "./dashboard-param.html",
    styleUrl: "./dashboard-param.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardParam {
    private clientContext = inject(ClientContextService);
    urlService: UrlService = inject(UrlService);
    dashboardService: DashboardService = inject(DashboardService);
    storage: StorageMap = inject(StorageMap);
    readonly client = this.clientContext.client;
    readonly clients = Object.values(ProxyClient);

    subscriptionForm = new FormGroup({
        secret: new FormControl<string | null>(null, {
            validators: [Validators.required],
            updateOn: "blur",
        }),
        url: new FormControl<string | null>(null, {
            validators: [Validators.required],
            updateOn: "blur",
        }),
        interval: new FormControl<number>(43200, {
            nonNullable: true,
            validators: [Validators.required],
            updateOn: "blur",
        }),
        strict: new FormControl<boolean>(true, { nonNullable: true }),
    });

    submit: Subject<void> = new Subject<void>();
    cancel: Subject<void> = new Subject<void>();

    paramRestore$ = merge(
        this.storage.get("url").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({ url: value, secret: undefined })),
        ),
        this.storage.get("secret").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({ url: undefined, secret: value })),
        ),
    );

    paramStore$ = merge(
        this.subscriptionForm.valueChanges.pipe(
            debounceTime(300),
            map(() => this.subscriptionForm.getRawValue()),
        ),
        this.submit.pipe(
            map(() => this.subscriptionForm.getRawValue()),
        ),
    ).pipe(
        switchMap(formValue => {
            const saveOperations: Observable<any>[] = [];

            if (formValue.url && formValue.url.trim()) {
                saveOperations.push(this.storage.set("url", formValue.url.trim()));
            }
            if (formValue.secret && formValue.secret.trim()) {
                saveOperations.push(this.storage.set("secret", formValue.secret.trim()));
            }

            if (saveOperations.length === 0) {
                return EMPTY;
            }

            return forkJoin(saveOperations).pipe(
                catchError(() => EMPTY),
            );
        }),
    );

    request$ = this.submit.pipe(
        map(() => this.subscriptionForm.getRawValue()),
        filter(() => this.subscriptionForm.valid),
        map(payload => this.toUrlParams(payload)),
        exhaustMap((urlParams) => defer(() => {
            this.subscriptionForm.disable({ emitEvent: false });

            const query = this.urlService.buildSubscriptionQuery(urlParams);
            return this.dashboardService.getSubscription(query).pipe(
                takeUntil(this.cancel),
                catchError((_err) => {
                    return EMPTY;
                }),
                finalize(() => {
                    this.subscriptionForm.enable({ emitEvent: false });
                }),
            );
        })),
    );

    public constructor() {
        this.paramRestore$.pipe(
            takeUntilDestroyed(),
        ).subscribe((value) => {
            if (!value.url) {
                delete value.url;
            }
            if (!value.secret) {
                delete value.secret;
            }
            this.subscriptionForm.patchValue(value, { emitEvent: false });
        });

        this.paramStore$.pipe(
            takeUntilDestroyed(),
        ).subscribe();

        this.request$.pipe(
            takeUntilDestroyed(),
        ).subscribe();
    }

    onSubmit() {
        this.submit.next();
    }

    onCancel() {
        this.cancel.next();
    }

    selectClient(client: ProxyClient): void {
        this.clientContext.setClient(client);
    }

    toggleStrict(): void {
        const control = this.subscriptionForm.controls.strict;
        control.setValue(!control.value);
    }

    toUrlParams(payload: {
        secret: string | null,
        url: string | null,
        interval: number,
        strict: boolean,
    }): UrlParams {
        return {
            secret: payload.secret!,
            url: payload.url!,
            client: this.clientContext.client(),
            interval: payload.interval,
            strict: payload.strict,
        };
    }

    deepEqual<T>(a: T, b: T): boolean {
        return JSON.stringify(a) === JSON.stringify(b);
    }
}
