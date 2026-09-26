"""`vibeos-cli conformance` — the NT test-corpus ratchet front door (quest M4).

One verb: (optionally) build a kmtest-payload image, boot it, run the full
in-guest kmtest sweep, parse the serial log into per-test results, and compare
against the committed per-arch baseline
(`tools/conformance-baseline/<arch>/kmtest.txt`).

Report-only by default (the quest keeps it report-only until a baseline is
stable across >=3 runs); `--gate` makes a ratchet regression exit nonzero so it
can be wired as an opt-in gate stage. `--write-baseline` stores the current
results as the new floor.

The sweep mechanics live in the payload harness: media built with
`VIBEOS_PAYLOAD=kmtest` grafts `kmtest-sweep.exe` into the All Users Startup
folder + `kmtest-tests.txt` next to the binaries; the sweep runs one
`kmtest_.exe` per listed test and tees results to serial. `payload.yaml` sets
`launch_marker: "[KMTEST-SWEEP] done"` + `launch_drain_secs`, so
`test_payload.py::test_payload_launched` holds the boot open and drains serial
until the sweep completes (or BUGCHECKs). We then parse
the private `<run_dir>/<pytest_arch>_serial.log`.

Design: docs/engineering/conformance-ratchet-quest.md (M3/M4).
"""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from . import artifacts, flakes

SUITE = "kmtest"
# base arch -> pytest arch (x86 boot the UEFI configs; arm64 is itself).
_PYTEST_ARCH = {"i386": "i386-uefi", "amd64": "amd64-uefi"}


def _load_parser(repo: Path):
    """Import tools/conformance/parse_kmtest_serial.py as a module."""
    path = repo / "tools" / "conformance" / "parse_kmtest_serial.py"
    spec = importlib.util.spec_from_file_location("parse_kmtest_serial", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load parser at {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def baseline_coverage_refusal(results, base_map, *, force: bool) -> str | None:
    """Refuse a `--write-baseline` that would DELETE tests from the ratchet floor.

    A baseline is a floor, and `--write-baseline` replaces it wholesale with this
    run's rows. So a sweep that launched fewer tests than the floor — a boot flake,
    an early BUGCHECK, a drained-out timeout — silently removes the missing tests
    from the floor, permanently, with no signal anywhere. That is the ratchet form
    of "never weaken a test to make it pass", and catching it was a hand-check
    before every re-seed.

    Returns the refusal message, or None to proceed.
    """
    new_names = {r["name"] for r in results}
    dropped = sorted(n for n in base_map if n not in new_names)
    if not dropped or force:
        return None
    return (
        f"[conformance] --write-baseline REFUSED: this sweep launched "
        f"{len(results)} tests but the existing baseline has {len(base_map)}, and "
        f"writing it would DELETE {len(dropped)} test(s) from the ratchet floor:\n"
        + "".join(f"    {n}\n" for n in dropped)
        + "  A short sweep is usually a boot flake or an early BUGCHECK, not a real "
        "coverage change — re-run it.\n"
        "  If the removal is intended (a test was retired), pass --force-baseline."
    )


def baseline_lowering_note(results, base_map) -> str | None:
    """Say out loud which rows a `--write-baseline` is about to make worse.

    Unlike coverage loss this is NOT refused: absorbing a known regression into the
    floor is sometimes deliberate (i386 and arm64 both accepted ExHardError
    30/0 -> 24/6 on purpose). It just must never happen silently.
    """
    worse = [
        f"    {r['name']}: pass {b.get('pass')} -> {r.get('pass')}, "
        f"fail {b.get('fail')} -> {r.get('fail')}"
        for r in results
        if (b := base_map.get(r["name"]))
        and (r.get("fail", 0) > b.get("fail", 0) or r.get("pass", 0) < b.get("pass", 0))
    ]
    # Counts can be flat while the SET moved — that is the whole reason identities
    # exist, so a re-seed that absorbs a newly-failing assertion has to say so even
    # when pass/fail did not budge.
    for r in results:
        b = base_map.get(r["name"])
        if not b or b.get("asserts") is None:
            continue
        new_ids = sorted(set(r.get("asserts") or ()) - b["asserts"])
        if new_ids:
            worse.append(
                f"    {r['name']}: absorbing {len(new_ids)} newly-failing "
                f"assertion(s): {', '.join(new_ids)}"
            )
    if not worse:
        return None
    return (
        "[conformance] NOTE: --write-baseline is LOWERING the floor for "
        f"{len(worse)} test(s) — accepting these as the new normal:\n" + "\n".join(worse)
    )


def seed_asserts_into_baseline(baseline_text: str, rounds, parser) -> tuple[str, str]:
    """Add `assert` / `assert?` lines to existing rows. Counts are NOT touched.

    Returns `(new_text, report)`.

    Unanimity is computed PER TEST over the rounds that actually ran it to
    completion, not over all rounds. A sweep that bugchecks partway through has
    no assertions for anything after the crash, and counting those rounds as
    "did not fail" would mark every later test's stable failures bistable —
    turning one bad boot into a permanently weakened floor. That is the same
    mistake `--only` exists to prevent, so this refuses to make it.

    Rows the baseline does not carry are SKIPPED, not added: adding a test to
    the floor is `--write-baseline`'s job and wants the coverage refusal in
    front of it.
    """
    # test -> [set(identities), ...] one entry per round that ran it
    seen: dict[str, list[set]] = {}
    for results in rounds:
        for r in results:
            # A test that did not finish tells us nothing about which of its
            # assertions fail; it only tells us the round died.
            if r["status"] in parser.BAD:
                continue
            seen.setdefault(r["name"], []).append(set(r.get("asserts") or ()))

    out: list[str] = []
    stable = flaky = rows = 0
    skipped: list[str] = []
    unseen: list[str] = []
    current: str | None = None
    for line in baseline_text.splitlines():
        # Drop any existing assert block; it is rebuilt from this soak.
        if line[:1].isspace() and line.strip().startswith("assert"):
            continue
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            out.append(line)
            continue
        current = stripped.split()[0]
        out.append(line)
        obs = seen.get(current)
        if obs is None:
            unseen.append(current)
            continue
        rows += 1
        union = set().union(*obs) if obs else set()
        for ident in sorted(union):
            unanimous = all(ident in o for o in obs)
            out.append(f"    assert{'' if unanimous else '?'} {ident}")
            if unanimous:
                stable += 1
            else:
                flaky += 1
    for name in sorted(seen):
        if name not in baseline_text:
            skipped.append(name)

    report = (
        f"[conformance] seeded {rows} row(s) with {stable} stable + {flaky} bistable "
        f"assertion identities from {len(rounds)} round(s)"
    )
    if unseen:
        report += (
            f"\n  {len(unseen)} baseline row(s) got NO identities — no round completed "
            f"them, so their count gate stands: {', '.join(unseen[:8])}"
            + (" ..." if len(unseen) > 8 else "")
        )
    if skipped:
        report += (
            f"\n  {len(skipped)} test(s) ran but are not on the floor; adding a row is "
            f"--write-baseline's job, not this: {', '.join(skipped[:8])}"
            + (" ..." if len(skipped) > 8 else "")
        )
    return "\n".join(out) + "\n", report


def _flaky_of(baseline_text: str, name: str) -> set[str]:
    """The `assert?` identities row `name` already carries.

    A re-seed must carry these forward. Re-rolling a row from one sweep would
    otherwise delete the bistable marks — banking whichever face that boot
    happened to show and handing the next honest run a false regression. It is
    the same failure `--only` exists to prevent, one level down.
    """
    out: set[str] = set()
    in_row = False
    for line in baseline_text.splitlines():
        if line[:1].isspace() and line.strip().startswith("assert"):
            kind, _, ident = line.strip().partition(" ")
            if in_row and kind == "assert?":
                out.add(ident.strip())
            continue
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        in_row = stripped.split()[0] == name
    return out


def merge_baseline_rows(existing_text: str, results, only, fmt):
    """Re-seed ONLY the named rows, leaving every other row's banked sample alone.

    A wholesale `--write-baseline` re-rolls EVERY row from the single sweep in
    front of it, including the rows we know are bistable (IoDeviceObject on all
    three arches, ExPools on arm64). One lucky boot then becomes the permanent
    floor and the next honest run reads as a regression. Until this existed the
    only safe way to re-seed a deterministic row was to hand-edit the file, which
    is how the CmSecurity row was done — fine for one row, not for six.

    Returns `(new_text, refusal)`; `refusal` is None on success.
    """
    by_name = {r["name"]: r for r in results}
    missing = sorted(n for n in only if n not in by_name)
    if missing:
        return None, (
            "[conformance] --write-baseline --only REFUSED: this sweep has no "
            f"result for {len(missing)} named test(s):\n"
            + "".join(f"    {n}\n" for n in missing)
            + "  A named test that never launched cannot be re-seeded from this run."
        )

    wanted = list(dict.fromkeys(only))
    seen: set[str] = set()
    out = []
    dropping = False  # inside the indented assert block of a row being re-seeded
    for line in existing_text.splitlines():
        # An indented `assert`/`assert?` line belongs to the row above it, not to
        # a test called "assert". Re-seeding a row replaces its whole block, so
        # the old block is dropped here and fmt() writes the new one; every other
        # row keeps its identities untouched, which is the whole point of --only.
        if line[:1].isspace() and line.strip().startswith("assert"):
            if not dropping:
                out.append(line)
            continue
        name = line.split()[0] if line.strip() and not line.startswith("#") else None
        if name is not None and name in by_name and name in wanted:
            dropping = True
            out.append(fmt(by_name[name], _flaky_of(existing_text, name)))
            seen.add(name)
        else:
            dropping = False
            out.append(line)
    # A named test the baseline does not carry yet is an ADDITION, not a re-seed.
    # Appending is right — it raises the floor — but say so rather than silently
    # growing the file.
    added = [n for n in wanted if n not in seen]
    out.extend(fmt(by_name[n], _flaky_of(existing_text, n)) for n in added)
    if added:
        print(
            "[conformance] --only added "
            f"{len(added)} test(s) the baseline did not carry: {', '.join(added)}"
        )
    return "\n".join(out) + "\n", None


def _run_dir(arch: str) -> Path:
    from .artifacts import create_run_dir

    return create_run_dir("conformance", arch)


# Durable soak ledger, alongside the flake farm's corpus so one triage pass can
# read both. /tmp/vibeos-cli run dirs rotate and a sweep truncates the previous
# same-arch serial log, so a multi-hour soak that keeps its findings only there
# loses them. The 2026-07-26 arm64 soak had to be partly RETRACTED because
# nobody could reconstruct, after the fact, which media each round ran on —
# hence `sha`/`dirty`/`smp`/`tripwire` in every record, not just the verdict.
FARM_DIR = Path.home() / "vibeos-flake-farm"
LEDGER = FARM_DIR / "conformance.jsonl"
_ROUNDS: list[dict] = []


def _head_sha(repo: Path) -> str:
    try:
        r = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return r.stdout.strip() if r.returncode == 0 else "?"
    except Exception:
        return "?"


def _append_round(rec: dict) -> None:
    _ROUNDS.append(rec)
    try:
        FARM_DIR.mkdir(parents=True, exist_ok=True)
        with LEDGER.open("a") as f:
            f.write(json.dumps(rec) + "\n")
    except Exception as e:  # a soak must never die on its own bookkeeping
        print(f"[conformance] ledger append failed: {e}", file=sys.stderr)


def _worktree_provisioning_gaps(repo: Path, base_arch: str) -> list[str]:
    """The gitignored things a fresh worktree lacks, each with its fix command.

    `git worktree add` brings tracked files only, so a new tree is missing the
    pinned toolchain symlink and the prebuilt payload binaries — and both fail
    several layers down from their cause, naming neither the file nor the remedy.
    Returns a list of human-readable gaps; empty means provisioned.
    """
    gaps: list[str] = []

    # 1. Toolchain. Sharing this one ACROSS worktrees is explicitly sanctioned —
    #    it is content-addressed and commit-independent, unlike generated
    #    headers — and build.sh below needs it, so report it first.
    tc = repo / ".build-tools" / "llvm-mingw"
    if not tc.exists() and not list(repo.parent.glob("llvm-mingw-*")):
        gaps.append(
            f"missing {tc} (pinned clang) -> CMAKE_CONFIGURE_FAIL "
            f"'no llvm-mingw toolchain found'\n"
            f"    fix: mkdir -p {tc.parent} && "
            f"ln -s <path-to>/llvm-mingw-*-ubuntu-*-x86_64 {tc}"
        )

    # 2. Payload binaries — .gitignore'd beside their own source
    #    (tools/payloads/kmtest/.gitignore: kmtest-sweep-*.exe), so a fresh tree
    #    has sweep.c and the test lists but not the exes. BUILD them; do NOT
    #    copy from a sibling worktree. They are generated per-commit artifacts,
    #    and resolving those across trees is the fleet-isolation trap that had
    #    verify_signatures grading one tree's Rust against another's headers.
    sweep = repo / "tools" / "payloads" / SUITE / f"{SUITE}-sweep-{base_arch}.exe"
    if not sweep.exists():
        gaps.append(
            f"missing {sweep} -> mkramdisk fails with \"payload '{SUITE}': src missing\"\n"
            f"    fix: bash tools/payloads/{SUITE}/build.sh {base_arch}"
        )

    # 3. arm64 boot assets. These are opaque firmware blobs, not built from our
    #    source, so copying IS the fix here (AGENTS.md says so explicitly).
    if base_arch == "arm64":
        for asset in ("QEMU_EFI.fd", "efivars.fd"):
            p = repo / "qemu" / asset
            if not p.exists():
                gaps.append(
                    f"missing {p} (gitignored firmware blob)\n"
                    f"    fix: cp <main-checkout>/qemu/{asset} {p}"
                )
    return gaps


def conformance(args, repo: Path) -> int:
    """`--repeat N` soak wrapper around one sweep.

    The arm64 flip blocker's five faces are all SWEEP faces — a bugcheck partway
    through, a sweep that launches nothing, a sweep that stalls — so `farm`,
    which only boots to desktop, cannot see any of them, and the soak that found
    them was hand-looped. Rounds after the first force `--no-build`: the media is
    fixed for the run, which is what makes rounds comparable.
    """
    repeat = int(getattr(args, "repeat", 0) or 0)
    seed_asserts = bool(getattr(args, "seed_asserts", False))
    if seed_asserts and repeat < 3:
        # The whole value of an identity floor is telling a stable failure from a
        # bistable one, and one or two boots cannot. The quest doc's own rule is
        # ">=3 runs" before a baseline is believed; enforce it rather than letting
        # a single lucky sweep mark 277 stable identities as gospel.
        print(
            "[conformance] --seed-asserts needs --repeat 3 or more: with fewer "
            "rounds a bistable assertion is indistinguishable from a stable one, "
            "and marking it `assert` freezes one boot's face into the floor.",
            file=sys.stderr,
        )
        return 2
    if repeat <= 1:
        return _conformance_once(args, repo)

    worst = 0
    # A soak whose media cannot launch anything produces no information, only
    # noise — and it appends that noise to the SHARED ledger every other lane
    # computes its statistics from. An unprovisioned worktree (no build-<arch>,
    # or a build that failed) banked 38 `launched=0` rounds in about a minute
    # before anyone noticed, and they are indistinguishable in the ledger from
    # the genuine no-launch face the smp corpus records 13 of.
    #
    # So: bail after CONSECUTIVE no-launch rounds, not the first. One is a real
    # observed face and must still be recorded; three in a row is broken media.
    NOLAUNCH_ABORT = 3
    consecutive_nolaunch = 0
    for i in range(repeat):
        if i:
            args.no_build = True
        print(f"\n=== conformance round {i + 1}/{repeat} ({args.arch}) ===", flush=True)
        built_here = not args.no_build
        rounds_before = len(_ROUNDS)
        try:
            rc = _conformance_once(args, repo)
        except KeyboardInterrupt:
            print(f"[conformance] interrupted after {i} round(s)", file=sys.stderr)
            break
        except Exception as e:  # one bad round must not end the soak
            print(f"[conformance] round {i + 1} raised {e!r} — continuing", file=sys.stderr)
            continue
        worst = max(worst, rc)

        # The build round failed: only this round builds, so every later one
        # would boot the PREVIOUS media and bank its verdicts in the shared
        # ledger as if they measured this tree. Measured 2026-09-24: an
        # assembler error in trap.S failed round 1, and rounds 2-3 then booted
        # the stale kernel and recorded two 0xE3 bugchecks against a fix that
        # was never in it.
        if built_here and rc != 0 and len(_ROUNDS) == rounds_before:
            print(
                "\n[conformance] ABORTING the soak: the build round failed, and every "
                "further round would boot stale media.",
                file=sys.stderr,
            )
            break

        if _ROUNDS and _ROUNDS[-1].get("launched") == 0:
            consecutive_nolaunch += 1
            if consecutive_nolaunch >= NOLAUNCH_ABORT:
                print(
                    f"\n[conformance] ABORTING the soak: {consecutive_nolaunch} consecutive "
                    f"rounds launched ZERO tests.\n"
                    f"  The media is not runnable, so every further round would append noise "
                    f"to {LEDGER}\n"
                    f"  rather than measure anything. This is nearly always an unprovisioned "
                    f"tree:\n"
                    f"    fix: ./vibeos-cli build --arch {args.arch}   (and check it SUCCEEDS)\n"
                    f"  A worktree also needs .build-tools/llvm-mingw and, on arm64, the "
                    f"gitignored\n"
                    f"  qemu/QEMU_EFI.fd + qemu/efivars.fd copied from the main checkout.",
                    file=sys.stderr,
                )
                worst = max(worst, 1)
                break
        else:
            consecutive_nolaunch = 0

    # Roll-up. Deliberately counts the SHAPES that distinguish the faces
    # (bugcheck / launched=0 / stalled / clean) rather than just pass-fail: a
    # bare failure count is what made nine rounds read as one bug.
    print(f"\n=== soak roll-up: {len(_ROUNDS)} round(s) -> {LEDGER} ===")
    shapes: dict[str, int] = {}
    for r in _ROUNDS:
        shapes[r["shape"]] = shapes.get(r["shape"], 0) + 1
    for k, v in sorted(shapes.items(), key=lambda kv: -kv[1]):
        print(f"  {v:>3}  {k}")

    if seed_asserts:
        worst = max(worst, _seed_asserts_from_rounds(args, repo))
    return worst


def _seed_asserts_from_rounds(args, repo: Path) -> int:
    """Re-parse this soak's archived serial logs and write the identity floor."""
    parser = _load_parser(repo)

    base = args.arch.split("-")[0]
    smp = args.smp or 0
    name = f"{SUITE}-smp{smp}.txt" if smp > 1 else f"{SUITE}.txt"
    baseline_path = repo / "tools" / "conformance-baseline" / base / name
    if not baseline_path.is_file():
        print(
            f"[conformance] --seed-asserts needs an existing baseline at {baseline_path};"
            " seed the counts first with --write-baseline",
            file=sys.stderr,
        )
        return 2

    rounds = []
    for rec in _ROUNDS:
        run_dir = Path(rec.get("run_dir", ""))
        # GLOB, do not spell the name: the archived serial is named for the
        # PYTEST arch, so i386 writes "i386-uefi_serial.log" and looking for
        # "i386_serial.log" silently finds nothing. That failure is quiet — the
        # soak runs, every round is clean, and the seeding step reports "0 rounds
        # left a serial log" after the boots are already spent.
        logs = sorted(run_dir.glob("*_serial.log")) if run_dir.is_dir() else []
        if not logs:
            continue
        results, _done, _bug = parser.parse(logs[0].read_text(errors="replace"))
        if results:
            rounds.append(results)
    if len(rounds) < 3:
        print(
            f"[conformance] --seed-asserts: only {len(rounds)} round(s) left an archived "
            f"serial log with results; need 3. Nothing written.",
            file=sys.stderr,
        )
        return 2

    text, report = seed_asserts_into_baseline(baseline_path.read_text(), rounds, parser)
    baseline_path.write_text(text)
    print(report)
    print(f"[conformance] identity floor written: {baseline_path}")
    return 0


def _conformance_once(args, repo: Path) -> int:
    raw = args.arch
    base = raw.split("-")[0]  # i386 / amd64 / arm64 (baseline dir + build --arch)
    pytest_arch = _PYTEST_ARCH.get(raw, raw)  # amd64 -> amd64-uefi; passthrough if suffixed
    smp = args.smp or 0

    if smp == 1 and getattr(args, "write_baseline", False):
        # `--smp 1` ratchets against the UP baseline on purpose (below); letting it
        # WRITE that file would overwrite the UP oracle with MP results and quietly
        # destroy the only thing the collapse experiment is measured against.
        print(
            "[conformance] --smp 1 --write-baseline refused: the UP baseline must be "
            f"produced by the UP kernel.\n  seed it with `--arch {base} --write-baseline` "
            "(no --smp), or use --smp 2 for an MP-specific baseline.",
            file=sys.stderr,
        )
        return 2

    parser = _load_parser(repo)
    run_dir = _run_dir(base)
    # SMP runs ratchet against their own baseline (kmtest-smp<N>.txt): MP kernels
    # legitimately diverge from UP (tests branch on KmtIsMultiProcessorBuild,
    # skip MP-fatal sections, expect SYNCH_LEVEL=IPI-2), so comparing an -smp
    # sweep against the UP baseline reports phantom regressions.
    #
    # `--smp 1` is the EXCEPTION and the point of the UP→smp1 collapse experiment
    # (smp-bringup-quest.md "UP endgame"): MP kernel, ONE cpu, ratcheted against
    # the UP baseline. Those two must agree test-for-test before the MP kernel can
    # become the default, so the UP baseline is deliberately the right oracle here.
    baseline_name = f"{SUITE}-smp{smp}.txt" if smp > 1 else f"{SUITE}.txt"
    baseline_path = repo / "tools" / "conformance-baseline" / base / baseline_name

    # --- 1. build (default on) with the kmtest payload -----------------------
    if not args.no_build:
        unprovisioned = _worktree_provisioning_gaps(repo, base)
        if unprovisioned:
            # Fail loud on the two gitignored things a FRESH WORKTREE silently
            # lacks — the toolchain and the payload exes. (The kmtest corpus used
            # to need a refusal here too; --kmtests below just fixes it.) Both cost
            # a full
            # build cycle each on 2026-07-29 because they surface far from their
            # cause: the toolchain as CMAKE_CONFIGURE_FAIL, and the payload as
            # `mkramdisk(ramdisk re-stage) -> payload 'kmtest': src missing`.
            # Neither names the fix; a preflight can.
            print(
                "[conformance] this worktree is not provisioned to build a payload:\n"
                + "".join(f"  - {g}\n" for g in unprovisioned)
                + "  (all are gitignored, so `git worktree add` cannot bring them.)",
                file=sys.stderr,
            )
            return 2
        # No kmtest corpus means a media that boots fine and then reports
        # `[KMTEST-SWEEP] launch FAILED`, minutes later and a layer away from the
        # cause — so ASK for the corpus rather than hoping the dir already has it.
        # --kmtests is sticky and cmake-cached, so this is a no-op on every run
        # after the first.
        #
        # This used to be a refuse-with-a-recipe on `_rostests_enabled() is False`,
        # which covered a dir configured WITHOUT the tests but not the case that
        # actually bites a fresh worktree: no build dir at all returns None, sailed
        # straight through, and then the fresh configure defaulted BOTH options OFF
        # (archcfg.py) — the silent degradation the check existed to prevent,
        # entering through the one door it did not cover. Nothing to refuse now:
        # both states want the same one-time reconfigure and `build` can do it.
        env = dict(os.environ, VIBEOS_PAYLOAD=SUITE)
        build_cmd = [str(repo / "vibeos-cli"), "build", "--arch", base, "--kmtests"]
        # ANY --smp N (including 1) must sweep the ntkrnlmp media: at N=1 that is
        # the whole experiment — "the MP kernel on one CPU" — and sweeping the UP
        # kernel instead would report a meaningless all-green.
        #
        # No arch needs a special build for that any more. Every media carries
        # both kernels and both HALs plus an MP freeldr entry, and
        # vm.smp_selected_iso repoints `DefaultOS=` inside a COPY of it at boot
        # time. --smp-kernel would only rewrite that same line at build time —
        # i.e. a whole media re-roll to change one byte we change for free.
        # arm64 was the last holdout (its curated ramdisk genuinely lacked the MP
        # pair until mkramdisk.sh started staging it unconditionally); forcing
        # --smp-kernel there now just costs a re-roll and, worse, makes the
        # --smp 1 and --smp 2 sweeps need two different builds of one commit.
        print(f"[conformance] building {base} with VIBEOS_PAYLOAD={SUITE} ...")
        bp = subprocess.run(build_cmd, cwd=str(repo), env=env)
        if bp.returncode != 0:
            print("[conformance] BUILD FAILED — aborting.", file=sys.stderr)
            return bp.returncode

    # --- 2. run the in-guest sweep -------------------------------------------
    # test_payload_launched drains serial up to launch_drain_secs waiting for
    # [KMTEST-SWEEP] done. Give the subprocess a margin over the drain budget.
    test_file = repo / "tests" / "qemu" / "test_payload.py"
    env = dict(os.environ)
    if smp:
        env["VIBEOS_QEMU_SMP"] = str(smp)
    # Give the shelled-out VM a durable place to drop hang diagnostics. Its own
    # work_dir is a pytest tmp_path that pytest prunes to the last 3 runs, so
    # without this a wedge's vCPU-PC capture is written and then deleted by
    # unrelated later runs — which is exactly what happened to the arm64 MP
    # wedge, leaving a run dir with serial and nothing else.
    env["VIBEOS_ARTIFACT_DIR"] = str(run_dir)
    sweep_cmd = [
        sys.executable,
        "-m",
        "pytest",
        str(test_file),
        "--arch",
        pytest_arch,
        "--payload",
        SUITE,
        "--skip-clean-build",  # we just built (or --no-build); don't nuke
        "--timeout",
        str(args.timeout),
        "-q",
        "-p",
        "no:cacheprovider",
        "-k",
        "launched",  # only the launch+drain test
    ]
    print(
        f"[conformance] {SUITE} sweep on {pytest_arch}"
        f"{f' -smp {smp}' if smp else ''} — draining up to {args.timeout}s ..."
    )
    t0 = time.time()
    sweep_rc: int | None = None
    try:
        sp = subprocess.run(sweep_cmd, cwd=str(repo), env=env, timeout=args.timeout + 300)
        sweep_rc = sp.returncode
    except subprocess.TimeoutExpired:
        print(
            f"[conformance] sweep subprocess exceeded {args.timeout + 300}s — "
            f"parsing whatever serial was captured.",
            file=sys.stderr,
        )
    elapsed = time.time() - t0

    # --- 3. parse the serial log ---------------------------------------------
    log_path = run_dir / f"{pytest_arch}_serial.log"
    if not log_path.is_file():
        print(f"[conformance] no serial log at {log_path} — did the guest boot?", file=sys.stderr)
        return 2
    serial_text = log_path.read_text(errors="replace")
    results, done_line, bugcheck = parser.parse(serial_text)

    lines = [parser.fmt(r) for r in results]
    ok = sum(1 for r in results if r["status"] == "OK")
    bad = sum(1 for r in results if r["status"] in parser.BAD)
    results_path = run_dir / f"{SUITE}-results.txt"
    results_path.write_text("\n".join(lines) + "\n")
    # The VM writes directly into this run, including while it is still alive.
    saved_serial = log_path

    # --- 3b. a wedged sweep: symbolicate the vCPU PCs ------------------------
    # The raw capture is written by the VM (VIBEOS_ARTIFACT_DIR above); resolving
    # the PCs to symbols has to happen HERE because it needs the build tree and
    # the MP/UP kernel choice, neither of which the in-guest sweep knows. Without
    # the symbols the file is a wall of hex and the operator still cannot tell
    # "spinning at DISPATCH_LEVEL" from "idle, waiting" — which is the whole fork
    # a stall hunt turns on.
    hang_state_path = run_dir / "hang-cpustate.txt"
    hang_pcs: list[str] = []
    if hang_state_path.is_file():
        try:
            from . import triage as _triage

            hang_pcs = _triage.symbolicate_cpu_state(
                repo,
                # BASE arch, not pytest_arch: the symbol index is
                # qemu/symbols-<base>.tsv and addr2sym.sh's arch_validate accepts
                # only i386/amd64/arm64, so "i386-uefi" resolves NOTHING — and it
                # fails by returning no symbols, not by complaining.
                base,
                hang_state_path.read_text(errors="replace"),
                mp="ntkrnlmp.exe" in serial_text,
                serial=serial_text,
            )
            if hang_pcs:
                with hang_state_path.open("a") as f:
                    f.write("\n=== symbolicated ===\n" + "\n".join(hang_pcs) + "\n")
        except Exception as exc:  # never let triage sink the run it is triaging
            print(f"[conformance] cpu-state symbolication failed: {exc!r}", file=sys.stderr)
        print(f"[conformance] guest was WEDGED — vCPU state: {hang_state_path}")
        for line in hang_pcs:
            print(f"    {line}")

    # An `--smp N` sweep that quietly swept the UP kernel is the false green this
    # whole experiment exists to avoid, and it is INVISIBLE in the results — a UP
    # sweep looks like a very good MP sweep. `smoke --smp N` asserts this
    # (_smp_check), but conformance boots via pytest and never went through that
    # path, so it had no such check at all. freeldr announces both, so this is an
    # assertion on evidence we already captured, not new instrumentation.
    if smp:
        from .smoke import _booted_hal, _booted_kernel

        booted_k, booted_h = _booted_kernel(serial_text), _booted_hal(serial_text)
        wrong = [
            f"{what}={got}"
            for what, got, want in (
                ("kernel", booted_k, "ntkrnlmp.exe"),
                ("HAL", booted_h, "halmp.dll"),
            )
            if got and got != want
        ]
        if wrong:
            print(
                f"[conformance] --smp {smp} did NOT get the MP pair: {', '.join(wrong)}.\n"
                f"  This sweep graded the UP kernel and its numbers mean nothing for the\n"
                f"  collapse experiment. Serial: {saved_serial}",
                file=sys.stderr,
            )
            return 2
        print(f"[conformance] MP pair confirmed on serial: {booted_k} + {booted_h}")

    # --- 4. compare against baseline -----------------------------------------
    regressions: list[str] = []
    new_passes: list[str] = []
    have_baseline = baseline_path.is_file()
    if have_baseline:
        base_map = parser.load_baseline(baseline_path)
        regressions = parser.compare(results, base_map)
        for r in results:
            b = base_map.get(r["name"])
            if r["status"] == "OK" and (b is None or b["status"] in parser.BAD):
                new_passes.append(r["name"])

    # --- 5. write baseline if asked ------------------------------------------
    only = getattr(args, "only", None)
    if only and not args.write_baseline:
        print("[conformance] --only has no meaning without --write-baseline", file=sys.stderr)
        return 2
    if only and not have_baseline:
        print(
            f"[conformance] --only needs an existing baseline at {baseline_path};"
            " seed the file first with a plain --write-baseline",
            file=sys.stderr,
        )
        return 2

    if args.write_baseline and only:
        # Targeted re-seed: the coverage refusal below does not apply (nothing is
        # dropped) and the lowering note is scoped to the rows actually touched.
        selected = [r for r in results if r["name"] in set(only)]
        note = baseline_lowering_note(selected, base_map)
        if note:
            print(note)
        merged, refusal = merge_baseline_rows(baseline_path.read_text(), results, only, parser.fmt)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
        baseline_path.write_text(merged)
        print(
            f"[conformance] baseline re-seeded for {len(only)} row(s): {baseline_path}\n"
            f"  {', '.join(only)}"
        )
    elif args.write_baseline:
        # A baseline is a RATCHET FLOOR, and --write-baseline replaces it wholesale
        # with this run's rows. So a sweep that launched fewer tests than the floor
        # — a boot flake, an early BUGCHECK, a drained-out timeout — silently
        # DELETES the missing tests from the floor, permanently, with no signal.
        # That is the ratchet form of "never weaken a test to make it pass", and it
        # is the exact hand-check that had to be done before every re-seed. Refuse
        # instead, and name the tests that would vanish.
        #
        # Only COVERAGE LOSS is refused. A row whose numbers got worse is a
        # judgement call the operator can legitimately accept (i386 and arm64 both
        # deliberately absorbed ExHardError 30/0 -> 24/6), so those are printed
        # loudly and allowed.
        if have_baseline:
            refusal = baseline_coverage_refusal(
                results, base_map, force=getattr(args, "force_baseline", False)
            )
            if refusal:
                print(refusal, file=sys.stderr)
                return 2
            note = baseline_lowering_note(results, base_map)
            if note:
                print(note)
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        smp_arg = f" --smp {smp}" if smp > 1 else ""
        # Rebuild the rows here rather than reusing `lines` (the results artifact):
        # a wholesale re-seed must still carry every `assert?` mark the old file
        # had, or it banks whichever face of a bistable assertion this one boot
        # showed. `lines` is deliberately mark-free — it describes THIS run.
        prev = baseline_path.read_text() if have_baseline else ""
        blines = [parser.fmt(r, _flaky_of(prev, r["name"])) for r in results]
        baseline_path.write_text(
            "# kmtest conformance baseline — parse_kmtest_serial.py format\n"
            f"# arch={base}{f' smp={smp}' if smp > 1 else ''}  generated by"
            f" `vibeos-cli conformance --arch {base}{smp_arg} --write-baseline`\n"
            + "\n".join(blines)
            + "\n"
        )
        print(f"[conformance] baseline written: {baseline_path}")

    # --- 6. summary.json ------------------------------------------------------
    summary = {
        "suite": SUITE,
        "arch": base,
        "pytest_arch": pytest_arch,
        # `smp or 1` used to live here, and it made the ledger LIE about which
        # kernel ran: no --smp boots the UP kernel (ntoskrnl) while `--smp 1`
        # boots the MP kernel on one CPU (ntkrnlmp, PSCI AP bring-up on arm64) —
        # two different binaries doing different work — and both were recorded as
        # smp=1. That collapse cost a whole A/B: 44 UP-kernel rounds at 100% clean
        # were compared against 85 banked MP-at-1-CPU rounds at 49%, and the
        # difference read as a fix when it was a kernel swap. Record 0 for "not
        # specified" and carry the kernel explicitly, so the distinction survives
        # into the shared corpus instead of having to be recovered by grepping
        # serial logs for an `ntkrnlmp.exe` load line.
        "smp": smp,
        # OBSERVED, not inferred. This was `bool(smp)`, which re-committed the very
        # error the note above describes: it read the kernel off the FLAG instead of
        # the boot. That was accurate only while `--smp N` was the only way to get
        # MP media. Since the MP promotion (2026-09-12) a DEFAULT boot runs
        # ntkrnlmp.exe with smp unset, so `bool(smp)` recorded every default MP
        # round as UP -- silently poisoning the one field every flake-rate query
        # filters on. The note above already prescribes the fix ("carry the kernel
        # explicitly"); this is that, and the same expression is computed three
        # lines from here for symbolication.
        "mp_kernel": "ntkrnlmp.exe" in serial_text,
        "launched": len(results),
        "ok": ok,
        "bad": bad,
        "sweep_done": done_line,
        "bugcheck": bugcheck,
        "sweep_rc": sweep_rc,
        "elapsed_s": round(elapsed, 1),
        "have_baseline": have_baseline,
        "regressions": regressions,
        "new_passes": new_passes,
        "results_file": str(results_path),
        "serial_log": str(saved_serial),
        "serial_log_live": str(log_path),  # owned by this sweep
        # Present only when the guest wedged. `hang_pcs` is the payload: on a
        # stall that prints nothing, this is the ONLY statement of where the box
        # actually was, and summary.json is what gets read.
        "hang_cpustate": str(hang_state_path) if hang_state_path.is_file() else None,
        "hang_pcs": hang_pcs,
    }
    (run_dir / "summary.json").write_text(json.dumps(summary, indent=2))

    # --- 6b. durable soak ledger ---------------------------------------------
    # `shape` is the face discriminator, computed from the fields that actually
    # separate them. Note launched==0 vs a stall vs a bugcheck all render the
    # same in a pass/fail view, which is precisely how five distinct bugs spent
    # months filed under one heading.
    serial_text = ""
    try:
        serial_text = Path(summary["serial_log"]).read_text(errors="replace")
    except Exception:
        pass
    if bugcheck:
        shape = f"A:bugcheck {bugcheck}"
    elif not results:
        shape = "B:no-launch"
    elif not done_line:
        shape = "stalled/DID-NOT-FINISH"
    elif regressions:
        shape = "clean-but-ratchet-regression"
    else:
        shape = "clean"
    try:
        from . import triage as _triage

        ev = _triage.wedge_evidence(serial_text)
    except Exception:
        ev = {"census_stuck": 0, "lockdep": 0, "cswitch": 0, "lines": []}
    rec = dict(summary)
    rec.update(
        {
            "ts": time.strftime("%Y%m%d-%H%M%S"),
            "shape": shape,
            "sha": _head_sha(repo),
            "tripwire": os.environ.get("VIBEOS_TRIPWIRE", ""),
            "run_dir": str(run_dir),
            "census_stuck": ev["census_stuck"],
            "lockdep_hits": ev["lockdep"],
            "cswitch_hits": ev["cswitch"],
            "census_lines": ev["lines"],
            # The widened census prints a wait-reason histogram when the parked
            # profile shifts; its presence is what turns face D from invisible
            # into named, so record whether it fired at all.
            "wait_profile": "[CENSUS] wait profile:" in serial_text,
            "rust_panics": serial_text.count("*** RUST PANIC"),
        }
    )
    # Out of /tmp before anything reads it: reboots wipe the artifact root, and a
    # red round's serial + results are the only way to re-grade it later.
    if shape != "clean":
        kept = artifacts.preserve_failure(run_dir, base, shape, repo)
        if kept:
            rec["preserved"] = kept
    _append_round(rec)

    # ...and to the SHARED ledger under vibeos_root, which is what every other
    # command writes and what `vibeos-cli health` reads. `_append_round` above is
    # not a substitute: it writes `~/vibeos-flake-farm/`, OUTSIDE the tree the
    # backups cover. On 2026-09-14 the box was rebuilt, the shared ledger came
    # back with 19,711 rows, and the flake farm did not come back at all -- taking
    # a 201-run arm64 census with it, along with every summary.json it had been
    # reconstructed from. The census had to scrape /tmp in the first place because
    # conformance verdicts were absent from the durable ledger: build 6152,
    # smoke 4741, check 3543, gate 1265, farm 1194, land 1087, conformance ZERO.
    #
    # The fields below are exactly what a census needs, so the next one is a query
    # over a durable file rather than an archaeology dig across volatile run dirs:
    # `shape` separates a lost boot from a flaky assertion (they are two
    # populations and pooling them corrupts the rate), and `regressions` names the
    # tests. Keep this a SUMMARY -- the bulky per-run evidence (census_lines,
    # serial excerpts) stays in the flake farm; this is the part that must survive.
    artifacts.ledger_append(
        repo,
        cmd="conformance",
        arch=base,
        verdict="OK" if shape == "clean" else "FAIL",
        classification=shape,
        duration_ms=int(elapsed * 1000),
        extra={
            "suite": SUITE,
            "smp": smp,
            "mp_kernel": summary["mp_kernel"],
            "launched": len(results),
            "ok": ok,
            "bad": bad,
            "bugcheck": bugcheck,
            "have_baseline": have_baseline,
            "regressions": regressions,
            "new_passes": new_passes,
            # What `flake-report` grades: the round's failure signatures, whether the
            # commit names the code that ran, and where the evidence survives.
            "sigs": flakes.conformance_signatures(rec),
            "dirty": artifacts._is_dirty(repo),
            "run_id": run_dir.name,
            "preserved": rec.get("preserved"),
        },
    )

    # --- 7. report + exit code -----------------------------------------------
    verdict = "done" if done_line else "DID NOT FINISH"
    print(
        f"\nconformance {SUITE} {base}"
        f"{f' -smp {smp}' if smp else ''}  —  "
        f"launched={len(results)} ok={ok} bad={bad} sweep={verdict}"
        f"{' BUGCHECK=' + bugcheck if bugcheck else ''}  ({elapsed:.0f}s)"
    )
    print(f"  results:  {results_path}")
    print(f"  summary:  {run_dir / 'summary.json'}")
    if bugcheck or bad:
        print(f"  serial:   {saved_serial}")

    if not have_baseline:
        print(f"  no baseline at {baseline_path} — pass --write-baseline to seed it.")
        return 0
    if new_passes:
        print(f"  NEW PASSES ({len(new_passes)}): {', '.join(new_passes)}")
    if regressions:
        print(f"  RATCHET REGRESSIONS ({len(regressions)}):")
        for reg in regressions:
            print(f"    {reg}")
        if args.gate:
            print("  === CONFORMANCE GATE RED ===")
            return 1
        print("  (report-only; pass --gate to fail the command on regressions)")
        return 0
    print("  ratchet holds: no regressions vs baseline.")
    return 0
