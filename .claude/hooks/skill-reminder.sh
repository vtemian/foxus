#!/usr/bin/env bash
set -euo pipefail

# UserPromptSubmit hook: inject skill activation reminder on every prompt.
# Keeps skill awareness high even after context compaction.

cat <<'EOF'
{
  "systemMessage": "SKILL CHECK: Before acting, verify if a skill applies. Key triggers: creating features -> brainstorming + writing-plans; writing code -> test-driven-development; debugging -> systematic-debugging; claiming done -> verification-before-completion. If a skill matches, use the Skill tool BEFORE proceeding."
}
EOF

exit 0
