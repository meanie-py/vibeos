"""agent_swarm — the vibeos agent-swarm platform core.

A core library with thin, interchangeable CLI (`agent_swarm.cli`) and MCP (`channel/`)
front-ends. Modules:
  config      — merged configuration + alias-aware accessors
  http        — stdlib HTTP client (no third-party deps)
  mattermost  — messaging + reads + (registrar) provisioning
  vikunja     — task source of truth (epics/quests/subtasks)
  authentik   — identity (agents = M2M service accounts)
  identity    — login + identity resolution
  roster      — who's in the swarm (SQLite cache; Go plugin later)
  state       — busy/idle state machine with PID-liveness
  queue       — pending-message queue + priority classification (one delivery path)
  send        — the shared `send` primitive
  registrar   — provisioning fan-out across the three services
"""

__version__ = "0.1.0"
