// 补充 lib.dom 尚未稳定纳入的 File System Access API 提案部分。
// 仅声明本项目用到的入口；其余 FileSystem*Handle 类型已在 lib.dom 中。

interface Window {
    showDirectoryPicker(options?: {
        id?: string;
        mode?: "read" | "readwrite";
        startIn?: FileSystemHandle | "desktop" | "documents" | "downloads" | "music" | "pictures" | "videos";
    }): Promise<FileSystemDirectoryHandle>;
}

interface FileSystemHandle {
    queryPermission(descriptor?: { mode?: "read" | "readwrite" }): Promise<PermissionState>;

    requestPermission(descriptor?: { mode?: "read" | "readwrite" }): Promise<PermissionState>;
}
