import { ProxyClient } from "../core/proxy-client";

/**
 * 一个本地配置目录的绑定：把"这个目录给谁用、哪个文件是什么角色"持久化下来。
 *
 * 语义角色复用 Rust/CLI 侧的 ProxyClient（surge/clash），与 confly 的 ClientConfig 概念对齐。
 * dirHandle 可结构化克隆存入 IndexedDB；重开页面后仍需 ensurePermission 重新确认读写授权。
 */
export interface SurgeBinding {
    /** 目录句柄（File System Access） */
    dirHandle: FileSystemDirectoryHandle;

    /** 目录显示名（句柄 .name） */
    dirName: string;

    /** 语义角色：这个目录是给哪个 client 用的 */
    client: ProxyClient;

    /** 主配置文件名（如 surge.conf），未指定为 null */
    mainProfile: string | null;

    /** 规则提供者文件名（如 rules.dconf），未指定为 null */
    rulesProfile: string | null;
}
