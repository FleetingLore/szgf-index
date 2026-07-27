//! ducia-wasm — 编译为 WebAssembly 的共享逻辑
//!
//! 用途：
//! - 前端 Markdown 渲染（与后端使用相同的 pulldown-cmark 引擎）
//! - 序列码验证（与后端共享同一套验证逻辑）
//! - 文件格式检测
//!
//! 构建：`wasm-pack build --target web`

use pulldown_cmark::{Parser, html};
use wasm_bindgen::prelude::*;

/// 将 Markdown 文本渲染为 HTML
///
/// 与后端渲染引擎相同，保证前后端输出一致。
#[wasm_bindgen]
pub fn markdown_to_html(markdown_input: &str) -> String {
    let parser = Parser::new(markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// 检查用户输入序列是否匹配目标序列
///
/// - `input`: 用户点击的数字序列
/// - `expected`: 配置文件中的目标序列
///
/// 返回 `true` 当 input 的末尾与 expected 完全匹配。
#[wasm_bindgen]
pub fn check_sequence(input: Vec<usize>, expected: Vec<usize>) -> bool {
    if expected.is_empty() || input.len() < expected.len() {
        return false;
    }

    let offset = input.len() - expected.len();
    input[offset..] == expected[..]
}

/// 检查序列前缀匹配（用户还在输入中）
#[wasm_bindgen]
pub fn check_sequence_prefix(input: Vec<usize>, expected: Vec<usize>) -> bool {
    if expected.is_empty() {
        return true;
    }
    if input.len() > expected.len() {
        return false;
    }
    expected.starts_with(&input)
}

/// 从文件名检测文档格式
#[wasm_bindgen]
pub fn detect_format(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "md" | "markdown" => "markdown".into(),
        "txt" => "plaintext".into(),
        "html" | "htm" => "html".into(),
        _ => "markdown".into(),
    }
}
