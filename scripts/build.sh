#!/usr/bin/env bash
# 프론트엔드를 빌드한 뒤, 그 결과물을 내장한 릴리스 바이너리를 만든다.
set -euo pipefail
cd "$(dirname "$0")/.."
(cd web && npm ci --no-audit --no-fund && npm run build)
cargo build --release
echo "→ target/release/kitchen"
