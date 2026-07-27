/** 文档元数据，对应后端 DocMeta */
export interface DocMeta {
    /** 文档标题 */
    title: string;
    /** 存储文件名（相对 docs 目录） */
    file: string;
    /** 创建时间戳（ms since epoch） */
    created_at: number;
    /** 是否已弃用 */
    deprecated: boolean;
    /** 是否已软删除 */
    deleted: boolean;
}

/** 文档列表项（API 返回的精简视图） */
export interface DocListItem {
    id: string;
    title: string;
    created_at: number;
}

/** 文档完整数据（包含 Markdown 内容） */
export interface DocFull extends DocListItem {
    file: string;
    content: string;
    deprecated: boolean;
    locked: boolean;
}

/** 创建文档的请求载荷 */
export interface CreateDocReq {
    title: string;
    content: string;
}

/** 弃用标记请求载荷 */
export interface DeprecatedReq {
    deprecated: boolean;
}

/** 文件格式枚举 */
export type DocFormat = "markdown" | "plaintext" | "html";

/** 从扩展名推断格式 */
export function detectFormat(filename: string): DocFormat {
    const ext = filename.split(".").pop()?.toLowerCase();
    switch (ext) {
        case "md":
        case "markdown":
            return "markdown";
        case "txt":
            return "plaintext";
        case "html":
        case "htm":
            return "html";
        default:
            return "markdown";
    }
}
