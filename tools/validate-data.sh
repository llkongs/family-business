#!/usr/bin/env bash
# validate-data.sh — Pre-deployment data integrity check (T023/T024)
# Ensures all static data files are self-contained and don't reference Feishu APIs.
# Exit code 0 = pass, non-zero = fail (blocks deployment).

set -euo pipefail

ERRORS=0
WARNINGS=0

error() { echo "ERROR: $1"; ERRORS=$((ERRORS + 1)); }
warn()  { echo "WARN:  $1"; WARNINGS=$((WARNINGS + 1)); }
ok()    { echo "OK:    $1"; }

echo "=== Data Integrity Check ==="
echo ""

# ---- 1. Feishu temporary URL interception ----
echo "--- Checking for Feishu temporary URLs ---"

FEISHU_PATTERNS=(
    "open.feishu.cn/open-apis/drive"
    "internal-api-drive-stream.feishu.cn"
    "open.feishu.cn/open-apis/auth"
)

# mockData.ts is used directly by the frontend — Feishu URLs here are ERRORS
for pattern in "${FEISHU_PATTERNS[@]}"; do
    if grep -q "$pattern" src/data/mockData.ts 2>/dev/null; then
        error "Feishu URL found in mockData.ts: '$pattern' — frontend will try to load this!"
    fi
done

# productDatabase.json is not imported by frontend — Feishu URLs here are WARNINGS
for pattern in "${FEISHU_PATTERNS[@]}"; do
    if grep -q "$pattern" src/data/productDatabase.json 2>/dev/null; then
        warn "Feishu URL found in productDatabase.json: '$pattern' (not used by frontend, but should be fixed)"
    fi
done

if [ $ERRORS -eq 0 ]; then
    ok "No Feishu temporary URLs in frontend data (mockData.ts)"
fi

echo ""

# ---- 2. mockData.ts structural checks ----
echo "--- Checking mockData.ts structure ---"

MOCK_FILE="src/data/mockData.ts"

if [ ! -f "$MOCK_FILE" ]; then
    error "mockData.ts not found"
else
    # Check required exports exist
    for export_name in "storeInfo" "mediaPlaylist" "categories" "products" "slogans"; do
        if grep -q "export const $export_name" "$MOCK_FILE"; then
            ok "Export '$export_name' found"
        else
            warn "Export '$export_name' not found in mockData.ts"
        fi
    done

    # Check storeInfo has real data (not empty strings)
    if grep -q "name: ''" "$MOCK_FILE"; then
        error "storeInfo.name is empty"
    fi

    if grep -q "phone: ''" "$MOCK_FILE"; then
        error "storeInfo.phone is empty"
    fi
fi

echo ""

# ---- 3. Build artifacts check (if dist/ exists) ----
echo "--- Checking build artifacts ---"

if [ -d "dist" ]; then
    if [ -f "dist/index.html" ]; then
        ok "dist/index.html exists"
    else
        error "dist/index.html missing after build"
    fi

    # Check for Feishu URLs in built JS
    if grep -r "open.feishu.cn" dist/ 2>/dev/null; then
        error "Feishu URLs found in build artifacts (dist/)"
    else
        ok "No Feishu URLs in build artifacts"
    fi
else
    warn "dist/ not found (run after build step)"
fi

echo ""

# ---- 4. Static resource checks ----
echo "--- Checking static resources ---"

# QR code should exist
if [ -f "public/images/qrcode.jpg" ]; then
    ok "QR code image exists"
else
    warn "public/images/qrcode.jpg missing (QR code won't display)"
fi

# At least one video HLS manifest should exist
VIDEO_COUNT=$(find public/videos -name "index.m3u8" 2>/dev/null | wc -l | tr -d ' ')
if [ "$VIDEO_COUNT" -gt 0 ]; then
    ok "Found $VIDEO_COUNT HLS video manifests"
else
    warn "No HLS video files found in public/videos/"
fi

echo ""

# ---- Summary ----
echo "=== Summary ==="
echo "Errors:   $ERRORS"
echo "Warnings: $WARNINGS"

if [ $ERRORS -gt 0 ]; then
    echo ""
    echo "FAILED: $ERRORS error(s) found. Deployment blocked."
    exit 1
fi

if [ $WARNINGS -gt 0 ]; then
    echo ""
    echo "PASSED with warnings. Deployment allowed."
fi

echo ""
echo "PASSED: All critical checks OK."
exit 0
