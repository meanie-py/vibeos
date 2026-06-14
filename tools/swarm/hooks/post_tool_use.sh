#!/usr/bin/env bash
# PostToolUse hook → heartbeat (last_seen) + pick up a /rename if it happened.
# Throttled: only touches the DB at most once per HB_INTERVAL seconds per session,
# so it stays cheap even though PostToolUse fires after every tool call.
set +e
IN="$(cat)"
PY="$(command -v python3 || command -v python)"
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
[ -z "$PY" ] && exit 0
[ -z "$DIR" ] && exit 0
HB_INTERVAL="${VIBEOS_SWARM_HB_INTERVAL:-60}"

read -r SID TRANS CWD < <("$PY" - "$IN" <<'EOF'
import json, sys
try:
    d = json.loads(sys.argv[1])
except Exception:
    d = {}
print(d.get("session_id",""), d.get("transcript_path",""), d.get("cwd",""))
EOF
)
[ -z "$SID" ] && exit 0

DBDIR="${VIBEOS_SWARM_DB:-$HOME/.vibeos-swarm/registry.db}"; DBDIR="$(dirname "$DBDIR")"
mkdir -p "$DBDIR" 2>/dev/null
STAMP="$DBDIR/.hb-$SID"
NOW=$(date +%s 2>/dev/null || echo 0)
if [ -f "$STAMP" ]; then
    LAST=$(cat "$STAMP" 2>/dev/null || echo 0)
    [ $((NOW - LAST)) -lt "$HB_INTERVAL" ] && exit 0
fi
echo "$NOW" > "$STAMP" 2>/dev/null

"$PY" "$DIR/tools/swarm/swarm.py" heartbeat \
    --session "$SID" --transcript "$TRANS" --cwd "${CWD:-$DIR}" \
    >/dev/null 2>&1

# Poll #swarm for new chatter. Output (if any) is delta-surfaced into the agent's
# context via the PostToolUse additionalContext field. Silent when nothing's new.
MSG="$("$PY" "$DIR/tools/swarm/swarm.py" poll --session "$SID" --transcript "$TRANS" 2>/dev/null)"
if [ -n "$MSG" ]; then
    "$PY" - "$MSG" <<'EOF'
import json, sys
print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": sys.argv[1],
}}))
EOF
fi
exit 0
