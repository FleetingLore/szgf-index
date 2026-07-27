/**
 * 调试日志工具。
 * 由服务端 DUCIA_DEBUG=true 环境变量控制。
 * 终端启动：DUCIA_DEBUG=true ./ducia-server
 */

function isDebug(): boolean {
    if (typeof window === "undefined") return false;
    return (window as any).__DUCIA_DEBUG === true;
}

export function debugLog(...args: unknown[]) {
    if (isDebug()) {
        console.log("[debug]", ...args);
    }
}
