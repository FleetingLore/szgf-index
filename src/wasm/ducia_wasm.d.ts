/* tslint:disable */
/* eslint-disable */

/**
 * 检查用户输入序列是否匹配目标序列
 *
 * - `input`: 用户点击的数字序列
 * - `expected`: 配置文件中的目标序列
 *
 * 返回 `true` 当 input 的末尾与 expected 完全匹配。
 */
export function check_sequence(input: Uint32Array, expected: Uint32Array): boolean;

/**
 * 检查序列前缀匹配（用户还在输入中）
 */
export function check_sequence_prefix(input: Uint32Array, expected: Uint32Array): boolean;

/**
 * 从文件名检测文档格式
 */
export function detect_format(filename: string): string;

/**
 * 将 Markdown 文本渲染为 HTML
 *
 * 与后端渲染引擎相同，保证前后端输出一致。
 */
export function markdown_to_html(markdown_input: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly check_sequence: (a: number, b: number, c: number, d: number) => number;
    readonly check_sequence_prefix: (a: number, b: number, c: number, d: number) => number;
    readonly detect_format: (a: number, b: number) => [number, number];
    readonly markdown_to_html: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
