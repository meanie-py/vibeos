"""vibeos-cli entry point: `python -m tests.qemu.cli <subcommand>`.

Locates the repo root from this file (not cwd-dependent), dispatches to the
command modules. No default --arch (plan principle #4).
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from . import build as build_mod
from . import cargoswap, debugcmd, hostlock, tooling
from . import gate as gate_mod
from . import gdbengine as gdb_mod
from . import health as health_mod
from . import prune as prune_mod
from . import repro as repro_mod
from . import smoke as smoke_mod
from .archcfg import ACCEPTED_ARCHES

# repo root = .../tests/qemu/cli/__main__.py -> up 3
REPO = Path(__file__).resolve().parents[3]


def _ps(args, repo: Path) -> int:
    ps = repo / "qemu" / "ps"
    if not ps.is_file():
        print("qemu/ps not found")
        return 127
    cmd = ["python3", str(ps)] + (["--json"] if getattr(args, "json", False) else [])
    return subprocess.run(cmd, cwd=repo).returncode


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="vibeos-cli", description="VibeOS build/check/smoke/debug control plane"
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    def arch_arg(sp, required=True):
        sp.add_argument(
            "--arch",
            choices=ACCEPTED_ARCHES,
            required=required,
            help="target arch (i386/amd64 alias to UEFI; no default)",
        )

    b = sub.add_parser("build", help="ninja + binary-lints + build-index + stamp")
    arch_arg(b)
    b.add_argument("--clean", action="store_true", help="cold: nuke all rust targets")
    b.add_argument(
        "--incremental",
        action="store_true",
        help="fast-iterate: rustc incremental (Rust sccache OFF; non-reproducible, not for gate)",
    )
    b.add_argument("--no-check", action="store_true", help="skip auto source-lints")
    b.add_argument(
        "--lockdep",
        action="store_true",
        help="sync-instrumentation §4: build ntoskrnl with the self-deadlock detector "
        "(-DVIBEOS_LOCKDEP=ON). Debug/repro only — has a per-pushlock cost; off by default",
    )
    b.add_argument("--strict", action="store_true")
    b.add_argument(
        "--cache",
        action=argparse.BooleanOptionalAction,
        default=None,
        help="ccache(C/C++)+sccache(Rust); ON by default (--no-cache / VIBEOS_BUILD_CACHE=0)",
    )
    b.set_defaults(fn=build_mod.build)

    c = sub.add_parser("check", help="source lints (baseline-diff gate)")
    c.add_argument("--arch", choices=ACCEPTED_ARCHES, required=False)
    c.add_argument("--strict", action="store_true", help="block on NEW findings")
    c.add_argument(
        "--update-baseline",
        action="store_true",
        help="capture current findings as the accepted baseline",
    )
    c.set_defaults(fn=debugcmd.check)

    s = sub.add_parser("smoke", help="staleness-gated boot verdict (front door)")
    arch_arg(s)
    s.add_argument("--allow-stale", action="store_true")
    s.add_argument("--no-gdb", action="store_true")
    s.add_argument("--attach-on-fail", action="store_true")
    s.add_argument("--repeat", type=int, default=1, help="N independent boots (flake rate)")
    s.add_argument("--until", default="", help="success marker (default: arch boot marker)")
    s.add_argument("--fail-on", default=None, help="regex; abort at first match")
    s.add_argument(
        "--no-net",
        action="store_true",
        help="skip the amd64 netstack net-suite (default-on; escape hatch for fast iteration)",
    )
    s.set_defaults(fn=smoke_mod.smoke)

    g = sub.add_parser(
        "gate", help="parallel pre-land verdict: build+smoke all arches, auto-retry flakes"
    )
    g.add_argument(
        "--arches",
        default=None,
        help="comma list (default i386,amd64,arm64); i386/amd64 alias to UEFI",
    )
    g.add_argument(
        "--retries",
        type=int,
        default=2,
        help="parallel retry attempts for a DOCUMENTED flake (default 2; 0 = none)",
    )
    g.add_argument("--no-build", action="store_true", help="smoke only (media must be current)")
    g.add_argument("--clean", action="store_true", help="cold builds (implies build)")
    g.add_argument("--no-net", action="store_true", help="skip the amd64 net stage (default-on)")
    g.set_defaults(fn=gate_mod.gate)

    bo = sub.add_parser("boot", help="raw fast boot (shares smoke arch-config)")
    arch_arg(bo)
    bo.add_argument("--until", required=True)
    bo.add_argument("--fail-on", default=None)
    bo.add_argument("--json", action="store_true")
    bo.set_defaults(fn=smoke_mod.boot)

    d = sub.add_parser("debug", help="run-id scoped debug verbs (wrap shell toolkit)")
    dsub = d.add_subparsers(dest="verb", required=True)
    for verb in (
        "start",
        "inspect",
        "run-until",
        "bt",
        "triage-bugcheck",
        "wait-for",
        "tail",
        "status",
        "stop",
        "screenshot",
        "profile",
        "gdb",
    ):
        dv = dsub.add_parser(verb)
        dv.add_argument("rest", nargs=argparse.REMAINDER)
        dv.set_defaults(fn=debugcmd.debug, verb=verb)
    dd = dsub.add_parser("diff", help="normalized serial diff between two runs")
    dd.add_argument("run_a")
    dd.add_argument("run_b")
    dd.set_defaults(fn=debugcmd.debug, verb="diff")

    # gdb — native-Python GDB engine over the windows-host backend (breakpoints fire
    # under TCG, unlike native KVM). Full port of the qemu/*.sh debug toolkit.
    gdbp = sub.add_parser("gdb", help="debug a windows-host QEMU via gdb (Python engine)")
    gdbsub = gdbp.add_subparsers(dest="gverb", required=True)

    gst = gdbsub.add_parser("start", help="launch a detached debug QEMU + gdbstub")
    gst.add_argument("arch", choices=ACCEPTED_ARCHES)
    gst.add_argument("--wait", action="store_true", help="halt at reset (-S) for boot-from-zero")
    gst.set_defaults(fn=gdb_mod.cmd_start)

    gstat = gdbsub.add_parser("status", help="gdbstub reachability + serial size for a run")
    gstat.add_argument("run_id")
    gstat.set_defaults(fn=gdb_mod.cmd_status)

    gwf = gdbsub.add_parser("wait-for", help="poll the serial log for a regex")
    gwf.add_argument("run_id")
    gwf.add_argument("--pattern", required=True)
    gwf.add_argument("--timeout", type=float, default=120.0)
    gwf.set_defaults(fn=gdb_mod.cmd_wait_for)

    gru = gdbsub.add_parser("run-until", help="break at a symbol, then run inspect verbs on hit")
    gru.add_argument("run_id")
    gru.add_argument("--break", dest="brk", required=True, help="breakpoint symbol/addr")
    gru.add_argument("--timeout", type=float, default=180.0)
    gru.add_argument("--then", action="append", help="verb to run on hit (repeatable)")
    gru.set_defaults(fn=gdb_mod.cmd_run_until)

    gin = gdbsub.add_parser("inspect", help="run one inspect verb (bt/triage-bugcheck/...)")
    gin.add_argument("run_id")
    gin.add_argument("verb", nargs=argparse.REMAINDER, help="e.g. bt 30 | triage-bugcheck")
    gin.add_argument("--timeout", type=float, default=60.0)
    gin.set_defaults(fn=gdb_mod.cmd_inspect)

    gstop = gdbsub.add_parser("stop", help="kill the debug QEMU + clean staging")
    gstop.add_argument("run_id")
    gstop.set_defaults(fn=gdb_mod.cmd_stop)

    gps = gdbsub.add_parser("ps", help="list debug runs + liveness")
    gps.set_defaults(fn=gdb_mod.cmd_ps)

    cs = sub.add_parser(
        "cargo-swap", help="runtime gate: boot-swap cargo-driven ntdll DLLs + ntoskrnl (x86)"
    )
    arch_arg(cs)
    cs.add_argument(
        "--module",
        choices=["all", "ntdll", "ntdll_vista", "ntoskrnl"],
        default="all",
        help="which cargo-driven module to boot-swap (ntoskrnl is i386-only)",
    )
    cs.set_defaults(fn=cargoswap.cargo_swap)

    ps = sub.add_parser("ps", help="all running VMs + slots (across worktrees)")
    ps.add_argument("--json", action="store_true")
    ps.set_defaults(fn=_ps)

    he = sub.add_parser(
        "health",
        help="trailing per-arch pass rates from the durable ledger (rates > point samples)",
    )
    he.add_argument("--days", type=float, default=7, help="trailing window (default 7)")
    he.add_argument("--json", action="store_true", help="raw ledger rows")
    he.set_defaults(fn=health_mod.health)

    hs = sub.add_parser(
        "host-status", help="host CPU broker: who holds the build/boot lease (across worktrees)"
    )
    hs.add_argument(
        "--json", action="store_true", help="machine-readable (for wait-for-quiet polls)"
    )
    hs.set_defaults(fn=hostlock.cmd_status)

    ct = sub.add_parser(
        "check-tooling",
        help="lint/type/test gate for our own Python tools (rustconv, cli, tools)",
    )
    ct.add_argument("--no-types", action="store_true", help="skip pyright")
    ct.add_argument("--no-tests", action="store_true", help="skip pytest")
    ct.set_defaults(fn=tooling.check_tooling)

    pr = sub.add_parser(
        "prune", help="reclaim disk: GC idle worktrees' build+target dirs (dry-run by default)"
    )
    pr.add_argument("--apply", action="store_true", help="actually delete (default: dry-run)")
    pr.add_argument("--self", action="store_true", help="also allow pruning the current worktree")
    pr.add_argument(
        "--idle-hours",
        type=float,
        default=0,
        help="only prune trees untouched for at least N hours (default 0 = no age filter)",
    )
    pr.set_defaults(fn=prune_mod.prune)

    # repro — determinism/reproducibility measurement (det-plan rung 0/5)
    rp = sub.add_parser(
        "repro", help="reproducibility gauge: byte-identity + PE-timestamp/section diff"
    )
    rpsub = rp.add_subparsers(dest="repro_cmd")

    def _add_diffoscope(sp):
        sp.add_argument(
            "--diffoscope",
            action="store_true",
            help="deep byte localization if diffoscope installed",
        )

    rp_c = rpsub.add_parser("compare", help="compare two PE files (localize the divergence)")
    rp_c.add_argument("file_a")
    rp_c.add_argument("file_b")
    _add_diffoscope(rp_c)
    rp_r = rpsub.add_parser("report", help="dump determinism metadata (timestamp/sections)")
    rp_r.add_argument("files", nargs="+")
    rp_d = rpsub.add_parser("diff-dirs", help="diff the staged artifact set across two build trees")
    rp_d.add_argument("dir_a")
    rp_d.add_argument("dir_b")
    rp_d.add_argument("--artifacts", nargs="*", help="relative artifact paths (default: staged)")
    _add_diffoscope(rp_d)
    rp_v = rpsub.add_parser("verify", help="rung 5: cold build-twice byte-identity gauge")
    rp_v.add_argument("--arch", choices=ACCEPTED_ARCHES, required=True)
    rp_v.add_argument("--artifacts", nargs="*", help="relative artifact paths (default: staged)")
    _add_diffoscope(rp_v)
    rp.set_defaults(fn=repro_mod.repro)

    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.fn(args, REPO)


if __name__ == "__main__":
    sys.exit(main())
