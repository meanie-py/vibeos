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
from . import (
    cargoswap,
    checkpoint,
    debugcmd,
    diskspace,
    host_clock,
    hostlock,
    maintenance,
    symbolize,
    tooling,
)
from . import conformance as conformance_mod
from . import difftest as difftest_mod
from . import farm as farm_mod
from . import gate as gate_mod
from . import gdbengine as gdb_mod
from . import health as health_mod
from . import killvm as kill_mod
from . import land as land_mod
from . import prune as prune_mod
from . import repro as repro_mod
from . import rrcmd as rr_mod
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
    checkpoint.add_parser(sub)
    host_clock.add_parser(sub)

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
        "--rust-storage-shards",
        type=int,
        choices=range(17),
        help="persist direct staticlib storage layout: 0 private, 1-16 local Cargo shards",
    )
    b.add_argument(
        "--incremental",
        action="store_true",
        help="fast-iterate: rustc incremental (Rust sccache OFF; non-reproducible, not for gate)",
    )
    checks = b.add_mutually_exclusive_group()
    checks.add_argument("--check", action="store_true", help="also run advisory source lints")
    checks.add_argument("--no-check", action="store_true", help=argparse.SUPPRESS)
    b.add_argument(
        "--keep-going",
        action="store_true",
        help="ninja -k0: report EVERY broken target, not just the first (verdict builds)",
    )
    b.add_argument(
        "--lockdep",
        action="store_true",
        help="sync-instrumentation §4: build ntoskrnl with the self-deadlock detector "
        "(-DVIBEOS_LOCKDEP=ON). Debug/repro only — has a per-pushlock cost; off by default",
    )
    b.add_argument(
        "--cswitchdep",
        action="store_true",
        help="cswitchdep-design.md: build ntoskrnl with the context-switch hand-off "
        "verifier (-DVIBEOS_CSWITCHDEP=ON; implies the lockdep cargo feature). "
        "Debug/repro only; off by default",
    )
    b.add_argument(
        "--up-kernel",
        action="store_true",
        help="OPT OUT to the UP kernel: build media whose freeldr default entry "
        "boots ntoskrnl.exe + hal.dll (-DVIBEOS_UP_DEFAULT_BOOT=ON). The DEFAULT "
        "is the MP pair (ntkrnlmp.exe + halmp.dll) since 2026-09-12, and nothing "
        "automated should pass this -- UP is a dead branch, so a green UP run "
        "tells us nothing about what we ship. Note this selects the KERNEL "
        "FLAVOR, which is now independent of vCPU count: `--smp N` sets how many "
        "CPUs the MP kernel runs on",
    )
    b.add_argument(
        "--kmtests",
        action="store_true",
        help="configure the build dir with -DENABLE_ROSTESTS_KMTESTS=ON so the "
        "kmtest corpus is built and staged. STICKY: a later plain build keeps it "
        "(unlike --lockdep and friends, which reconcile back OFF). `conformance` "
        "passes this itself; you want it by hand only to provision a fresh "
        "worktree ahead of time",
    )
    b.add_argument(
        "--apitests",
        action="store_true",
        help="configure the build dir with -DENABLE_ROSTESTS_APITESTS=ON so the "
        "user-mode apitest corpus (63 suites) is built and staged, WITHOUT the "
        "winetests. STICKY, like --kmtests, and for the same reason: the apitest "
        "conformance payload always wants them. Prefer this over --rostests for "
        "anything that needs media, because 4 of the full corpus's non-apitest "
        "targets do not build (one is an llvm-rc gap we cannot patch) and a "
        "failed build produces no ISO",
    )
    b.add_argument(
        "--rostests",
        action="store_true",
        help="configure the build dir with -DENABLE_ROSTESTS=ON so the full "
        "winetest corpus (~128 suites) is built. RECONCILED, not sticky: a "
        "later plain build turns it back OFF, so an opt-in never bloats the "
        "default build. NOT all suites build: on 2026-09-23 an i386 "
        "--keep-going build still failed shell32_apitest, shlwapi_apitest, "
        "interop_unittest, menuaccels, combase and ole32 (user32 and "
        "ntdll_apitest were fixed that day), so no livecd is produced. "
        "Build one target with ninja and stage it as a payload instead",
    )
    b.add_argument(
        "--winetest-sweep",
        action="store_true",
        help="stage the winetest SWEEP marker, so winlogon runs EVERY subtest "
        "of each staged Wine conformance suite instead of the two hand-picked "
        "ones (user32:wsprintf, user32:class). Enumerated from the binary via "
        "`--list`, one guest process per subtest, so a sweep costs minutes and "
        "is deliberately off the gate's path. Needs the suites to exist: an "
        "i386 `build --rostests` first, then this on the arch whose media you "
        "will boot. RECONCILED, not sticky",
    )
    b.add_argument(
        "--winetest-skip",
        default="",
        metavar="SUBTESTS",
        help="space-separated subtest names to QUARANTINE from --winetest-sweep, "
        "e.g. 'input dde'. A subtest that BUGCHECKs the machine blinds every one "
        "after it in the sweep, so the list is how a sweep gets past a known "
        "crasher. Written into the sweep marker as `skip <name>` lines; winlogon "
        "logs each as QUARANTINED so an absent subtest never reads as a pass. "
        "Same discipline as tools/payloads/apitest/quarantine-amd64.txt: re-derive "
        "entries, do not inherit them",
    )
    b.add_argument(
        "--hal-pic",
        action="store_true",
        help="i386 legacy PIC HAL escape hatch "
        "(docs/engineering/active-lanes/i386-apic-hal-plan.md): build hal.dll "
        "with the legacy 8259 PIC interrupt model instead of the DEFAULT "
        "LAPIC/IOAPIC/TPR APIC HAL (-DVIBEOS_PIC_HAL=ON). i386 only (amd64 hal "
        "is always APIC); a plain build is the APIC HAL",
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
        "--scope",
        choices=("source", "source-early", "source-deferred", "arch"),
        help="strict partial check: shared sources or per-architecture guards; not a full gate",
    )
    c.add_argument(
        "--only",
        metavar="TOOL[,TOOL...]",
        help="run selected source scanners only; explicitly PARTIAL, not a landing gate",
    )
    c.add_argument(
        "--update-baseline",
        nargs="?",
        const="__ALL__",
        default=None,
        metavar="TOOL[,TOOL...]",
        help="capture current findings as the accepted baseline. Bare = ALL tools "
        "(footgun once the gate blocks — silently accepts unrelated drift); pass a "
        "tool name (or comma-list) to re-bless only that tool, e.g. "
        "--update-baseline lint_kernel",
    )
    c.set_defaults(fn=debugcmd.check)

    s = sub.add_parser("smoke", help="staleness-gated boot verdict (front door)")
    arch_arg(s)
    s.add_argument("--allow-stale", action="store_true")
    s.add_argument(
        "--timing",
        action="store_true",
        help="boot-phase timing: stamp the live serial log with elapsed seconds "
        "(VIBEOS_SERIAL_TIMESTAMPS=1) and print a boot-phase report "
        "(tools/boot_phase_report.py) after each boot",
    )
    s.add_argument("--no-gdb", action="store_true")
    s.add_argument("--attach-on-fail", action="store_true")
    s.add_argument("--repeat", type=int, default=1, help="N independent boots (flake rate)")
    s.add_argument("--until", default="", help="success marker (default: arch boot marker)")
    s.add_argument("--fail-on", default=None, help="regex; abort at first match")
    s.add_argument(
        "--tripwire",
        default="",
        help="per-boot kernel debug knobs via fw_cfg, e.g. 'SPECIALPOOL=Gla5' or "
        "'SPECIALPOOL=*'; KEY=VAL tokens separated by spaces/semicolons (no commas). "
        "See ntoskrnl-rust src/tripwire.rs",
    )
    s.add_argument(
        "--require-probe",
        action="append",
        metavar="LITERAL",
        help="refuse to boot unless LITERAL is present in EVERY boot image "
        "(repeatable). Use it whenever a run's conclusion depends on a probe "
        "firing: checking `strings build-<arch>/…/ntoskrnl.exe` is the wrong "
        "artifact, and a probe missing from the media makes its silence look "
        "like evidence of absence. Field manual §9.87",
    )
    s.add_argument(
        "--smp",
        type=int,
        default=0,
        help="boot with N virtual CPUs (-smp N via VIBEOS_QEMU_SMP). Opt-in "
        "SMP bring-up knob (docs/engineering/smp-bringup-quest.md); the default "
        "is ONE vcpu. That is the cpu COUNT, not the kernel: the MP kernel boots "
        "by default on every arch and --up-kernel is the opt-out, so the default "
        "run is the MP kernel on one vcpu. N>1 needs SMP-capable media (ntkrnlmp)",
    )
    s.add_argument(
        "--no-net",
        action="store_true",
        help="skip the amd64 netstack net-suite (default-on; escape hatch for fast iteration)",
    )
    s.add_argument(
        "--payload",
        default="",
        help="opt-in payload gate (tests/qemu/test_payload.py): comma-separated "
        "payload names; launch + render + input gates from each payload's YAML. "
        "Media must be built with VIBEOS_PAYLOAD=<names>. "
        "See docs/engineering/payload-harness-design.md",
    )
    s.add_argument(
        "--doom",
        action="store_true",
        help="legacy alias for --payload doom",
    )
    s.add_argument(
        "--ladder",
        action="store_true",
        help="after the desktop marker, keep draining serial until every WoW64 "
        "probe guest reports, and FAIL the boot on a red rung. The ladder runs "
        "after explorer, so a plain smoke/gate stops watching before it starts "
        "and cannot see a red guest at all (selfmod32 was red on amd64 for a "
        "day that way). Run it on BOTH 64-bit arches after touching a guest, "
        "wow64.dll or a marshaller; arm64 needs media built with VIBEOS_FEX=1. "
        "See tests/qemu/cli/ladder.py",
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
    g.add_argument(
        "--keep-going", action="store_true", help="ninja -k0 in each build: collect ALL failures"
    )
    g.add_argument("--no-net", action="store_true", help="skip the amd64 net stage (default-on)")
    g.add_argument(
        "--no-lint",
        action="store_true",
        help="skip the strict scanner suite (default-on; blocks the stamp on NEW findings)",
    )
    g.add_argument(
        "--no-conformance",
        action="store_true",
        help=(
            "skip the kmtest conformance ratchet lanes (default-on: one boot per arch, "
            "reusing this gate's media, failing on assertions that fail now and were not "
            "failing in tools/conformance-baseline/<arch>/kmtest.txt). Turning it off also "
            "drops --kmtests from the gate's builds"
        ),
    )
    g.set_defaults(fn=gate_mod.gate)

    la = sub.add_parser(
        "land",
        help="merge queue: submit your lane; train the batch if nobody else is",
    )
    la.add_argument(
        "--retries",
        type=int,
        default=2,
        help="gate flake-retries (passthrough; default 2)",
    )
    la.add_argument("--clean", action="store_true", help="cold builds in the gate")
    la.add_argument(
        "--direct",
        action="store_true",
        help="bypass the merge queue: merge master + gate + push this lane alone",
    )
    la.add_argument(
        "--drain",
        action="store_true",
        help="don't submit; just train the queue if nobody else is (orphan recovery)",
    )
    la.add_argument(
        "--foreground",
        action="store_true",
        help="conduct the batch inline instead of spawning a detached conductor",
    )
    la.add_argument("--wait", action="store_true", help="block until this submission's verdict")
    la.add_argument(
        "--wait-timeout", type=float, default=3600, help="seconds for --wait (default 3600)"
    )
    la.add_argument(
        "--no-push", action="store_true", help="do everything but the push (rehearse a landing)"
    )
    la.add_argument(
        "--skip-lint",
        action="store_true",
        help="skip the check-tooling preflight (the conductor still runs it; report every use)",
    )
    la.add_argument(
        "--dry-run",
        action="store_true",
        help="report the plan (merge needed? gate needed? who is landing) and exit",
    )
    la.add_argument(
        "--hold",
        metavar="REASON",
        help="STOP THE LINE: no conductor starts a batch, submissions still queue, only this "
        "lane may --direct. Say why; it shows in every land/land-status",
    )
    la.add_argument(
        "--preempt",
        action="store_true",
        help="with --hold: also stop a detached conductor mid-batch (its batch stays queued)",
    )
    la.add_argument("--release", action="store_true", help="lift the hold and wake the queue")
    la.add_argument(
        "--queue-timeout",
        type=float,
        default=0,
        help=f"seconds to wait for the land lock (default {hostlock.LAND_LOCK_TIMEOUT:.0f})",
    )
    la.set_defaults(fn=land_mod.land)

    ls = sub.add_parser("land-status", help="merge queue: conductor, pending submissions, verdicts")
    ls.add_argument("--json", action="store_true")
    ls.add_argument("--mine", action="store_true", help="only this lane's verdicts")
    ls.add_argument("--limit", type=int, default=8, help="recent verdicts to show (default 8)")
    ls.set_defaults(fn=land_mod.cmd_land_status)

    f = sub.add_parser(
        "farm", help="flake farm: soak boots into a durable corpus (~/vibeos-flake-farm)"
    )
    f.add_argument(
        "--arches", default="amd64", help="comma list (default amd64); parallel worker per arch"
    )
    f.add_argument(
        "--boots",
        type=int,
        default=0,
        help="max boots per arch (default 5, or unlimited with --minutes)",
    )
    f.add_argument("--minutes", type=float, default=0, help="wall-clock budget (overnight soak)")
    f.add_argument(
        "--tripwire",
        default="",
        help="arm every boot, e.g. 'SPECIALPOOL=*' (see smoke --tripwire)",
    )
    f.add_argument(
        "--require-probe",
        action="append",
        metavar="LITERAL",
        help="refuse to start unless LITERAL is in every boot image (repeatable). "
        "A soak is the expensive case: hours of boots against one frozen image, "
        "where a missing probe wastes the whole run AND reads as a null result. "
        "See smoke --require-probe",
    )
    f.add_argument(
        "--smp",
        type=int,
        default=0,
        help="soak with N vCPUs (VIBEOS_QEMU_SMP), same knob as `smoke --smp`. "
        "N>1 needs SMP-capable media (ntkrnlmp); every boot still runs smoke's "
        "_smp_check, so a soak that quietly booted the UP kernel FAILS rather "
        "than banking hours of false green. Without this the SMP lane could not "
        "use the farm corpus at all and had to hand-loop smoke",
    )
    f.add_argument("--allow-stale", action="store_true")
    f.set_defaults(fn=farm_mod.farm)

    fr = sub.add_parser("farm-report", help="cluster the farm corpus into failure families")
    fr.add_argument("--since", default="", help="only records with ts >= YYYYMMDD[-HHMMSS]")
    fr.add_argument("--sha", default="", help="only records from this short sha")
    fr.set_defaults(fn=farm_mod.farm_report)

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

    gp = gdbsub.add_parser(
        "probe", help="repeat a Python GDB probe on fresh boots; stop on hit/error"
    )
    gp.add_argument("arch", choices=ACCEPTED_ARCHES)
    gp.add_argument("--script", required=True, help="GDB Python script setting VIBEOS_PROBE_RESULT")
    gp.add_argument("--repeat", type=int, default=1)
    gp.add_argument(
        "--timeout", type=float, default=180.0, help="probe budget plus 30s cleanup margin"
    )
    gp.set_defaults(fn=gdb_mod.cmd_probe)

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

    cf = sub.add_parser(
        "conformance",
        help="NT test-corpus ratchet: build kmtest payload -> boot -> sweep -> "
        "parse -> compare vs baseline (quest M4)",
    )
    arch_arg(cf)
    cf.add_argument(
        "--no-build",
        action="store_true",
        help="skip the VIBEOS_PAYLOAD=kmtest build; media must already be staged",
    )
    cf.add_argument(
        "--write-baseline",
        action="store_true",
        help="store this run's results as the new per-arch baseline "
        "(tools/conformance-baseline/<arch>/kmtest.txt). Refuses if the sweep "
        "launched fewer tests than the existing baseline — that would delete "
        "them from the ratchet floor",
    )
    cf.add_argument(
        "--force-baseline",
        action="store_true",
        help="with --write-baseline, allow dropping tests the existing baseline "
        "has but this sweep never launched (only when a test was really retired)",
    )
    cf.add_argument(
        "--only",
        metavar="TEST",
        nargs="+",
        default=None,
        help="with --write-baseline, re-seed ONLY these tests and leave every "
        "other row at its banked sample. Use this whenever the sweep contains "
        "a known-flaky test: a wholesale rewrite re-rolls those rows from one "
        "sample and can freeze a lucky boot into the floor",
    )
    cf.add_argument(
        "--gate",
        action="store_true",
        help="exit nonzero on any ratchet regression (default: report-only)",
    )
    cf.add_argument(
        "--seed-asserts",
        action="store_true",
        help="with --repeat N (N>=3): add the failing-assertion identities to the "
        "existing baseline rows WITHOUT touching their counts or status. An "
        "identity every round that ran the test agreed on is written `assert`; "
        "one they disagreed on is written `assert?` (bistable). This is how a "
        "count-only baseline is upgraded to an identity floor — never a "
        "wholesale --write-baseline, which re-rolls the counts from one boot",
    )
    cf.add_argument(
        "--smp",
        type=int,
        default=0,
        help="boot the sweep with N vCPUs (VIBEOS_QEMU_SMP); N>1 builds the "
        "ntkrnlmp SMP media. For the SMP/AP lane's guest-load testing",
    )
    cf.add_argument(
        "--timeout",
        type=int,
        default=6000,
        help="serial-drain budget in seconds (default 6000; the sweep's launch_drain_secs is 5400)",
    )
    cf.add_argument(
        "--repeat",
        type=int,
        default=0,
        help="run the sweep N times (soak). Rounds after the first force "
        "--no-build so every round sweeps the SAME media, and each appends a "
        "record to ~/vibeos-flake-farm/conformance.jsonl carrying its face "
        "SHAPE (bugcheck / no-launch / stall / clean), sha, smp and tripwire. "
        "`farm` only boots to desktop and so cannot see any sweep-shaped "
        "failure; pair this with a SHORT --timeout, since a stalled round that "
        "drains the full default budget costs ~40x a clean one and is what "
        "limited the previous soak to 9 rounds",
    )
    cf.set_defaults(fn=conformance_mod.conformance)

    rrp = sub.add_parser(
        "rr",
        help="QEMU record/replay: capture one boot, then re-run it deterministically",
        description="Record a boot (kmtest sweep by default) into a self-contained "
        "trace, then replay that exact boot as often as you like — with gdb, and "
        "with reverse-execution available. Built for heisenbugs: a 1-in-60 crash "
        "you have recorded is a crash you can open, not one you have to catch again.",
    )
    rrsub = rrp.add_subparsers(dest="rr_verb", required=True)

    rrec = rrsub.add_parser("record", help="boot once under -icount rr=record and keep the trace")
    rrec.add_argument("--arch", required=True, choices=ACCEPTED_ARCHES)
    rrec.add_argument(
        "--payload",
        default="kmtest",
        choices=("kmtest", "none"),
        help="kmtest: record the conformance sweep (default — the most guest work "
        "per trace). none: record a plain smoke boot",
    )
    rrec.add_argument(
        "--smp",
        type=int,
        default=1,
        help="vCPUs (default 1). Record/replay is single-vCPU only; on the MP "
        "kernel --smp 1 IS the UP:=smp1 collapse case",
    )
    rrec.add_argument("--name", help="trace directory name (default <ts>-<arch>-<payload>)")
    rrec.add_argument(
        "--build", action="store_true", help="build the media first (default: use what is there)"
    )
    rrec.add_argument("--timeout", type=int, default=6000, help="serial-drain budget (seconds)")
    rrec.add_argument(
        "--shift",
        type=int,
        default=rr_mod.rr.DEFAULT_ICOUNT_SHIFT,
        help="icount shift: the virtual clock costs 2**N ns per instruction, so "
        "this fixes the guest's apparent CPU at 2**-N GHz. Firmware and "
        "bootloader watchdogs are denominated in that same virtual time, so too "
        "LARGE a value makes a healthy guest time out. Lower = faster virtual "
        f"CPU, bigger trace (default {rr_mod.rr.DEFAULT_ICOUNT_SHIFT})",
    )
    rrec.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rrec.set_defaults(fn=rr_mod.rr_cmd, rr_verb="record")

    rrep = rrsub.add_parser("replay", help="re-run a recorded boot, instruction-for-instruction")
    rrep.add_argument("trace", help="trace name, path, or `latest`")
    rrep.add_argument(
        "--gdb",
        action="store_true",
        help="open the gdbstub halted at instruction 0. CAVEAT on QEMU 9.0.2: "
        "connect/read-registers/continue work, but `stepi` aborts QEMU and "
        "`interrupt` does not stop a running replay, so reverse execution cannot "
        "be reached. See docs/engineering/qemu-record-replay.md",
    )
    rrep.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rrep.add_argument(
        "--break-at",
        type=int,
        metavar="ICOUNT",
        help="stop the replay at this icount via QMP replay-break, snapped forward "
        "to a legal instruction-chunk boundary (see `rr breaks`); pair with --gdb "
        "to inspect the paused guest",
    )
    rrep.set_defaults(fn=rr_mod.rr_cmd, rr_verb="replay")

    rvf = rrsub.add_parser(
        "verify",
        help="replay a trace and PROVE the guest re-executed identically (serial diff)",
    )
    rvf.add_argument("trace", help="trace name, path, or `latest`")
    rvf.add_argument("--gdb", action="store_true", help=argparse.SUPPRESS)
    rvf.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rvf.set_defaults(fn=rr_mod.rr_cmd, rr_verb="verify")

    rpr = rrsub.add_parser(
        "prune",
        help="delete old traces (DRY-RUN unless --apply); a kmtest capture is ~1.4GB",
    )
    rpr.add_argument("--keep", type=int, default=3, help="keep the N newest traces (default 3)")
    rpr.add_argument(
        "--name", action="append", help="delete these traces by name instead of by age"
    )
    rpr.add_argument("--apply", action="store_true", help="actually delete (default: dry-run)")
    rpr.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rpr.set_defaults(fn=rr_mod.rr_cmd, rr_verb="prune")

    rls = rrsub.add_parser("list", help="list recorded traces")
    rls.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rls.set_defaults(fn=rr_mod.rr_cmd, rr_verb="list")

    rif = rrsub.add_parser("info", help="describe a trace and flag anything that would diverge")
    rif.add_argument("trace", help="trace name, path, or `latest`")
    rif.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rif.set_defaults(fn=rr_mod.rr_cmd, rr_verb="info")

    rbk = rrsub.add_parser(
        "breaks",
        help="icounts a replay of this trace can legally be stopped at (see `--near`)",
    )
    rbk.add_argument("trace", help="trace name, path, or `latest`")
    rbk.add_argument("--root", help=f"trace root (default {rr_mod.DEFAULT_ROOT})")
    rbk.add_argument(
        "--near",
        type=int,
        help="show the boundaries around this icount and the one it would snap to; "
        "without it, print a summary of the whole set",
    )
    rbk.add_argument(
        "--count", type=int, default=5, help="how many boundaries either side of --near (default 5)"
    )
    rbk.set_defaults(fn=rr_mod.rr_cmd, rr_verb="breaks")

    kl = sub.add_parser(
        "kill",
        help="stop a QEMU SAFELY (uid/pid-scoped; refuses to kill our own ancestors)",
    )
    kl.add_argument("pids", nargs="*", type=int, help="qemu pid(s) from `vibeos-cli ps`")
    kl.add_argument("--arch", help="every VM of this arch")
    kl.add_argument("--stale", action="store_true", help="orphans: owning process is gone")
    kl.add_argument("--overdue", action="store_true", help="past the lifetime deadline")
    kl.add_argument("--all", action="store_true", help="every running VM (explicit, loud)")
    kl.add_argument("--dry-run", action="store_true", help="print what would be killed")
    kl.set_defaults(fn=kill_mod.cmd_kill)

    sy = sub.add_parser(
        "sym",
        help="resolve an address to module!symbol+off (USER-MODE PEs, which the "
        "kernel symbol index does not cover)",
    )
    sy.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    sy.add_argument("addr", nargs="+", help="hex addresses, e.g. 7ffb70078fd")
    sy.add_argument(
        "--module",
        help="force a module by name (needed when several share an image base, "
        "e.g. drivers on the linker default)",
    )
    sy.add_argument(
        "--rebuild", action="store_true", help="reindex even if the index looks current"
    )
    sy.set_defaults(fn=symbolize.run)

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

    rp = sub.add_parser(
        "reap", help="kill orphan QEMUs (owner gone) and any past their lifetime cap"
    )
    rp.add_argument("--dry-run", action="store_true", help="report, kill nothing")
    rp.add_argument("--verbose", "-v", action="store_true", help="(accepted; always verbose)")
    rp.set_defaults(fn=_cmd_reap)

    hs = sub.add_parser(
        "host-status", help="host CPU broker: who holds the build/boot lease (across worktrees)"
    )
    hs.add_argument(
        "--json", action="store_true", help="machine-readable (for wait-for-quiet polls)"
    )
    hs.set_defaults(fn=hostlock.cmd_status)

    from . import buildpolicy

    bp = sub.add_parser("host-policy", help="preview or activate a drained-fleet build trial")
    bp_mode = bp.add_mutually_exclusive_group()
    bp_mode.add_argument(
        "--apply", action="store_true", help="activate three builds after rollout/drain"
    )
    bp_mode.add_argument("--rollback", action="store_true", help="return to two builds after drain")
    bp_mode.add_argument(
        "--pause", action="store_true", help="refuse new CPU work; let holders finish"
    )
    bp_mode.add_argument("--resume", action="store_true", help="resume CPU admission after cutover")
    bp.set_defaults(fn=buildpolicy.command)

    ds = sub.add_parser(
        "disk-status", help="disk headroom, live reservations, and optional footprint report"
    )
    ds.add_argument(
        "--sizes", action="store_true", help="measure this lane's outputs and shared caches"
    )
    ds.set_defaults(fn=diskspace.command)

    ct = sub.add_parser(
        "check-tooling",
        help="lint/type/test gate for our own Python tools (rustconv, cli, tools)",
    )
    ct.add_argument("--no-types", action="store_true", help="skip pyright")
    ct.add_argument("--no-tests", action="store_true", help="skip pytest")
    ct.set_defaults(fn=tooling.check_tooling)

    dt = sub.add_parser(
        "difftest",
        help="host differential harnesses: shipping Rust vs the genuine C it replaced",
    )
    dt.add_argument(
        "--only",
        metavar="NAME[,NAME]",
        help="restrict to named *-difftest crates (default: every workspace member)",
    )
    dt.set_defaults(fn=difftest_mod.difftest)

    pr = sub.add_parser(
        "prune", help="reclaim disk: GC idle worktrees' build+target dirs (dry-run by default)"
    )
    pr.add_argument("--apply", action="store_true", help="actually delete (default: dry-run)")
    pr.add_argument("--self", action="store_true", help="also allow pruning the current worktree")
    pr.add_argument(
        "--retain-gib",
        type=float,
        help="retain this fleet output budget; remove eligible trees largest-first only until met",
    )
    pr.add_argument(
        "--idle-hours",
        type=float,
        default=0,
        help="only prune trees untouched for N hours (default 24 with --retain-gib, otherwise 0)",
    )
    pr.add_argument(
        "--include-integration",
        action="store_true",
        help="allow idle conductor scratch to participate in retention",
    )
    pr.add_argument(
        "--min-free-gib",
        type=float,
        help="also reclaim eligible outputs toward this filesystem headroom",
    )
    pr.set_defaults(fn=prune_mod.prune)

    from . import windows_cleanup

    wc = sub.add_parser("windows-cleanup", help="clean abandoned Windows QEMU staging")
    wc.add_argument("--dry-run", action="store_true", help="list eligible paths without deleting")
    wc.set_defaults(fn=windows_cleanup.command)

    from . import retention

    rt = sub.add_parser("retention", help="configure or run hourly idle-output retention")
    modes = rt.add_mutually_exclusive_group()
    modes.add_argument("--enable", action="store_true")
    modes.add_argument("--disable", action="store_true")
    modes.add_argument("--run", action="store_true")
    rt.add_argument("--retain-gib", type=float, default=100)
    rt.add_argument("--min-free-gib", type=float, default=150)
    rt.add_argument("--idle-hours", type=float, default=24)
    rt.set_defaults(fn=retention.command)

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


def _cmd_reap(args, repo: Path) -> int:
    """`vibeos-cli reap` -- and `qemu/reap.sh`, which is now a wrapper around it. Action lines
    go to stderr as `[reap] ...` (start.sh passes them through to the user)."""
    from tests.qemu.vm import reap_orphan_qemus

    actions = reap_orphan_qemus(dry_run=bool(getattr(args, "dry_run", False)))
    for line in actions:
        print(f"[reap] {line}", file=sys.stderr)
    failed = [a for a in actions if a.startswith("FAILED")]
    print(f"reap: {len(actions) - len(failed)} action(s), {len(failed)} failed")
    return 1 if failed else 0


def main(argv: list[str] | None = None) -> int:
    # Before anything can take a CPU permit: guarantee a policy EXISTS. It is durable
    # now, so this is a no-op on a healthy box -- it is here for the fresh clone and for
    # the case where the file was lost, which used to mean "silently build uncapped".
    from . import buildpolicy

    buildpolicy.heal()
    args = build_parser().parse_args(argv)
    if args.cmd not in maintenance.UNLOCKED and args.cmd != "retention":
        try:
            with maintenance.activity(REPO):
                return _dispatch(args)
        except maintenance.MaintenanceError as exc:
            # Reuse the storage-admission infrastructure classification: no
            # check ran, so the conductor must not blame a source regression.
            print(f"DISK_ADMISSION_FAIL: maintenance: {exc}", flush=True)
            return 2
    return _dispatch(args)


def _dispatch(args) -> int:
    # The except must stay OUTSIDE the `in REQUESTS` test: a DEFERRED kind takes its own
    # reservation deep inside the command (build, once the CPU broker admits it), and that
    # AdmissionError has to reach the same stdout line and exit 2 as one raised here, or the
    # conductor would classify an infrastructure refusal as a source regression.
    try:
        if args.cmd in diskspace.DEFERRED:
            # Fail fast if the host is hopeless, then let the command claim its real
            # budget once the CPU broker admits it.
            diskspace.refuse_if_hopeless(REPO, args.cmd)
        elif args.cmd in diskspace.REQUESTS:
            with diskspace.reserve(REPO, args.cmd):
                return args.fn(args, REPO)
        return args.fn(args, REPO)
    except diskspace.AdmissionError as exc:
        # stdout is the durable fallback when even summary.json cannot be
        # created. The conductor classifies this as infrastructure, not code.
        print(f"{diskspace.FAILURE}: {exc}", flush=True)
        return 2


def _cli_exit_code(argv: list[str] | None = None) -> int:
    """main(), but argparse's own usage exits come back as a value like any other."""
    try:
        return main(argv)
    except SystemExit as exc:  # argparse usage errors exit(2) from inside parse_args
        code = exc.code
        if code is None:
            return 0
        return code if isinstance(code, int) else 1


def cli_main(argv: list[str] | None = None) -> int:
    """Entry point that puts the exit code in the OUTPUT, not just the status.

    A pipeline reports the exit status of its LAST stage, so `vibeos-cli ... | tail`
    yields 0 no matter how the CLI exited -- agents have three times in one night
    reported a finding manufactured by reading a green pipeline off a red command.
    A rule against piping has not stopped it (it is in the always-loaded push channel,
    and one report quoted the rule while breaking it), so the number is emitted into
    the stream the reader is actually looking at. It survives `| tail`, `| tail -1`
    and a redirect alike, and unlike a not-a-tty warning it has no false positives:
    it is not a warning, it is the missing datum.

    Nothing parses our stdout tail today -- the last-line consumers in gate.py and
    land.py read `bash build.sh`, pytest and `git push`, never a vibeos-cli child.
    But this line is now RESERVED: it is always last, so a consumer that shells out
    to vibeos-cli and wants the salient diagnostic must reverse-SEARCH for it the way
    gate.py:392 finds its pytest summary, never take `.splitlines()[-1]`. Blind-tailing
    a vibeos-cli child gets `exit=N` and looks like the CLI stopped reporting failures.
    (land.py:287 and gate.py:258 do blind-tail, and are safe only because of what they
    happen to read -- copy gate.py:392's idiom instead, not theirs.)

    `main()` stays free of this on purpose: unit tests call it directly, and a plain
    return value is easier to assert than captured stdout. Don't tidy the two together.
    """
    rc = 1
    try:
        rc = _cli_exit_code(argv)
    finally:
        print(f"vibeos-cli: exit={rc}", flush=True)
    return rc


if __name__ == "__main__":
    sys.exit(cli_main())
