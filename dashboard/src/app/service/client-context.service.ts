import { Injectable, signal } from "@angular/core";
import { ProxyClient } from "../common/model/core/proxy-client";

/**
 * 全局 client 上下文：Surge / Clash 贯穿生成、本地同步、合并渲染。
 * 顶部切换器写它，其余模块只读——保证 gen 产物与 sync 目标永远是同一个 client，
 * 不会出现把 Surge 链接 patch 进 Clash 配置这种 mismatch。
 */
@Injectable({ providedIn: "root" })
export class ClientContextService {
    readonly client = signal<ProxyClient>(ProxyClient.Surge);

    setClient(client: ProxyClient): void {
        this.client.set(client);
    }
}
