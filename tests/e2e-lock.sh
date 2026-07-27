#!/usr/bin/env bash
# tests/e2e-lock.sh — 锁定功能端到端测试
# 用法: bash tests/e2e-lock.sh
set -euo pipefail

BASE="http://localhost:3001"
PASS=0
FAIL=0

log()  { echo "  [LOG] $*"; }
pass() { echo "  [PASS] $*"; ((PASS++)); }
fail() { echo "  [FAIL] $*"; ((FAIL++)); }

echo "=== 锁定功能端到端测试 ==="
echo ""

echo "1. 获取管理序列码"
SEQ=$(curl -sf "$BASE/api/admin/sequence" | grep -o '\[[0-9,]*\]')
log "序列码: $SEQ"
echo ""

echo "2. 创建管理员会话"
TOKEN=$(curl -sf -X POST "$BASE/api/admin/session")
log "Token: ${TOKEN:0:20}..."
echo ""

echo "3. 查看文档 1 当前状态"
DOC=$(curl -sf "$BASE/api/cats/1")
log "文档: $DOC"
DEPRECATED=$(echo "$DOC" | grep -o '"deprecated":[^,}]*' | cut -d: -f2)
if [ "$DEPRECATED" = "true" ]; then
    log "文档已弃用，先恢复..."
    curl -sf -X PUT "$BASE/api/cats/1/deprecated" \
        -H "Content-Type: application/json" \
        -H "X-Session-Token: $TOKEN" \
        -d '{"deprecated":false}' > /dev/null
fi
echo ""

echo "4. 锁定文档"
LOCK_RES=$(curl -sf -X PUT "$BASE/api/cats/1/lock" \
    -H "Content-Type: application/json" \
    -H "X-Session-Token: $TOKEN" \
    -d '{"locked":true}')
log "锁定结果: $LOCK_RES"
echo ""

echo "5. 验证锁定状态（管理员）"
DOC=$(curl -sf "$BASE/api/cats/1")
LOCKED=$(echo "$DOC" | grep -o '"locked":[^,}]*' | cut -d: -f2)
log "locked = $LOCKED"
if [ "$LOCKED" = "true" ]; then
    pass "管理员确认: 文档已锁定"
else
    fail "管理员确认: locked 应为 true，实际 $LOCKED"
fi
echo ""

echo "6. 退出管理"
curl -sf -X DELETE "$BASE/api/admin/session" \
    -H "X-Session-Token: $TOKEN" > /dev/null
log "会话已销毁"
echo ""

echo "7. 验证锁定状态（访客）"
DOC=$(curl -sf "$BASE/api/cats/1")
LOCKED=$(echo "$DOC" | grep -o '"locked":[^,}]*' | cut -d: -f2)
DEPRECATED=$(echo "$DOC" | grep -o '"deprecated":[^,}]*' | cut -d: -f2)
log "locked = $LOCKED, deprecated = $DEPRECATED"
if [ "$LOCKED" = "true" ]; then
    pass "访客确认: 文档已锁定"
else
    fail "访客确认: locked 应为 true，实际 $LOCKED"
fi
echo ""

echo "8. 清理：解锁 + 恢复"
TOKEN=$(curl -sf -X POST "$BASE/api/admin/session")
curl -sf -X PUT "$BASE/api/cats/1/lock" \
    -H "Content-Type: application/json" \
    -H "X-Session-Token: $TOKEN" \
    -d '{"locked":false}' > /dev/null
curl -sf -X PUT "$BASE/api/cats/1/deprecated" \
    -H "Content-Type: application/json" \
    -H "X-Session-Token: $TOKEN" \
    -d '{"deprecated":false}' > /dev/null
log "清理完成"
echo ""

echo "=== 结果: $PASS 通过, $FAIL 失败 ==="
if [ "$FAIL" -gt 0 ]; then exit 1; fi
