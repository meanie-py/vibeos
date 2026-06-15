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
const SWARM = join(HERE, '..', '..', 'bin', 'swarm')           // the CLI wrapper
const POLL_MS = Number(process.env.VIBEOS_SWARM_CHANNEL_POLL_MS || 3000)
const BATCH_MS = Number(process.env.VIBEOS_SWARM_BATCH_MS || 180000)

function swarm(args, { json = false } = {}) {
  try {
    const out = execFileSync('bash', [SWARM, ...args], { encoding: 'utf8', timeout: 8000 })
    return json ? JSON.parse(out || 'null') : out.trim()
  } catch {
    return json ? null : ''
  }
}

async function main() {
  const mcp = new Server(
    { name: 'mattermost-swarm', version: '0.2.0' },
    {
      capabilities: { experimental: { 'claude/channel': {} }, tools: {} },
      instructions:
        'Events tagged <channel source="mattermost-swarm"> are chat from the vibeos swarm. ' +
        'High-priority ones (an @-mention of you, @here/@channel, or a high board priority) ' +
        'arrive immediately; lower-priority traffic arrives as a periodic count. Read and act ' +
        'on them; coordinate. To reply, call swarm_reply. For the full backlog run `swarm read`.',
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
      const out = swarm(['say', String(req.params.arguments?.text ?? '')])
      return { content: [{ type: 'text', text: out || 'posted' }] }
    }
    throw new Error('unknown tool: ' + req.params.name)
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
