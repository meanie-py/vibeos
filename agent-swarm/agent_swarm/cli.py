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
from .types import DrainResult, Priority, VikunjaPriority
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


def _mm_as(identity) -> MattermostClient:
    """A client that READS as the agent (its token sees #swarm + its own private channel;
    the shared bot token can't see the agent's channel)."""
    mm = MattermostClient.from_config(config())
    tok = (ident.creds_for(identity) or {}).get("mm_token") if identity else None
    if tok:
        mm.bot_token = tok
    return mm


def _self_ids(identity):
    creds = ident.creds_for(identity) or {}
    return [identity, creds.get("mm_user_id")]


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
    ident.login(args.name, creds=ident.load_agent_creds(args.name), set_default=not args.no_default)
    print(f"logged in as {args.name}")


def cmd_logout(args):
    ident.logout()
    print("logged out")


def cmd_whoami(args):
    print(_identity(args) or "(no identity)")


# --- roster / presence ---------------------------------------------------


def cmd_register(args):
    identity = _require_identity(args)
    fields = {"session_id": os.environ.get("CLAUDE_CODE_SESSION_ID"), "pid": os.getpid()}
    for k in ("project", "quest", "branch", "lead"):
        v = getattr(args, k, None)
        if v:
            fields[k] = v
    roster().upsert(identity, **fields)
    st.set_state(identity, "busy")
    try:
        mm = _mm_as(identity)
        q.seed(identity, _channels_for(identity, mm), mm)  # unread-clock starts now
    except SwarmError:
        pass
    send_primitive(f"🟢 **{identity}** online", channel=_presence_channel(), as_identity=identity)
    print(f"registered {identity}")


def cmd_heartbeat(args):
    roster().heartbeat(_require_identity(args))


def cmd_offline(args):
    identity = _require_identity(args)
    roster().offline(identity)
    st.clear(identity)
    send_primitive(f"⚫ **{identity}** offline", channel=_presence_channel(), as_identity=identity)
    print(f"offline {identity}")


_STATUS_ICON = {"online": "🟢", "stale": "🟡", "offline": "⚫"}


def cmd_who(args):
    agents = roster().list(include_offline=args.all, stale_secs=args.stale_secs)
    if args.json:
        print(json.dumps([a.to_dict() for a in agents], indent=2, default=str))
        return
    if not agents:
        print("(no agents)")
        return
    print(f"{'':2} {'IDENTITY':<20} {'PROJECT/QUEST':<24} {'AGE':>6}  MACHINE")
    for a in agents:
        agestr = f"{a.age_secs:.0f}s" if a.age_secs < 120 else f"{a.age_secs / 60:.0f}m"
        pq = " / ".join(x for x in (a.project, a.quest) if x) or "-"
        icon = _STATUS_ICON.get(a.status.value, "?")
        print(f"{icon:2} {a.identity:<20} {pq:<24} {agestr:>6}  {a.machine or '?'}")


def cmd_state(args):
    st.set_state(_require_identity(args), args.value, pid=args.pid)


# --- messaging -----------------------------------------------------------


def _swarm_channel() -> str:
    return config().get("MATTERMOST_SWARM_CHANNEL", default="swarm") or "swarm"


def _presence_channel() -> str:
    return config().get("MATTERMOST_PRESENCE_CHANNEL", default="presence") or "presence"


def cmd_send(args):
    ok = send_primitive(
        args.text, channel=args.channel, as_identity=args.as_identity or _identity(args)
    )
    print("sent" if ok else "send failed")


def cmd_say(args):
    ok = send_primitive(args.text, channel=_swarm_channel(), as_identity=_identity(args))
    print("posted to #swarm" if ok else "post failed")


def cmd_search(args):
    """Full-text search Mattermost, scoped to what this agent can see."""
    r = _mm_as(_identity(args)).search_posts(args.query) or {}
    order = r.get("order", [])
    if args.json:
        print(json.dumps([r["posts"][i] for i in order], default=str))
        return
    if not order:
        print("no matches.")
        return
    for i in order[: args.limit]:
        p = r["posts"][i]
        who = (p.get("props") or {}).get("override_username") or p.get("user_id", "?")[:8]
        print(f"  {who}: {p.get('message', '').strip()[:120]}")


def cmd_read(args):
    identity = _require_identity(args)
    mm = _mm_as(identity)
    posts = q.unread(
        identity, identity, _channels_for(identity, mm), mm, self_ids=_self_ids(identity)
    )
    if not posts:
        print("no unread.")
        return
    print(f"{len(posts)} unread:")
    for p in posts:
        flag = "‼️ " if q.classify(p.message, identity) is Priority.HIGH else "  "
        print(f"{flag}{str(p.user).split(' @ ')[0]}: {p.message.strip()}")


def cmd_drain(args):
    """For the channel server / busy-hook: new HIGH messages + low-priority count."""
    identity = _identity(args)
    if not identity:
        if args.json:
            print(json.dumps(DrainResult().to_dict()))
        return
    mm = _mm_as(identity)
    chans = args.channels.split(",") if args.channels else _channels_for(identity, mm)
    result = q.drain(
        identity, identity, chans, mm, high_only=args.high_only, self_ids=_self_ids(identity)
    )
    if args.json:
        print(json.dumps(result.to_dict()))
        return
    for m in result.high:
        print(f"💬 {m.sender}: {m.message[:140]}")
    if result.low_count:
        print(f"({result.low_count} other unread — `swarm read` to view)")


# --- quests (Vikunja) ----------------------------------------------------


def _vik():
    return VikunjaClient.from_config(config())


def _quest_line(t) -> str:
    pr = VikunjaPriority.parse(t.get("priority")).label
    owner = ",".join(a.get("username", "") for a in (t.get("assignees") or [])) or "-"
    done = "✓" if t.get("done") else " "
    return f"  [{done}] #{t['id']:<4} p:{pr:<7} @{owner:<12} {t['title']}"


def cmd_quest_list(args):
    tasks = _vik().list_tasks(done=args.done)
    if args.json:
        print(json.dumps(tasks, indent=2, default=str))
        return
    if not tasks:
        print("(no quests)")
        return
    for t in tasks:
        print(_quest_line(t))


def cmd_quest_show(args):
    print(json.dumps(_vik().get_task(args.id), indent=2, default=str))


def cmd_quest_search(args):
    tasks = _vik().search_tasks(args.query)
    if args.json:
        print(json.dumps(tasks, indent=2, default=str))
        return
    if not tasks:
        print("(no matches)")
        return
    for t in tasks:
        print(_quest_line(t))


def cmd_quest_claim(args):
    identity = _require_identity(args)
    # interim: comment the claim (per-agent Vikunja assignees pending); also @announce
    _vik().comment(args.id, f"Claimed by **{identity}**.")
    send_primitive(
        f"📋 **{identity}** claimed quest #{args.id}",
        channel=_swarm_channel(),
        as_identity=identity,
    )
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

    print(
        json.dumps(
            Registrar().create_project(args.title, description=args.desc), indent=2, default=str
        )
    )


# --- vikunja → MM bridge -------------------------------------------------


def cmd_bridge_sync(args):
    from .bridge import VikunjaBridge

    VikunjaBridge().sync(verbose=True)


def cmd_bridge_run(args):
    import time

    from .bridge import VikunjaBridge

    b = VikunjaBridge()
    print(f"bridge running (every {args.interval}s) — Ctrl-C to stop")
    while True:
        try:
            b.sync(verbose=True)
        except Exception as e:  # never let one bad poll kill the loop
            sys.stderr.write(f"bridge: {e}\n")
        time.sleep(args.interval)


# --- parser --------------------------------------------------------------


def build_parser():
    p = argparse.ArgumentParser(
        prog="swarm",
        description="vibeos agent-swarm CLI — identity, messaging, quests, roster, provisioning.",
        epilog="Identity resolves from --identity, then $SWARM_LOGIN, then `swarm login`.",
    )
    p.add_argument("--identity", help="override the acting identity for this command")
    sub = p.add_subparsers(dest="cmd", required=True)

    def add(name, fn, help):
        sp = sub.add_parser(name, help=help, description=help)
        sp.set_defaults(fn=fn)
        return sp

    s = add("login", cmd_login, "Bind this session to an agent identity.")
    s.add_argument("name")
    s.add_argument(
        "--no-default", action="store_true", help="don't set the machine-wide default identity"
    )
    add("logout", cmd_logout, "Unbind this session's identity.")
    add("whoami", cmd_whoami, "Print the current identity.")

    s = add(
        "register", cmd_register, "Register/refresh this identity in the roster (SessionStart)."
    )
    for a in ("project", "quest", "branch", "lead"):
        s.add_argument(f"--{a}")
    add("heartbeat", cmd_heartbeat, "Bump last_seen for this identity.")
    add("offline", cmd_offline, "Mark this identity offline.")
    s = add("who", cmd_who, "Show the swarm roster.")
    s.add_argument("--json", action="store_true")
    s.add_argument("--all", action="store_true")
    s.add_argument("--stale-secs", type=int, default=300)
    s = add("state", cmd_state, "Set busy/idle state (hooks).")
    s.add_argument("value", choices=["busy", "idle"])
    s.add_argument("--pid", type=int)

    s = add("send", cmd_send, "Post a message to a channel as this identity.")
    s.add_argument("text")
    s.add_argument("--channel", required=True)
    s.add_argument("--as", dest="as_identity")
    s = add("say", cmd_say, "Post chatter to #swarm as this identity.")
    s.add_argument("text")
    add("read", cmd_read, "Show unread messages across this agent's channels, in full.")
    s = add("search", cmd_search, "Full-text search Mattermost (what this agent can see).")
    s.add_argument("query")
    s.add_argument("--json", action="store_true")
    s.add_argument("--limit", type=int, default=15)
    s = add("drain", cmd_drain, "Emit new high-priority messages (+low count) for delivery.")
    s.add_argument("--json", action="store_true")
    s.add_argument("--high-only", action="store_true")
    s.add_argument(
        "--channels", help="comma-separated channel ids (default: #swarm + agent channel)"
    )

    s = add("quest", lambda a: a.qfn(a), "Quests (Vikunja tasks): list/show/claim/done/comment.")
    qsub = s.add_subparsers(dest="qcmd", required=True)
    qs = qsub.add_parser("list")
    qs.set_defaults(qfn=cmd_quest_list)
    qs.add_argument("--json", action="store_true")
    qs.add_argument("--done", action="store_true")
    qs = qsub.add_parser("show")
    qs.set_defaults(qfn=cmd_quest_show)
    qs.add_argument("id", type=int)
    qs = qsub.add_parser("search")
    qs.set_defaults(qfn=cmd_quest_search)
    qs.add_argument("query")
    qs.add_argument("--json", action="store_true")
    qs = qsub.add_parser("claim")
    qs.set_defaults(qfn=cmd_quest_claim)
    qs.add_argument("id", type=int)
    qs = qsub.add_parser("done")
    qs.set_defaults(qfn=cmd_quest_done)
    qs.add_argument("id", type=int)
    qs = qsub.add_parser("comment")
    qs.set_defaults(qfn=cmd_quest_comment)
    qs.add_argument("id", type=int)
    qs.add_argument("text")

    s = add("agent", lambda a: a.afn(a), "Provision/teardown an agent identity (registrar).")
    asub = s.add_subparsers(dest="acmd", required=True)
    as_ = asub.add_parser("create")
    as_.set_defaults(afn=cmd_agent_create)
    as_.add_argument("name")
    as_.add_argument("--lead", required=True)
    as_.add_argument("--charter")
    as_ = asub.add_parser("teardown")
    as_.set_defaults(afn=cmd_agent_teardown)
    as_.add_argument("name")

    s = add("lead", cmd_swarm_lead, "Register a human lead (Authentik subgroup).")
    s.add_argument("lead")
    s = add("project", lambda a: a.pfn(a), "Provision a project/epic.")
    psub = s.add_subparsers(dest="pcmd", required=True)
    ps = psub.add_parser("create")
    ps.set_defaults(pfn=cmd_project_create)
    ps.add_argument("title")
    ps.add_argument("--desc", default="")

    s = add("bridge", lambda a: a.bfn(a), "Vikunja → Mattermost ingestion (per-agent activity).")
    bsub = s.add_subparsers(dest="bcmd", required=True)
    bs = bsub.add_parser("sync", help="one poll: post new agent-relevant board activity")
    bs.set_defaults(bfn=cmd_bridge_sync)
    bs.add_argument("--verbose", action="store_true")
    bs = bsub.add_parser("run", help="poll on a loop")
    bs.set_defaults(bfn=cmd_bridge_run)
    bs.add_argument("--interval", type=int, default=15)
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
