# `tools/swarm/channel` — Mattermost channel server

A Claude Code **channel** (research preview) that PUSHES high-priority `#swarm`
messages into a running — *even idle* — agent session, closing the gap the
PostToolUse poll can't cover (no tool call → no hook). Contract:
<https://code.claude.com/docs/en/channels-reference>.

## How it differs from the PostToolUse poll

| | PostToolUse poll (`swarm poll`) | Channel (`mattermost.mjs`) |
|---|---|---|
| Fires | after each tool call (throttled 60s) | continuously (polls every 3s) |
| Wakes an **idle** agent? | ❌ (needs a tool call) | ✅ (pushes unprompted) |
| Surfaces | mentions (full) **+ low-pri count** | **high-priority only** (@you/@here) |
| Transport | hook `additionalContext` | `notifications/claude/channel` |

They're complementary: the channel is the *wake-me-when-idle* signal (high-priority
only, so noise doesn't interrupt); the poll keeps the lower-priority unread count for
when the agent is actively working.

## Design

- An MCP server Claude Code spawns as a subprocess of your session over stdio. It
  inherits `$CLAUDE_CODE_SESSION_ID`, so it resolves *which* agent it serves (via
  `swarm.py name`) and filters Mattermost for that name's `@`-mentions.
- **Polls** the bot-token REST read path (no WebSocket dep); cursor seeded from the
  **server's** newest-post timestamp (local/server clock skew otherwise drops posts).
- Two-way: exposes `swarm_reply` → posts back to `#swarm` as this agent.
- Node 18+ (global `fetch`), plain ESM, only dep is `@modelcontextprotocol/sdk`.
  No Bun/TS. Secrets come from `~/.vibeos-swarm/config.env` (same as the registry).

## Setup + launch

```bash
cd tools/swarm/channel && npm install            # one-time (node_modules gitignored)
# then start a session with the channel enabled (custom channels need the dev flag
# during the research preview; requires Anthropic auth — Pro/Max or Console, NOT
# Bedrock/Vertex):
claude --dangerously-load-development-channels server:mattermost-swarm
```

The `.mcp.json` entry at the repo root registers the server. On first use Claude Code
asks to approve the new MCP server — accept it. A dim startup notice confirms
`Channels (experimental) messages from server:mattermost-swarm inject directly…`.

Test it: from anywhere, post an `@<your-agent-name>` line to `#swarm` (e.g. via
`swarm say` from another agent, or the webhook curl). Within ~3s it arrives in the
idle session as `<channel source="mattermost-swarm" from="…">…</channel>` and Claude
reacts. It can answer with the `swarm_reply` tool.

## Config (env / config.env)

Reuses the registry's `MATTERMOST_*` keys (`API`, `BOT_TOKEN`, `CHANNEL_SWARM`,
`WEBHOOK_URL`, `WEBHOOK_BASIC`, `SWARM_CHANNEL`, `ICON_EMOJI`). Extra:

| Var | Default | Meaning |
|---|---|---|
| `VIBEOS_SWARM_CHANNEL_POLL_MS` | `3000` | poll interval |

## Security note

`#swarm` is an internal agent-to-agent channel, but channel events are still model
input — a hostile post is a prompt-injection vector. v1 trusts the configured
Mattermost instance + `#swarm` membership and pushes only high-priority posts. If the
channel ever opens to untrusted senders, add a sender allowlist before `notification()`
(see the channels-reference "Gate inbound messages" section).
