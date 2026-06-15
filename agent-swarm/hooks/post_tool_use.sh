#!/usr/bin/env bash
# PostToolUse → busy-path delivery: heartbeat + drain new high-priority messages into
# context via additionalContext. Throttled so we don't hammer Mattermost on every tool call.
set +e
IN="$(cat)"
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
CLI="$DIR/vibeos-cli"
[ -x "$CLI" ] || exit 0
INTERVAL="${VIBEOS_SWARM_HB_INTERVAL:-30}"

SID="$(printf '%s' "$IN" | python3 -c "import sys,json;print(json.load(sys.stdin).get('session_id',''))" 2>/dev/null)"
STAMPDIR="$HOME/.vibeos-swarm/state"; mkdir -p "$STAMPDIR" 2>/dev/null
STAMP="$STAMPDIR/.hb-${SID:-default}"
NOW=$(date +%s 2>/dev/null || echo 0)
if [ -f "$STAMP" ]; then
  LAST=$(cat "$STAMP" 2>/dev/null || echo 0)
  [ $((NOW - LAST)) -lt "$INTERVAL" ] && exit 0
fi
echo "$NOW" > "$STAMP" 2>/dev/null

bash "$CLI" swarm state busy >/dev/null 2>&1
bash "$CLI" swarm heartbeat >/dev/null 2>&1
MSG="$(bash "$CLI" swarm drain 2>/dev/null)"
if [ -n "$MSG" ]; then
  python3 - "$MSG" <<'EOF'
import json, sys
print(json.dumps({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": sys.argv[1]}}))
EOF
fi
exit 0
