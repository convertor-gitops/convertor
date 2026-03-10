import { AsyncPipe } from "@angular/common";
import { Component, DestroyRef, inject } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from "@angular/forms";
import { MatButton } from "@angular/material/button";
import { MatCardContent, MatCardHeader, MatCardTitle } from "@angular/material/card";
import { MatFormField, MatLabel } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { MatOption, MatSelect } from "@angular/material/select";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { StorageMap } from "@ngx-pwa/local-storage";
import {
    catchError,
    debounceTime,
    defer,
    distinctUntilChanged,
    EMPTY,
    exhaustMap,
    filter,
    finalize,
    forkJoin,
    map,
    merge,
    Observable,
    shareReplay,
    Subject,
    switchMap,
    takeUntil,
} from "rxjs";
import { ProxyClient } from "../../../common/model/enums";
import { DashboardService } from "../../../service/dashboard.service";
import { UrlParams, UrlService } from "../../../service/url.service";
import { DashboardPanel } from "../dashboard-panel/dashboard-panel";

@Component({
    selector: "app-dashboard-param",
    imports: [
        ReactiveFormsModule,
        DashboardPanel,
        MatCardHeader,
        MatCardContent,
        MatFormField,
        MatLabel,
        MatInput,
        MatSelect,
        MatOption,
        MatSlideToggle,
        AsyncPipe,
        MatButton,
        MatCardTitle,
    ],
    templateUrl: "./dashboard-param.html",
    styleUrl: "./dashboard-param.scss",
})
export class DashboardParam {
    clients: ProxyClient[] = Object.values(ProxyClient);

    destroyRef: DestroyRef = inject(DestroyRef);
    urlService: UrlService = inject(UrlService);
    dashboardService: DashboardService = inject(DashboardService);
    storage: StorageMap = inject(StorageMap);

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
        client: new FormControl<string>(ProxyClient.Surge.toLowerCase(), {nonNullable: true}),
        strict: new FormControl<boolean>(true, {nonNullable: true}),
    });

    // urlResult = new BehaviorSubject<UrlResult | undefined>(undefined);
    // urls$: Observable<ConvertorUrl[]> = this.urlResult.pipe(
    //     map((result?: UrlResult) => {
    //         if (!result) {
    //             return [];
    //         }
    //         return [
    //             result.raw_url,
    //             result.raw_profile_url,
    //             result.profile_url,
    //             ...result.rule_providers_url,
    //         ];
    //     }),
    // );

    submit: Subject<void> = new Subject<void>();
    cancel: Subject<void> = new Subject<void>();

    params$ = this.subscriptionForm.valueChanges.pipe(
        debounceTime(300),
        map(() => this.subscriptionForm.getRawValue()),
        distinctUntilChanged(this.deepEqual),
        filter(() => this.subscriptionForm.valid),
        map(payload => this.toUrlParams(payload)),
        shareReplay({bufferSize: 1, refCount: true}),
    );

    paramRestoreSub = merge(
        this.storage.get("url").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({url: value, secret: undefined})),
        ),
        this.storage.get("secret").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({url: undefined, secret: value})),
        ),
    ).pipe(
        takeUntilDestroyed(this.destroyRef),
    ).subscribe((value) => {
        if (!value.url) {
            delete value.url;
        }
        if (!value.secret) {
            delete value.secret;
        }
        this.subscriptionForm.patchValue(value, {emitEvent: false});
    });

    paramStoreSub = merge(
        // 监听表单变化，debounce后保存
        this.subscriptionForm.valueChanges.pipe(
            debounceTime(300),
            map(() => this.subscriptionForm.getRawValue()),
        ),
        // 手动提交时立即保存
        this.submit.pipe(
            map(() => this.subscriptionForm.getRawValue()),
        ),
    ).pipe(
        switchMap(formValue => {
            const saveOperations: Observable<any>[] = [];

            // 只保存有值的字段
            if (formValue.url && formValue.url.trim()) {
                saveOperations.push(this.storage.set("url", formValue.url.trim()));
            }
            if (formValue.secret && formValue.secret.trim()) {
                saveOperations.push(this.storage.set("secret", formValue.secret.trim()));
            }

            // 如果没有要保存的字段，返回空的Observable
            if (saveOperations.length === 0) {
                return EMPTY;
            }

            return forkJoin(saveOperations).pipe(
                catchError(() => EMPTY),
            );
        }),
        takeUntilDestroyed(this.destroyRef),
    ).subscribe();

    requestSub = merge(
        this.params$,
        // 手动：点击提交时直接抓取当前 rawValue（不依赖 params$ 是否已发过值）
        this.submit.pipe(
            map(() => this.subscriptionForm.getRawValue()),
            filter(() => this.subscriptionForm.valid),
            map(payload => this.toUrlParams(payload)),
        ),
    )
        .pipe(
            exhaustMap((urlParams) => {
                return defer(() => {
                    // 请求开始：锁表单 & 开 loading
                    this.subscriptionForm.disable({emitEvent: false});

                    const query = this.urlService.buildSubscriptionQuery(urlParams);
                    return this.dashboardService.getSubscription(query).pipe(
                        // 主动取消当前请求
                        takeUntil(this.cancel),
                        catchError((err) => {
                            console.log("[DashboardParam requestSub] catchError:", err);
                            // 捕获错误，避免流中断
                            return EMPTY;
                        }),
                        // 结束（成功/失败/取消）：解锁
                        finalize(() => {
                            console.log("[DashboardParam requestSub] finalize");
                            this.subscriptionForm.enable({emitEvent: false});
                            return EMPTY;
                        }),
                    );
                });
            }),
            takeUntilDestroyed(this.destroyRef),
        )
        .subscribe();

    onSubmit() {
        this.submit.next();
    }

    onCancel() {
        this.cancel.next();
    }

    toUrlParams(payload: {
        secret: string | null,
        url: string | null,
        client: string,
        interval: number,
        strict: boolean,
    }): UrlParams {
        return {
            secret: payload.secret!,
            url: payload.url!,
            client: payload.client,
            interval: payload.interval,
            strict: payload.strict,
        };
    }

    deepEqual<T>(a: T, b: T): boolean {
        return JSON.stringify(a) === JSON.stringify(b);
    }
}
