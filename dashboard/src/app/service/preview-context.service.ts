import { computed, Injectable, signal } from "@angular/core";
import { UrlResult } from "../common/model/api/url_result";
import { SurgeBinding } from "../common/model/local/surge-binding";

export interface PreviewContext {
    binding: SurgeBinding;
    urlResult: UrlResult;
    version: string | null;
}

/**
 * 暂存"预览改动"所需的 binding + urlResult，供路由后的合并子页面读取。
 * 对象无法塞进 URL 参数，故用这个内存上下文中转；离开预览页时 clear。
 */
@Injectable({ providedIn: "root" })
export class PreviewContextService {
    private readonly ctx = signal<PreviewContext | null>(null);
    readonly current = computed(() => this.ctx());

    set(ctx: PreviewContext): void {
        this.ctx.set(ctx);
    }

    get(): PreviewContext | null {
        return this.ctx();
    }

    clear(): void {
        this.ctx.set(null);
    }
}
