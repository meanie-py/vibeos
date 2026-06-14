#!/usr/bin/env bash
# SessionStart hook → register this agent in the swarm roster.
# Claude Code pipes JSON {session_id, transcript_path, cwd, ...} on stdin.
# Defensive by design: any failure exits 0 so it can never block a session.
set +e
IN="$(cat)"
PY="$(command -v python3 || command -v python)"
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
[ -z "$PY" ] && exit 0
[ -z "$DIR" ] && exit 0

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

"$PY" "$DIR/tools/swarm/swarm.py" register \
    --session "$SID" --transcript "$TRANS" --cwd "${CWD:-$DIR}" --pid "$PPID" \
    >/dev/null 2>&1

# Surface the live roster into the new session so the agent knows who else is working.
ROSTER="$("$PY" "$DIR/tools/swarm/swarm.py" list 2>/dev/null)"
if [ -n "$ROSTER" ]; then
    "$PY" - "$ROSTER" <<'EOF'
import json, sys
print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "You are registered in the vibeos swarm. Live agents:\n" + sys.argv[1]
        + "\n(Coordinate via Mattermost #swarm: `swarm read` to view, `swarm say \"...\"` to post.)",
}}))
EOF
fi
exit 0
