#!/usr/bin/env node
// vibeos agent-swarm — Mattermost CHANNEL server (Claude Code "channels" research preview).
//
// THIN front-end: all delivery logic (read, classify, cursor, identity) lives in the
// Python core, reached through the `swarm` CLI. This server is pure transport — it pushes
// `notifications/claude/channel` events into a running (even idle) session and exposes a
// reply tool. So the channel and the CLI are interchangeable views of ONE core/queue.
//
// LAUNCH (research preview; needs Anthropic auth — Pro/Max/Console, not Bedrock/Vertex):
//   SWARM_LOGIN=<agent> claude --dangerously-load-development-channels server:mattermost-swarm
// (identity also resolves from the machine default written by `swarm login`.)

import { Server } from '@modelcontextprotocol/sdk/server/index.js'
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js'
import { ListToolsRequestSchema, CallToolRequestSchema } from '@modelcontextprotocol/sdk/types.js'
import { execFileSync } from 'node:child_process'
import { dirname, join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))            // .../agent-swarm/channel
const CLI = join(HERE, '..', '..', 'vibeos-cli')               // repo-root vibeos-cli
const POLL_MS = Number(process.env.VIBEOS_SWARM_CHANNEL_POLL_MS || 3000)
const BATCH_MS = Number(process.env.VIBEOS_SWARM_BATCH_MS || 180000)

function swarm(args, { json = false } = {}) {
  try {
    const out = execFileSync('bash', [CLI, 'swarm', ...args], { encoding: 'utf8', timeout: 8000 })
    return json ? JSON.parse(out || 'null') : out.trim()
  } catch {
    return json ? null : ''
  }
}

// First-class tools over the swarm CLI core. These work even WITHOUT --channels (only
// the push needs the flag), so any agent loading this MCP server gets outbound + search.
const TOOLS = [
  { name: 'swarm_post', description: 'Post a message to #swarm (or a named channel) as this agent.',
    inputSchema: { type: 'object', required: ['text'], properties: {
      text: { type: 'string' }, channel: { type: 'string', description: 'channel name; default #swarm' } } },
    argv: (a) => a.channel ? ['send', String(a.text), '--channel', String(a.channel)] : ['say', String(a.text)] },
  { name: 'swarm_read', description: "Show unread messages across this agent's channels, in full.",
    inputSchema: { type: 'object', properties: {} }, argv: () => ['read'] },
  { name: 'swarm_who', description: 'Show the swarm roster (who is online, on what quest).',
    inputSchema: { type: 'object', properties: {} }, argv: () => ['who'] },
  { name: 'mm_search', description: 'Full-text search Mattermost for what this agent can see.',
    inputSchema: { type: 'object', required: ['query'], properties: { query: { type: 'string' } } },
    argv: (a) => ['search', String(a.query)] },
  { name: 'quest_list', description: 'List open quests (Vikunja tasks) by priority.',
    inputSchema: { type: 'object', properties: {} }, argv: () => ['quest', 'list'] },
  { name: 'quest_search', description: 'Search the quest board (Vikunja) by text.',
    inputSchema: { type: 'object', required: ['query'], properties: { query: { type: 'string' } } },
    argv: (a) => ['quest', 'search', String(a.query)] },
  { name: 'quest_comment', description: 'Comment on a quest.',
    inputSchema: { type: 'object', required: ['id', 'text'], properties: {
      id: { type: 'integer' }, text: { type: 'string' } } },
    argv: (a) => ['quest', 'comment', String(a.id), String(a.text)] },
  { name: 'quest_claim', description: 'Claim a quest (announces to #swarm + comments).',
    inputSchema: { type: 'object', required: ['id'], properties: { id: { type: 'integer' } } },
    argv: (a) => ['quest', 'claim', String(a.id)] },
  { name: 'quest_done', description: 'Mark a quest done.',
    inputSchema: { type: 'object', required: ['id'], properties: { id: { type: 'integer' } } },
    argv: (a) => ['quest', 'done', String(a.id)] },
]

async function main() {
  const mcp = new Server(
    { name: 'mattermost-swarm', version: '0.3.0' },
    {
      capabilities: { experimental: { 'claude/channel': {} }, tools: {} },
      instructions:
        'Events tagged <channel source="mattermost-swarm"> are chat from the vibeos swarm. ' +
        'High-priority ones (an @-mention of you, @here/@channel, or a high board priority) ' +
        'arrive immediately; lower-priority traffic arrives as a periodic count. Read and act ' +
        'on them; coordinate. Use swarm_post to reply/announce, swarm_read for the full backlog, ' +
        'mm_search / quest_search to look things up, and quest_* to work the board.',
    },
  )

  mcp.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: TOOLS.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })),
  }))

  mcp.setRequestHandler(CallToolRequestSchema, async (req) => {
    const t = TOOLS.find((x) => x.name === req.params.name)
    if (!t) throw new Error('unknown tool: ' + req.params.name)
    const out = swarm(t.argv(req.params.arguments || {}))
    return { content: [{ type: 'text', text: out || 'ok' }] }
  })

  await mcp.connect(new StdioServerTransport())

  // Poll loop: ask the core for new HIGH messages; push each. Periodically heartbeat the
  // (possibly idle) agent so the roster keeps it live.
  let ticks = 0
  let lastDigested = 0
  let lowCount = 0
  setInterval(() => {
    if (ticks++ % 10 === 0) swarm(['heartbeat'])
    const r = swarm(['drain', '--json'], { json: true })
    if (!r) return
    for (const m of r.high || []) {
      mcp.notification({
        method: 'notifications/claude/channel',
        // meta KEYS must be identifiers, but VALUES may keep hyphens etc. — only strip
        // what would break the <channel from="..."> attribute (quotes/newlines).
        params: { content: m.message, meta: { from: String(m.from).replace(/["\r\n]/g, ''), post_id: m.post_id } },
      })
    }
    lowCount = r.low_count || 0
  }, POLL_MS)

  // Batched low-priority digest: a single nudge when the lower-priority backlog has grown.
  setInterval(() => {
    if (lowCount > lastDigested) {
      mcp.notification({
        method: 'notifications/claude/channel',
        params: {
          content: `${lowCount} lower-priority #swarm update(s) — run \`swarm read\` for full text.`,
          meta: { priority: 'low', count: String(lowCount) },
        },
      })
      lastDigested = lowCount
    }
  }, BATCH_MS)
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => { process.stderr.write(`mattermost-swarm: ${e}\n`); process.exit(1) })
}
