#!/usr/bin/env bash
set -euo pipefail

# PreToolUse hook: block git commit if tests haven't passed.
# Checks for /tmp/foxus-tests-passed-<session_id> created by test-pass-tracker.sh.

input=$(cat)

tool_name=$(echo "$input" | jq -r '.tool_name // empty')
if [[ "$tool_name" != "Bash" ]]; then
  exit 0
fi

command=$(echo "$input" | jq -r '.tool_input.command // empty')
if [[ -z "$command" ]]; then
  exit 0
fi

# Only gate git commit commands
if ! echo "$command" | grep -qE '^git commit|&& git commit|; git commit'; then
  exit 0
fi

session_id=$(echo "$input" | jq -r '.session_id // "default"')
flag_file="/tmp/foxus-tests-passed-${session_id}"

if [[ ! -f "$flag_file" ]]; then
  cat >&2 <<'EOF'
{
  "hookSpecificOutput": {
    "permissionDecision": "deny"
  },
  "systemMessage": "BLOCKED: Tests have not passed in this session. Run `make test` or `make check` before committing. The commit-gate hook requires evidence that tests pass before allowing commits."
}
EOF
  exit 2
fi

exit 0
