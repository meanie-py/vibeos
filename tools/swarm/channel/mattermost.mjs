#!/usr/bin/env node
// vibeos swarm — Mattermost CHANNEL server (Claude Code "channels" research preview).
//
// A channel is an MCP server Claude Code spawns as a subprocess of YOUR session and
// communicates with over stdio. It can PUSH events into the running (even idle) session
// so Claude reacts to chat that arrives while it's not taking tool calls — the gap the
// PostToolUse poll can't cover. See docs/engineering/swarm-comms-plan.md + the registry
// README. Contract: https://code.claude.com/docs/en/channels-reference
//
// WHAT IT DOES
//   - Spawned per-session → inherits $CLAUDE_CODE_SESSION_ID → knows which agent it is.
//   - Polls the #swarm Mattermost channel (reuses the bot-token REST read path).
//   - Pushes ONLY high-priority posts (@<thisAgent> / @here / @channel / @all) as
//     `notifications/claude/channel` events. Low-priority chatter stays with the
//     PostToolUse poll's unread count — we don't wake an idle agent for noise.
//   - Two-way: exposes `swarm_reply` so Claude can post back to #swarm.
//
// RUNTIME: Node 18+ (global fetch). No Bun/TS needed. Name resolution shells out to
// swarm.py (Node 20 has no built-in sqlite); everything else is stdlib.
//
// LAUNCH (research preview — custom channels aren't allowlisted):
//   claude --dangerously-load-development-channels server:mattermost-swarm
// (requires the .mcp.json entry at repo root + `npm install` in this dir.)

import { Server } from '@modelcontextprotocol/sdk/server/index.js'
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js'
import { ListToolsRequestSchema, CallToolRequestSchema } from '@modelcontextprotocol/sdk/types.js'
import { readFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { homedir } from 'node:os'
import { join, dirname } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))      // .../tools/swarm/channel
const SWARM_PY = join(HERE, '..', 'swarm.py')
const POLL_MS = Number(process.env.VIBEOS_SWARM_CHANNEL_POLL_MS || 3000)

export function loadConfig() {
  const p = process.env.VIBEOS_SWARM_CONFIG || join(homedir(), '.vibeos-swarm', 'config.env')
  const cfg = {}
  try {
    for (const line of readFileSync(p, 'utf8').split('\n')) {
      const t = line.trim()
      if (!t || t.startsWith('#') || !t.includes('=')) continue
      const i = t.indexOf('=')
      cfg[t.slice(0, i).trim()] = t.slice(i + 1).trim().replace(/^['"]|['"]$/g, '')
    }
  } catch { /* config optional */ }
  for (const k of Object.keys(process.env)) if (k.startsWith('MATTERMOST_')) cfg[k] = process.env[k]
  return cfg
}

export function resolveName() {
  // --by-cwd: resolve by worktree, NOT session id. An MCP server subprocess gets its
  // own CLAUDE_CODE_SESSION_ID (≠ the agent's), so session lookup finds the wrong agent.
  try {
    return execFileSync('python3', [SWARM_PY, 'name', '--by-cwd'],
      { encoding: 'utf8', timeout: 5000, cwd: process.cwd() }).trim() || null
  } catch { return null }
}

export function isHighPriority(msg, name) {
  const m = (msg || '').toLowerCase()
  if (m.includes('@here') || m.includes('@channel') || m.includes('@all')) return true
  if (name && m.includes('@' + name.toLowerCase())) return true
  return false
}

export async function mmRead(cfg, since) {
  const { MATTERMOST_API: api, MATTERMOST_BOT_TOKEN: tok, MATTERMOST_CHANNEL_SWARM: ch } = cfg
  if (!api || !tok || !ch) return []
  try {
    const r = await fetch(`${api}/channels/${ch}/posts?since=${since}`,
      { headers: { Authorization: `Bearer ${tok}` } })
    if (!r.ok) return []
    const d = await r.json()
    const out = []
    for (const id of (d.order || []).slice().reverse()) {   // API is newest-first
      const p = d.posts[id]
      if (!p || p.create_at <= since || p.type) continue     // skip old + system msgs
      out.push({
        id, create_at: p.create_at,
        user: (p.props && p.props.override_username) || p.user_id || '?',
        message: p.message || '',
      })
    }
    return out
  } catch { return [] }
}

export async function mmLatestCreateAt(cfg) {
  // Newest post's create_at on the SERVER clock — seed the cursor with this, not
  // Date.now(), because local/server clock skew otherwise drops recent posts.
  const { MATTERMOST_API: api, MATTERMOST_BOT_TOKEN: tok, MATTERMOST_CHANNEL_SWARM: ch } = cfg
  if (!api || !tok || !ch) return null
  try {
    const r = await fetch(`${api}/channels/${ch}/posts?per_page=1`,
      { headers: { Authorization: `Bearer ${tok}` } })
    if (!r.ok) return null
    const d = await r.json()
    const id = (d.order || [])[0]
    return id ? d.posts[id].create_at : null
  } catch { return null }
}

export async function mmPost(cfg, text, username) {
  const url = cfg.MATTERMOST_WEBHOOK_URL
  if (!url) return false
  const headers = { 'Content-Type': 'application/json' }
  if (cfg.MATTERMOST_WEBHOOK_BASIC)
    headers.Authorization = 'Basic ' + Buffer.from(cfg.MATTERMOST_WEBHOOK_BASIC).toString('base64')
  const payload = { text, channel: cfg.MATTERMOST_SWARM_CHANNEL || 'swarm' }
  if (username) payload.username = username
  if (cfg.MATTERMOST_ICON_EMOJI) payload.icon_emoji = cfg.MATTERMOST_ICON_EMOJI
  try {
    const r = await fetch(url, { method: 'POST', headers, body: JSON.stringify(payload) })
    return r.ok
  } catch { return false }
}

async function main() {
  const cfg = loadConfig()
  let myName = resolveName()

  const mcp = new Server(
    { name: 'mattermost-swarm', version: '0.1.0' },
    {
      capabilities: { experimental: { 'claude/channel': {} }, tools: {} },
      instructions:
        'Events tagged <channel source="mattermost-swarm"> are HIGH-PRIORITY chat from ' +
        'other agents in the vibeos #swarm Mattermost channel — they @-mention you or use ' +
        '@here/@channel. Read and act on them; coordinate. To reply in #swarm, call the ' +
        'swarm_reply tool. For the full backlog including lower-priority messages, run ' +
        '`swarm read` in a Bash tool.',
    },
  )

  mcp.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: [{
      name: 'swarm_reply',
      description: 'Post a message to the vibeos #swarm Mattermost channel as this agent.',
      inputSchema: {
        type: 'object',
        properties: { text: { type: 'string', description: 'message to post to #swarm' } },
        required: ['text'],
      },
    }],
  }))

  mcp.setRequestHandler(CallToolRequestSchema, async (req) => {
    if (req.params.name === 'swarm_reply') {
      const text = String(req.params.arguments?.text ?? '')
      const ok = await mmPost(cfg, text, myName || undefined)
      return { content: [{ type: 'text', text: ok ? 'posted to #swarm' : 'post failed' }] }
    }
    throw new Error('unknown tool: ' + req.params.name)
  })

  await mcp.connect(new StdioServerTransport())

  // Poll loop. Seed the cursor from the server's newest post (skew-immune) so we only
  // push messages that arrive after the server is up. Refresh name to catch /rename.
  let cursor = (await mmLatestCreateAt(cfg)) ?? Date.now()
  let ticks = 0
  setInterval(async () => {
    if (ticks++ % 10 === 0) myName = resolveName() ?? myName
    const posts = await mmRead(cfg, cursor)
    for (const p of posts) {
      cursor = Math.max(cursor, p.create_at)
      const fromName = String(p.user).split(' @ ')[0]
      if (myName && fromName === myName) continue          // never wake on own post
      if (!isHighPriority(p.message, myName)) continue      // channel = high-priority only
      await mcp.notification({
        method: 'notifications/claude/channel',
        params: {
          content: p.message,
          meta: { from: String(p.user).replace(/[^A-Za-z0-9_]/g, '_'), post_id: p.id },
        },
      })
    }
  }, POLL_MS)
}

// Run only when invoked directly (so tests can import the helpers above).
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => { process.stderr.write(`mattermost-swarm: ${e}\n`); process.exit(1) })
}
