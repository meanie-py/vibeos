#!/usr/bin/env bash
# SessionEnd hook → mark this agent offline in the roster.
set +e
IN="$(cat)"
PY="$(command -v python3 || command -v python)"
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
[ -z "$PY" ] && exit 0
[ -z "$DIR" ] && exit 0

read -r SID REASON < <("$PY" - "$IN" <<'EOF'
import json, sys
try:
    d = json.loads(sys.argv[1])
except Exception:
    d = {}
print(d.get("session_id",""), d.get("reason","ended"))
EOF
)
[ -z "$SID" ] && exit 0

DBDIR="${VIBEOS_SWARM_DB:-$HOME/.vibeos-swarm/registry.db}"; DBDIR="$(dirname "$DBDIR")"
rm -f "$DBDIR/.hb-$SID" 2>/dev/null

"$PY" "$DIR/tools/swarm/swarm.py" offline --session "$SID" --reason "$REASON" \
    >/dev/null 2>&1
exit 0
