"""`swarm` — command-line front-end over agent_swarm.core.

Agents are the primary users, so every command carries real help text and the output is
terse + scriptable (`--json` where useful). The CLI and the MCP server are interchangeable
front-ends over the same core modules.
"""
from __future__ import annotations

import argparse
import json
import os
import sys

from . import identity as ident
from . import queue as q
from . import state as st
from .config import config
from .errors import SwarmError
from .mattermost import MattermostClient
from .roster import roster
from .send import send as send_primitive
from .vikunja import VikunjaClient


# --- helpers -------------------------------------------------------------

def _identity(args) -> str | None:
    return getattr(args, "identity", None) or ident.current_identity()


def _require_identity(args) -> str:
    i = _identity(args)
    if not i:
        raise SwarmError("no identity — run `swarm login <name>` or set $SWARM_LOGIN")
    return i


def _mm() -> MattermostClient:
    return MattermostClient.from_config(config())


def _channels_for(identity, mm) -> list:
    """The channels this agent listens on: #swarm + its own agent channel (if present)."""
    cfg = config()
    chans = [cfg.mm_swarm_channel_id]
    try:
        ach = mm.find_channel(f"agent-{identity}")
        if ach:
            chans.append(ach)
    except SwarmError:
        pass
    return [c for c in chans if c]


# --- identity ------------------------------------------------------------

def cmd_login(args):
    ident.login(args.name, set_default=not args.no_default)
    print(f"logged in as {args.name}")


def cmd_logout(args):
    ident.logout()
    print("logged out")


def cmd_whoami(args):
    print(_identity(args) or "(no identity)")


# --- roster / presence ---------------------------------------------------

def cmd_register(args):
    identity = _require_identity(args)
    fields = {"session_id": os.environ.get("CLAUDE_CODE_SESSION_ID"),
              "pid": os.getpid()}
    for k in ("project", "quest", "branch", "lead"):
        v = getattr(args, k, None)
        if v:
            fields[k] = v
    roster().upsert(identity, **fields)
    st.set_state(identity, "busy")
    try:
        mm = _mm()
        q.seed(identity, _channels_for(identity, mm), mm)  # unread-clock starts now
    except SwarmError:
        pass
    send_primitive(f"🟢 **{identity}** online", channel=config().get("MATTERMOST_PRESENCE_CHANNEL", default="presence"),
                   as_identity=identity)
    print(f"registered {identity}")


def cmd_heartbeat(args):
    roster().heartbeat(_require_identity(args))


def cmd_offline(args):
    identity = _require_identity(args)
    roster().offline(identity)
    st.clear(identity)
    send_primitive(f"⚫ **{identity}** offline", channel=config().get("MATTERMOST_PRESENCE_CHANNEL", default="presence"),
                   as_identity=identity)
    print(f"offline {identity}")


def cmd_who(args):
    rows = roster().list(include_offline=args.all, stale_secs=args.stale_secs)
    if args.json:
        print(json.dumps(rows, indent=2, default=str)); return
    if not rows:
        print("(no agents)"); return
    icon = {"online": "🟢", "stale": "🟡", "offline": "⚫"}
    print(f"{'':2} {'IDENTITY':<20} {'PROJECT/QUEST':<24} {'AGE':>6}  MACHINE")
    for a in rows:
        age = a["age_secs"]; agestr = f"{age:.0f}s" if age < 120 else f"{age/60:.0f}m"
        pq = " / ".join(x for x in (a.get("project"), a.get("quest")) if x) or "-"
        print(f"{icon.get(a['status'],'?'):2} {(a['identity']):<20} {pq:<24} {agestr:>6}  {a.get('machine','?')}")


def cmd_state(args):
    st.set_state(_require_identity(args), args.value, pid=args.pid)


# --- messaging -----------------------------------------------------------

def cmd_send(args):
    ok = send_primitive(args.text, channel=args.channel,
                        as_identity=args.as_identity or _identity(args))
    print("sent" if ok else "send failed")


def cmd_say(args):
    identity = _identity(args)
    ok = send_primitive(args.text, channel=config().get("MATTERMOST_SWARM_CHANNEL", default="swarm"),
                        as_identity=identity)
    print("posted to #swarm" if ok else "post failed")


def cmd_read(args):
    identity = _require_identity(args)
    mm = _mm()
    posts = q.unread(identity, identity, _channels_for(identity, mm), mm)
    if not posts:
        print("no unread."); return
    print(f"{len(posts)} unread:")
    for p in posts:
        flag = "‼️ " if p["priority"] == "high" else "  "
        print(f"{flag}{str(p['user']).split(' @ ')[0]}: {p['message'].strip()}")


def cmd_drain(args):
    """For the channel server / busy-hook: new HIGH messages + low-priority count."""
    identity = _identity(args)
    if not identity:
        if args.json:
            print(json.dumps({"high": [], "low_count": 0}))
        return
    mm = _mm()
    chans = args.channels.split(",") if args.channels else _channels_for(identity, mm)
    result = q.drain(identity, identity, chans, mm, high_only=args.high_only)
    if args.json:
        print(json.dumps(result)); return
    if result["high"]:
        for m in result["high"]:
            print(f"💬 {m['from']}: {m['message'][:140]}")
    if result["low_count"]:
        print(f"({result['low_count']} other unread — `swarm read` to view)")


# --- quests (Vikunja) ----------------------------------------------------

def _vik():
    return VikunjaClient.from_config(config())


def cmd_quest_list(args):
    tasks = _vik().list_tasks(done=args.done)
    if args.json:
        print(json.dumps(tasks, indent=2, default=str)); return
    if not tasks:
        print("(no quests)"); return
    PR = {0: "-", 1: "low", 2: "med", 3: "HIGH", 4: "URGENT", 5: "DO NOW"}
    for t in tasks:
        owner = ",".join(a.get("username", "") for a in (t.get("assignees") or [])) or "-"
        done = "✓" if t.get("done") else " "
        print(f"  [{done}] #{t['id']:<4} p:{PR.get(t.get('priority',0),'?'):<7} @{owner:<12} {t['title']}")


def cmd_quest_show(args):
    print(json.dumps(_vik().get_task(args.id), indent=2, default=str))


def cmd_quest_claim(args):
    identity = _require_identity(args)
    # interim: comment the claim (per-agent Vikunja assignees pending); also @announce
    _vik().comment(args.id, f"Claimed by **{identity}**.")
    send_primitive(f"📋 **{identity}** claimed quest #{args.id}",
                   channel=config().get("MATTERMOST_SWARM_CHANNEL", default="swarm"), as_identity=identity)
    print(f"claimed #{args.id}")


def cmd_quest_done(args):
    _vik().set_done(args.id, True)
    print(f"marked #{args.id} done")


def cmd_quest_comment(args):
    _vik().comment(args.id, args.text)
    print(f"commented on #{args.id}")


# --- provisioning (registrar) -------------------------------------------

def cmd_agent_create(args):
    from .registrar import Registrar
    bundle = Registrar().register_agent(args.name, args.lead, charter=args.charter)
    print(json.dumps(bundle, indent=2, default=str))


def cmd_agent_teardown(args):
    from .registrar import Registrar
    print(json.dumps(Registrar().teardown_agent(args.name), indent=2, default=str))


def cmd_swarm_lead(args):
    from .registrar import Registrar
    print(json.dumps(Registrar().register_swarm(args.lead), indent=2, default=str))


def cmd_project_create(args):
    from .registrar import Registrar
    print(json.dumps(Registrar().create_project(args.title, description=args.desc), indent=2, default=str))


# --- parser --------------------------------------------------------------

def build_parser():
    p = argparse.ArgumentParser(
        prog="swarm",
        description="vibeos agent-swarm CLI — identity, messaging, quests, roster, provisioning.",
        epilog="Identity resolves from --identity, then $SWARM_LOGIN, then `swarm login`.")
    p.add_argument("--identity", help="override the acting identity for this command")
    sub = p.add_subparsers(dest="cmd", required=True)

    def add(name, fn, help):
        sp = sub.add_parser(name, help=help, description=help)
        sp.set_defaults(fn=fn)
        return sp

    s = add("login", cmd_login, "Bind this session to an agent identity.")
    s.add_argument("name"); s.add_argument("--no-default", action="store_true",
                                           help="don't set the machine-wide default identity")
    add("logout", cmd_logout, "Unbind this session's identity.")
    add("whoami", cmd_whoami, "Print the current identity.")

    s = add("register", cmd_register, "Register/refresh this identity in the roster (SessionStart).")
    for a in ("project", "quest", "branch", "lead"):
        s.add_argument(f"--{a}")
    add("heartbeat", cmd_heartbeat, "Bump last_seen for this identity.")
    add("offline", cmd_offline, "Mark this identity offline.")
    s = add("who", cmd_who, "Show the swarm roster.")
    s.add_argument("--json", action="store_true"); s.add_argument("--all", action="store_true")
    s.add_argument("--stale-secs", type=int, default=300)
    s = add("state", cmd_state, "Set busy/idle state (hooks).")
    s.add_argument("value", choices=["busy", "idle"]); s.add_argument("--pid", type=int)

    s = add("send", cmd_send, "Post a message to a channel as this identity.")
    s.add_argument("text"); s.add_argument("--channel", required=True)
    s.add_argument("--as", dest="as_identity")
    s = add("say", cmd_say, "Post chatter to #swarm as this identity.")
    s.add_argument("text")
    add("read", cmd_read, "Show unread messages across this agent's channels, in full.")
    s = add("drain", cmd_drain, "Emit new high-priority messages (+low count) for delivery.")
    s.add_argument("--json", action="store_true"); s.add_argument("--high-only", action="store_true")
    s.add_argument("--channels", help="comma-separated channel ids (default: #swarm + agent channel)")

    s = add("quest", lambda a: a.qfn(a), "Quests (Vikunja tasks): list/show/claim/done/comment.")
    qsub = s.add_subparsers(dest="qcmd", required=True)
    qs = qsub.add_parser("list"); qs.set_defaults(qfn=cmd_quest_list)
    qs.add_argument("--json", action="store_true"); qs.add_argument("--done", action="store_true")
    qs = qsub.add_parser("show"); qs.set_defaults(qfn=cmd_quest_show); qs.add_argument("id", type=int)
    qs = qsub.add_parser("claim"); qs.set_defaults(qfn=cmd_quest_claim); qs.add_argument("id", type=int)
    qs = qsub.add_parser("done"); qs.set_defaults(qfn=cmd_quest_done); qs.add_argument("id", type=int)
    qs = qsub.add_parser("comment"); qs.set_defaults(qfn=cmd_quest_comment)
    qs.add_argument("id", type=int); qs.add_argument("text")

    s = add("agent", lambda a: a.afn(a), "Provision/teardown an agent identity (registrar).")
    asub = s.add_subparsers(dest="acmd", required=True)
    as_ = asub.add_parser("create"); as_.set_defaults(afn=cmd_agent_create)
    as_.add_argument("name"); as_.add_argument("--lead", required=True); as_.add_argument("--charter")
    as_ = asub.add_parser("teardown"); as_.set_defaults(afn=cmd_agent_teardown); as_.add_argument("name")

    s = add("lead", cmd_swarm_lead, "Register a human lead (Authentik subgroup).")
    s.add_argument("lead")
    s = add("project", lambda a: a.pfn(a), "Provision a project/epic.")
    psub = s.add_subparsers(dest="pcmd", required=True)
    ps = psub.add_parser("create"); ps.set_defaults(pfn=cmd_project_create)
    ps.add_argument("title"); ps.add_argument("--desc", default="")
    return p


def main(argv=None):
    args = build_parser().parse_args(argv)
    try:
        args.fn(args)
        return 0
    except SwarmError as e:
        sys.stderr.write(f"swarm: {e}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
