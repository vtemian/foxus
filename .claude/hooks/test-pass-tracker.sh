#!/usr/bin/env bash
set -euo pipefail

# PostToolUse hook: track whether tests have passed in this session.
# Creates /tmp/foxus-tests-passed-<session_id> on test success.
# Removes it on test failure.

input=$(cat)

tool_name=$(echo "$input" | jq -r '.tool_name // empty')
if [[ "$tool_name" != "Bash" ]]; then
  exit 0
fi

command=$(echo "$input" | jq -r '.tool_input.command // empty')
if [[ -z "$command" ]]; then
  exit 0
fi

# Match test commands
if ! echo "$command" | grep -qE '(cargo test|npm test|vitest run|make test|make check|make test-rust|make test-frontend)'; then
  exit 0
fi

session_id=$(echo "$input" | jq -r '.session_id // "default"')
flag_file="/tmp/foxus-tests-passed-${session_id}"

# Check tool_result for failure indicators
tool_result=$(echo "$input" | jq -r '.tool_result // empty')

# FAILED (uppercase) = cargo test failure. FAIL (uppercase) = vitest failure.
# "0 failed" in success output is lowercase, so case-sensitive match avoids false positives.
if echo "$tool_result" | grep -qE '(FAILED|FAIL |failures:|error\[|panicked|exit code [1-9])'; then
  rm -f "$flag_file"
else
  touch "$flag_file"
fi

exit 0
