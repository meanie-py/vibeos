"""`vibeos-cli kill` — stop a QEMU safely, because the rule alone was not enough.

AGENTS.md's most dangerous rule is *"Never broad-kill QEMUs — you WILL kill yourself"*, and
it has killed a Claude mid-run at least twice (exit 9 / exit 144). The mechanism it asks for
— `backend.kill_by_uid`, pid/uid/pgroup-scoped — has existed all along, but only `vm.py` and
the gdb engine could reach it. There was no CLI verb. So an agent that needed to stop a boot
had to either hand-roll a Python call against `qemu/ps` output, or type `pkill -f qemu` — the
exact thing that kills it. A rule with no tool is a rule you get to break.

Two invariants make this safe, and they are enforced in CODE rather than asked for in prose:

  1. **Selection is never by pattern.** You select by qemu pid, by arch, or by a *state*
     (`--stale`: owner process gone; `--overdue`: past its lifetime deadline). There is no
     name/regex selector, because that is the footgun: `pkill -f "qemu/start.sh"` matches
     your own shell's ancestor and SIGKILLs your whole process tree.
  2. **A target that is one of OUR OWN ancestors is refused**, loudly. That is the actual
     failure mode — not "a bad regex" in the abstract, but killing the process tree you are
     running inside. Checked by walking /proc ppid chain, so it holds however the target
     was selected.

`--all` exists but is explicit and prints what it is about to do; it is still uid-scoped
per VM, never a pattern sweep.
"""

from __future__ import annotations

import json
import os
import re
import signal
import time
from pathlib import Path

from tests.qemu import backend as be

from .artifacts import Run, Stage

LOCK_DIR = Path(os.environ.get("VIBEOS_QEMU_LOCK_DIR", "/tmp/vibeos-qemu-locks"))


def _pid_is_zombie(pid: int) -> bool:
    """True if `pid` has exited but its parent has not reaped it yet.

    `os.kill(pid, 0)` succeeds on a zombie — the pid is still in the table — so a
    naive liveness check reads a process that is already dead as alive, and no
    further signal will change that: only the parent can reap it. This produced a
    "STILL ALIVE after SIGTERM+SIGKILL — unkillable (D state?)" report for a QEMU
    that had in fact died on the SIGTERM, which is a worse lie than the silent
    success this escalation path was added to fix: it sends the reader hunting an
    unkillable process that does not exist.

    Linux-only; anywhere without /proc this returns False and the caller falls
    back to the pid check, which is what it did before.
    """
    try:
        for line in (Path("/proc") / str(pid) / "status").read_text().splitlines():
            if line.startswith("State:"):
                return line.split()[1] == "Z"
    except OSError:
        return False
    return False


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    except OSError:
        return False
    return True


def _ppid(pid: int) -> int | None:
    """Parent of `pid` from /proc, or None. `stat` field 4 is ppid, but the comm field (2)
    can contain spaces and parens — so split after the last ')'."""
    try:
        raw = Path(f"/proc/{pid}/stat").read_text()
    except OSError:
        return None
    try:
        return int(raw[raw.rindex(")") + 1 :].split()[1])
    except (ValueError, IndexError):
        return None


def own_ancestors(pid: int | None = None) -> set[int]:
    """Every ancestor of `pid` (default: us), up to init. Killing one of these is the
    self-immolation the never-broad-kill rule exists to prevent."""
    out: set[int] = set()
    cur = pid if pid is not None else os.getpid()
    seen = 0
    while cur and cur > 1 and seen < 64:
        out.add(cur)
        cur = _ppid(cur) or 0
        seen += 1
    return out


def running_vms() -> list[dict]:
    """Live QEMU manifests (same lock files `qemu/ps` reads), newest first."""
    vms: list[dict] = []
    try:
        paths = sorted(LOCK_DIR.glob("*.json"))
    except OSError:
        return vms
    for p in paths:
        try:
            info = json.loads(p.read_text())
        except (OSError, ValueError):
            continue
        qpid = info.get("qemu_pid")
        if not isinstance(qpid, int):
            continue
        info["_lock"] = str(p)
        info["alive"] = _pid_alive(qpid)
        lpid = info.get("launcher_pid")
        info["owner_alive"] = bool(isinstance(lpid, int) and _pid_alive(lpid))
        vms.append(info)
    return vms


def win_uid_of(info: dict) -> str:
    """The windows-host `-name <uid>` tag from the recorded argv, if any. That tag is what
    makes a kill precise on the Windows side (a concurrent agent's qemu.exe is untouched)."""
    cmd = info.get("cmd") or []
    if isinstance(cmd, str):
        cmd = cmd.split()
    for i, tok in enumerate(cmd):
        if tok == "-name" and i + 1 < len(cmd):
            return str(cmd[i + 1])
        m = re.match(r"^-name=(.+)$", str(tok))
        if m:
            return m.group(1)
    return ""


def select(vms: list[dict], args) -> tuple[list[dict], str]:
    """Resolve the selector to a concrete VM list. Returns (selected, description)."""
    pids = {int(p) for p in (getattr(args, "pids", None) or [])}
    if pids:
        return [v for v in vms if v["qemu_pid"] in pids], f"pid(s) {sorted(pids)}"
    if getattr(args, "stale", False):
        return [v for v in vms if not v["owner_alive"]], "orphans (owner process gone)"
    if getattr(args, "overdue", False):
        import time

        now = time.time()
        return [v for v in vms if (v.get("deadline") or 0) and now > v["deadline"]], "past deadline"
    if getattr(args, "arch", None):
        a = str(args.arch)
        return [v for v in vms if str(v.get("arch", "")).startswith(a)], f"arch {a}"
    if getattr(args, "all", False):
        return list(vms), "ALL running QEMUs"
    return [], ""


def kill_one(info: dict, *, dry_run: bool = False) -> str:
    """Stop one VM the precise way for its backend. Returns a one-line outcome."""
    qpid = info["qemu_pid"]
    uid = win_uid_of(info)
    what = f"qemu_pid={qpid} arch={info.get('arch', '?')}"
    if dry_run:
        return f"would kill {what}" + (f" via uid {uid}" if uid else " via SIGTERM")
    if uid:
        # windows-host: the process lives on the Windows side; the uid tag is the only
        # precise handle (its Linux-side pid is the interop shim, not qemu.exe).
        be.kill_by_uid(uid)
        return f"killed {what} (uid {uid})"
    try:
        os.kill(qpid, signal.SIGTERM)
    except ProcessLookupError:
        return f"already gone {what}"
    except OSError as e:
        return f"FAILED {what}: {e}"

    # SIGTERM is a REQUEST, and a QEMU can decline it. One under `-icount` with a
    # wedged replay does exactly that: measured, a VM survived two `vibeos-cli
    # kill` calls 29 minutes apart, each of which reported success, while holding
    # a concurrency slot and 94% of a core. Reporting "SIGTERM sent" as if it were
    # "stopped" is the same trap as trusting a tool's OK over the artifact.
    #
    # So: confirm, and escalate if it declines. start.sh's own watchdog has always
    # done TERM -> sleep -> KILL; the interactive path never did.
    for _ in range(25):  # ~5s
        time.sleep(0.2)
        try:
            os.kill(qpid, 0)
        except ProcessLookupError:
            return f"SIGTERM {what}"
        if _pid_is_zombie(qpid):
            return f"SIGTERM {what} (exited; awaiting reap by its parent)"
    try:
        os.kill(qpid, signal.SIGKILL)
    except ProcessLookupError:
        return f"SIGTERM {what}"
    except OSError as e:
        return f"SIGTERM ignored, SIGKILL FAILED {what}: {e}"
    for _ in range(25):
        time.sleep(0.2)
        try:
            os.kill(qpid, 0)
        except ProcessLookupError:
            return f"SIGKILL {what} (declined SIGTERM)"
        if _pid_is_zombie(qpid):
            return f"SIGKILL {what} (declined SIGTERM; awaiting reap by its parent)"
    if _pid_is_zombie(qpid):
        return f"SIGKILL {what} (exited; awaiting reap by its parent)"
    return f"STILL ALIVE after SIGTERM+SIGKILL {what} — unkillable (D state?)"


def cmd_kill(args, repo: Path) -> int:
    run = Run("kill", getattr(args, "arch", None) or "vms")
    vms = [v for v in running_vms() if v["alive"]]
    if not vms:
        # Same distinction as the `not chosen` branch below, one level earlier:
        # "no QEMUs running" is a fine answer to a sweep and a WRONG answer to
        # "kill these pids", which is a request that then silently did nothing.
        if getattr(args, "pids", None):
            run.add(
                Stage(
                    "select",
                    status="fail",
                    detail="no tracked QEMUs at all; pid(s) "
                    f"{', '.join(map(str, args.pids))} were NOT killed",
                    salient=[
                        "a QEMU started outside qemu/start.sh or tests/qemu/vm.py has no "
                        "lockfile and is invisible here",
                        "confirm it is yours, then stop it by pid directly",
                    ],
                )
            )
            return run.finalize(repo)
        run.add(Stage("select", status="ok", detail="no QEMUs running"))
        return run.finalize(repo)
    chosen, how = select(vms, args)
    if not how:
        run.add(
            Stage(
                "select",
                status="fail",
                detail="no selector — pass pids, --arch, --stale, --overdue, or --all",
                salient=["refusing to guess which VM you meant; `vibeos-cli ps` lists them"],
            )
        )
        return run.finalize(repo)
    if not chosen:
        # A SWEEP matching nothing is success -- `--stale` with no orphans is the
        # normal case. Explicit PIDS matching nothing is not: the caller named
        # processes they wanted stopped and none were, and reporting `ok` there
        # sends them away believing it worked. Measured: a QEMU launched outside
        # start.sh/vm.py has no lockfile, so it is invisible to `running_vms()`,
        # and `vibeos-cli kill <its pid>` reported OK twice, 29 minutes apart,
        # while it held 94% of a core.
        explicit = list(getattr(args, "pids", None) or [])
        if explicit:
            run.add(
                Stage(
                    "select",
                    status="fail",
                    detail=f"no tracked VM for pid(s) {', '.join(map(str, explicit))}",
                    salient=[
                        "these pids are not in the QEMU lockfile registry, so nothing was "
                        "signalled — they were NOT killed",
                        "a QEMU started outside qemu/start.sh or tests/qemu/vm.py is not "
                        "tracked; stop it by pid yourself after confirming it is yours",
                        "`vibeos-cli ps` lists exactly what this command can see",
                    ],
                )
            )
            return run.finalize(repo)
        run.add(Stage("select", status="ok", detail=f"nothing matched {how}"))
        return run.finalize(repo)

    # THE guard: never kill a process we are running inside.
    mine = own_ancestors()
    suicidal = [v for v in chosen if v["qemu_pid"] in mine or v.get("launcher_pid") in mine]
    if suicidal:
        run.classification = "SELF_KILL_REFUSED"
        run.add(
            Stage(
                "guard",
                status="fail",
                detail=f"{len(suicidal)} target(s) are OUR OWN ancestors — refusing",
                salient=[
                    f"qemu_pid={v['qemu_pid']} launcher_pid={v.get('launcher_pid')} "
                    "is in this process's ancestry; killing it would take out this agent"
                    for v in suicidal
                ],
            )
        )
        return run.finalize(repo)

    dry = getattr(args, "dry_run", False)
    outcomes = [kill_one(v, dry_run=dry) for v in chosen]
    for line in outcomes:
        print(line)
    run.add(
        Stage(
            "kill",
            status="ok",
            detail=f"{'would stop' if dry else 'stopped'} {len(chosen)} VM(s) — {how}",
            salient=outcomes[:5],
        )
    )
    run.extra["killed"] = [v["qemu_pid"] for v in chosen]
    return run.finalize(repo)
