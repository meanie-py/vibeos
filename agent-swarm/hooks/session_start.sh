#!/usr/bin/env bash
# SessionStart → login (if $SWARM_LOGIN), register in the roster, emit a situation report.
# Defensive: any failure exits 0 so it can never block a session.
set +e
IN="$(cat)"
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
SWARM="$DIR/bin/swarm"; [ -x "$SWARM" ] || SWARM="$(command -v swarm)"
[ -z "$SWARM" ] && exit 0

[ -n "$SWARM_LOGIN" ] && bash "$SWARM" login "$SWARM_LOGIN" >/dev/null 2>&1
bash "$SWARM" register --branch "$(git -C "$DIR" rev-parse --abbrev-ref HEAD 2>/dev/null)" >/dev/null 2>&1

ROSTER="$(bash "$SWARM" who 2>/dev/null)"
WHO="$(bash "$SWARM" whoami 2>/dev/null)"
if [ -n "$ROSTER" ]; then
  python3 - "$WHO" "$ROSTER" <<'EOF'
import json, sys
who, roster = sys.argv[1], sys.argv[2]
ctx = f"Swarm situation report — you are **{who}**.\nLive agents:\n{roster}\n" \
      "(Coordinate via #swarm: `swarm read` to view, `swarm say \"...\"` to post; " \
      "`swarm quest list` for the board.)"
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": ctx}}))
EOF
fi
exit 0
