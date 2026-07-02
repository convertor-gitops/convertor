import { Injectable } from "@angular/core";
import { SurgeBinding } from "../common/model/local/surge-binding";

const DB_NAME = "convertor-local";
const DB_VERSION = 1;
const STORE = "bindings";
const BINDING_KEY = "surge";

/**
 * 浏览器原生 File System Access 封装：选目录、读写文件、权限确认，
 * 以及把目录绑定（含句柄）持久化到 IndexedDB。
 *
 * 仅 Chromium 系 + secure context（https / localhost）可用；用 supported 做特性检测。
 */
@Injectable({ providedIn: "root" })
export class LocalFsService {
    /** 浏览器是否支持 File System Access API */
    get supported(): boolean {
        return typeof window !== "undefined" && typeof window.showDirectoryPicker === "function";
    }

    /** 弹出系统目录选择框，返回目录句柄 */
    async pickDirectory(): Promise<FileSystemDirectoryHandle> {
        return await window.showDirectoryPicker({ id: "surge-config", mode: "readwrite" });
    }

    /** 列出目录下的文件名（不含子目录），按名排序 */
    async listFiles(dir: FileSystemDirectoryHandle): Promise<string[]> {
        const names: string[] = [];
        for await (const [name, handle] of dir.entries()) {
            if (handle.kind === "file") {
                names.push(name);
            }
        }
        return names.sort((a, b) => a.localeCompare(b));
    }

    async readFile(dir: FileSystemDirectoryHandle, name: string): Promise<string> {
        const fileHandle = await dir.getFileHandle(name);
        const file = await fileHandle.getFile();
        return await file.text();
    }

    async writeFile(dir: FileSystemDirectoryHandle, name: string, content: string): Promise<void> {
        const fileHandle = await dir.getFileHandle(name, { create: true });
        const writable = await fileHandle.createWritable();
        await writable.write(content);
        await writable.close();
    }

    /** 确认对句柄有读写权限（首次或权限过期时会弹系统请求） */
    async ensurePermission(handle: FileSystemHandle, mode: "read" | "readwrite" = "readwrite"): Promise<boolean> {
        if ((await handle.queryPermission({ mode })) === "granted") {
            return true;
        }
        return (await handle.requestPermission({ mode })) === "granted";
    }

    // ===== 绑定持久化（IndexedDB；FileSystemDirectoryHandle 可结构化克隆）=====

    async saveBinding(binding: SurgeBinding): Promise<void> {
        const db = await this.openDb();
        try {
            await this.run(db, "readwrite", store => store.put(binding, BINDING_KEY));
        } finally {
            db.close();
        }
    }

    async loadBinding(): Promise<SurgeBinding | null> {
        const db = await this.openDb();
        try {
            const result = await this.run<SurgeBinding | undefined>(db, "readonly", store => store.get(BINDING_KEY));
            return result ?? null;
        } finally {
            db.close();
        }
    }

    async clearBinding(): Promise<void> {
        const db = await this.openDb();
        try {
            await this.run(db, "readwrite", store => store.delete(BINDING_KEY));
        } finally {
            db.close();
        }
    }

    private openDb(): Promise<IDBDatabase> {
        return new Promise((resolve, reject) => {
            const req = indexedDB.open(DB_NAME, DB_VERSION);
            req.onupgradeneeded = () => {
                if (!req.result.objectStoreNames.contains(STORE)) {
                    req.result.createObjectStore(STORE);
                }
            };
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => reject(req.error);
        });
    }

    private run<T>(db: IDBDatabase, mode: IDBTransactionMode, op: (store: IDBObjectStore) => IDBRequest): Promise<T> {
        return new Promise((resolve, reject) => {
            const transaction = db.transaction(STORE, mode);
            const request = op(transaction.objectStore(STORE));
            request.onsuccess = () => resolve(request.result as T);
            request.onerror = () => reject(request.error);
        });
    }
}
