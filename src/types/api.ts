import type { DocListItem, DocFull } from "./doc";

/** API 统一响应包裹 */
export interface ApiResponse<T = unknown> {
    success: boolean;
    data?: T;
    message?: string;
    siteName?: string;
}

/** 列表接口返回 */
export interface ListCatsResponse extends ApiResponse<DocListItem[]> {
    siteName: string;
}

/** 单文档接口返回 */
export interface GetCatResponse extends ApiResponse<DocFull> {}

/** 上传文档接口返回 */
export interface UploadCatResponse extends ApiResponse<DocListItem> {}

/** 管理序列接口返回 */
export interface SequenceResponse {
    sequence: number[];
}

/** Session 创建返回 */
export interface CreateSessionResponse {
    token: string;
}

/** Session 检查返回 */
export interface CheckSessionResponse {
    isAdmin: boolean;
}

/** 通用成功标记 */
export interface SuccessResponse {
    success: boolean;
}
