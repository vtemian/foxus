#!/usr/bin/env bash
set -euo pipefail

# PostToolUse hook: auto-format files after Write/Edit/MultiEdit.
# Routes by extension: .ts/.tsx/.js/.jsx/.json/.css -> biome, .rs -> rustfmt.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIOME="${PROJECT_ROOT}/node_modules/.bin/biome"
BIOME_WORKING_DIR="${PROJECT_ROOT}/src/web"

input=$(cat)

tool_name=$(echo "$input" | jq -r '.tool_name // empty')

# Extract file path based on tool type
case "$tool_name" in
  Write|Edit)
    file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')
    ;;
  MultiEdit)
    file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')
    ;;
  *)
    exit 0
    ;;
esac

if [[ -z "$file_path" || ! -f "$file_path" ]]; then
  exit 0
fi

# Only format files within our project
if [[ "$file_path" != "${PROJECT_ROOT}"/* ]]; then
  exit 0
fi

extension="${file_path##*.}"

case "$extension" in
  ts|tsx|js|jsx|json|css)
    if [[ -x "$BIOME" ]]; then
      (cd "$BIOME_WORKING_DIR" && "$BIOME" format --write "$file_path") 2>/dev/null || true
    fi
    ;;
  rs)
    if command -v rustfmt &>/dev/null; then
      rustfmt "$file_path" 2>/dev/null || true
    fi
    ;;
esac

exit 0
