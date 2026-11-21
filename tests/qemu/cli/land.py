"""`vibeos-cli land --direct`: the integrator's landing step. Merge origin/master into THIS
tree, gate it, push it, all under the fleet-wide land lock.

Lanes do not land themselves. They commit and message the integration session
(`merge <branch> @ <sha> — <what>`; docs/engineering/active-lanes/integration.md), which
merges a batch of lanes into its `integration` branch and runs this. Plain `land` prints
that message, ready to send, and refuses: the merge train it used to start (a queue, and
whichever lane found the lock conducting it) was held from 2026-10-16 and retired on
2026-11-08, because one integrator gating N lanes per batch beat it on every measure the
integration brief records.

THE RACE THIS PATH REMOVED (2026-07-25). Before it, landing was: merge origin/master →
commit → `vibeos-cli gate` → `git push`. The gate mints a push credential keyed to an exact
TREE HASH, so everything between "gate green" and "push accepted" was a race: a lane whose
push lost to another landing had to merge, which changed the tree, which voided the stamp,
which meant a whole new gate, and past a couple of active lanes that was a congestion
collapse. So landing is mutual exclusion. The land lock (hostlock.land_lock) is held
across the WHOLE critical section:

    fetch → merge origin/master → gate → push HEAD:master → release

and master cannot move under a gating tree; a green gate always lands. With one integrator
the lock is rarely contended, but it stays: it costs nothing, and it is what makes a second
lander (a lane on an old commit, a human) safe rather than a race.

Two deliberate design points:

  * **Gate INSIDE the lock, don't pre-gate.** If a fresh green full-gate stamp already
    covers the exact tree being pushed, or the delta is docs-only, the gate is skipped.
  * **Push `HEAD:master`, never move the local `master` ref.** Force-updating `master`
    from a worktree that doesn't have it checked out leaves that worktree's index+files at
    the old tree, which resurfaces as a giant phantom "staged revert" (2026-07-25: 61
    commits of landed work shown as a 197K-line deletion). Pushing the ref straight to
    origin cannot do that; every worktree picks it up via its own `git merge origin/master`.

The pre-push hook stays as the backstop: it refuses a master update whose tree has no fresh
green stamp, so a hand-rolled `git push` is no less safe than this.

The old queue's HOLD.json is deliberately left in the shared git dir: a lane on a commit
from before the retirement still runs the old CLI, and the hold is what stops that CLI
conducting a train. This code never reads it.
"""

from __future__ import annotations

import contextlib
import functools
import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

from . import hostlock, warmth
from .artifacts import Run, Stage

STAMP_MAX_AGE_S = 24 * 3600
FULL_ARCHES = {"i386", "amd64", "arm64"}
INTEGRATION_BRANCH = "integration"

# A network blip, not an answer from the remote. The router-first resolver here drops
# lookups now and then; one `Could not resolve hostname github.com` in a fetch killed a
# landing twice in one night (2026-10-15).
TRANSIENT_NETWORK = re.compile(
    r"Could not resolve hostname|Temporary failure in name resolution|"
    r"Connection (timed out|reset|refused|closed)|Network is unreachable|"
    r"Could not resolve host|Operation timed out|early EOF|unexpected disconnect"
)
NETWORK_COMMANDS = ("fetch", "push", "ls-remote", "pull")
NETWORK_RETRY_DELAYS = (5, 15, 30, 60)


def run_git(repo: Path, args: tuple[str, ...], timeout: float) -> subprocess.CompletedProcess:
    """`git <args>`; a network command that fails on a TRANSIENT_NETWORK error is retried
    after each of NETWORK_RETRY_DELAYS. Anything the remote actually said (a rejected
    push, a missing ref) is returned at once."""
    for delay in (*NETWORK_RETRY_DELAYS, None):
        cp = subprocess.run(
            ["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=timeout
        )
        if (
            cp.returncode == 0
            or delay is None
            or not args
            or args[0] not in NETWORK_COMMANDS
            or not TRANSIENT_NETWORK.search(cp.stderr)
        ):
            return cp
        time.sleep(delay)
    raise AssertionError("unreachable")


class LandAbort(RuntimeError):
    """A precondition failed or a step could not be completed. Carries the operator-facing
    reason; `land()` turns it into a fail Stage + nonzero exit (never a silent skip)."""


def _git(repo: Path, *args: str, check: bool = True) -> str:
    cp = run_git(repo, args, 600)
    if check and cp.returncode != 0:
        raise LandAbort(f"git {' '.join(args)} failed ({cp.returncode}): {cp.stderr.strip()}")
    return cp.stdout.strip()


def _git_ok(repo: Path, *args: str) -> bool:
    return (
        subprocess.run(
            ["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=600
        ).returncode
        == 0
    )


def _stamps_dir(repo: Path) -> Path:
    common = Path(_git(repo, "rev-parse", "--git-common-dir"))
    if not common.is_absolute():
        common = (repo / common).resolve()
    return common


def fresh_full_stamp(repo: Path, tree: str, *, now: float | None = None) -> dict | None:
    """The gate stamp validating exactly `tree`, if one is fresh (<24h) and full-arch.

    Same acceptance rule as the pre-push hook — deliberately duplicated rather than
    imported, because the hook is a standalone script that must keep working on a clone
    whose python can't import this package. Kept in sync by test_cli_units."""
    now = time.time() if now is None else now
    common = _stamps_dir(repo)
    candidates = []
    d = common / "vibeos-gate-stamps"
    if d.is_dir():
        candidates.extend(sorted(d.glob("*.json")))
    legacy = common / "vibeos-gate-stamp.json"
    if legacy.is_file():
        candidates.append(legacy)
    for p in candidates:
        try:
            s = json.loads(p.read_text())
        except (OSError, ValueError):
            continue
        if s.get("tree") != tree:
            continue
        if now - s.get("time", 0) > STAMP_MAX_AGE_S:
            continue
        if not FULL_ARCHES.issubset(set(s.get("arches", []))):
            continue
        return s
    return None


def _preflight(repo: Path) -> tuple[str, str]:
    """Refuse to start unless the tree can produce a stampable push. Returns (branch, head)."""
    branch = _git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    if branch == "HEAD":
        raise LandAbort("detached HEAD — check out the lane branch you want to land")
    dirty = _git(repo, "status", "--porcelain", "--untracked-files=no")
    if dirty:
        n = len(dirty.splitlines())
        raise LandAbort(
            f"{n} uncommitted change(s) — commit (or `git stash push -m ...`) first; the "
            "gate stamp must hash the exact content being pushed, so a dirty tree can "
            "never mint one"
        )
    return branch, _git(repo, "rev-parse", "HEAD")


def _docs_only_vs_master(repo: Path) -> bool:
    """Does this lane change NOTHING but docs, measured against the merge base with master?

    The pre-push hook has carried this exemption since it was written -- a docs-only delta
    needs no fresh gate stamp -- but the land preflights did not, so a commit touching one
    `.md` paid a full `check-tooling` plus both strict source-lint partitions. A docs-only
    change cannot fail a Python linter or a source scanner, because it contains no Python
    and no source.

    It bit the change that recorded the orphan inventory: refused at
    `test_verify_ucrtbase_cohab.py` over a stale `build-i386` in the author's worktree,
    for a delta of one Markdown file.

    Skipping here is safe in the direction that matters. The CONDUCTOR still runs every
    check on the merged batch, so this cannot put unlinted code on master -- it only
    declines to lint a lane that has none. The predicate is `tools/docs_only.py`, shared
    with the pre-push hook: two spellings of "docs-only" that can disagree is worse than
    neither, so there is exactly one.

    Uses the three-dot merge base, so a lane that is merely BEHIND master is still judged
    on what IT changed rather than on what landed while it worked. Fails CLOSED -- any git
    error returns False and the lane lints normally.
    """
    try:
        diff = _git(repo, "diff", "--name-only", "origin/master...HEAD")
    except Exception:
        return False
    module = _docs_only_module(repo)
    return bool(module and module.docs_only(diff.splitlines()))


def _docs_only_module(repo: Path):
    """`tools/docs_only.py` from THIS worktree, or None.

    By PATH, not by name: a `sys.path` insert leaves `docs_only` in `sys.modules`, and a
    process that later conducts a drain would then judge the INTEGRATION tree with this
    lane's copy of the rule."""
    import importlib.util

    try:
        spec = importlib.util.spec_from_file_location(
            "_land_docs_only", repo / "tools" / "docs_only.py"
        )
        if spec is None or spec.loader is None:
            return None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception:
        return None
    return module


GATE_ARCHES = ("i386", "amd64", "arm64")


def _warm_preflight(repo: Path, run: Run, args) -> None:
    """Refuse a lane whose code no build has seen, so the gate would compile it cold.

    See warmth.py for the incident. Cheap (a few git calls). Skipped for a docs-only lane,
    which the landing gate does not gate at all, and overridable with `--cold` for the rare
    lane that means to land cold anyway -- report every use, since that lane's gate can
    hold the integrator's batch for hours. Fails OPEN when the receipts cannot be read: this is a
    courtesy to the fleet, not a correctness check, and the gate itself still judges."""
    if getattr(args, "cold", False):
        run.add(
            Stage("warm", status="warn", detail="--cold: gate may compile this lane from scratch")
        )
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("warm", status="skip", detail="docs-only delta"))
        return
    module = _docs_only_module(repo)
    if module is None:
        run.add(Stage("warm", status="warn", detail="docs_only predicate unavailable; not checked"))
        return
    try:
        tree = _git(repo, "rev-parse", "HEAD^{tree}")
        master = _git(repo, "rev-parse", "origin/master^{tree}")
        cold = warmth.missing(repo, tree, GATE_ARCHES, module.is_inert, master=master)
    except Exception as exc:
        run.add(Stage("warm", status="warn", detail=f"could not check receipts: {exc}"))
        return
    if not cold:
        run.add(Stage("warm", status="ok", detail="built on all three arches"))
        return
    raise LandAbort(
        f"this tree has not been BUILT on {', '.join(cold)} — the landing gate builds "
        "cold and leans on the shared compiler caches, so code no build has seen compiles "
        "from scratch there and can hold the whole batch for hours (upstream-sync, "
        "2026-09-26). Build first, which warms the cache for the gate:\n    "
        + "\n    ".join(f"./vibeos-cli build --arch {a}" for a in cold)
        + "\n  then `./vibeos-cli land` again. Commit exactly what you built: the receipt is "
        "keyed on the tree's content (docs, and tools/ or tests/ Python the build never runs, "
        "may differ).\n"
        "  override (report every use): ./vibeos-cli land --cold"
    )


def _merged_labels(merged: str) -> tuple[str, str]:
    """(stage/artifact suffix, message clause) for a preflight run on the PREMERGE tree."""
    if not merged:
        return "", ""
    return "@merged", f" on this lane MERGED with origin/master {merged[:12]}"


def _merged_hint(merged: str) -> str:
    if not merged:
        return ""
    return (
        "\n  your tip alone may be clean: master moved under the lane (a regenerated table, a"
        "\n  new baseline entry, a renamed helper). `git fetch && git merge origin/master`,"
        "\n  reproduce there, fix, rebuild, and `land` again."
    )


@contextlib.contextmanager
def _premerge_worktree(repo: Path, run: Run, upstream: str = "origin/master"):
    """Yield (worktree, master_sha) holding THIS lane merged with origin/master, or None.

    The tip preflights below are an approximation in one direction: they lint the lane
    tip, while the landing gate lints the tip MERGED with master. Whatever master changed
    since the lane last merged is invisible to them, and that is the commonest way a lane
    with a clean tip still bounces: on 2026-10-16 native-batch's tip passed check-tooling,
    then lost a 75-minute gate to `test_gen_apisets_table` because master had edited
    `apisets.table.c` and the committed generated table was now stale.

      * Lane already contains origin/master → None: the tip IS the merged tree.
      * `git merge-tree` reports a conflict → LandAbort now, with the paths. The landing gate
        would hit the same CONFLICT, but only after a merge attempt.
      * Otherwise the merged tree is checked out DETACHED into a scratch worktree under
        the shared git dir (one per lane and pid; ~1.3G, no build dirs) and removed after.

    Fail OPEN like every other preflight: if git cannot produce the tree, warn and let
    the landing gate be the oracle. It mutates nothing of the lane's: the merge commit is
    an unreferenced object and the worktree is this process's own.
    """
    try:
        master = _git(repo, "rev-parse", upstream)
    except LandAbort as exc:
        run.add(Stage("premerge", status="warn", detail=f"no {upstream}: {exc}"[:300]))
        yield None
        return
    if _git_ok(repo, "merge-base", "--is-ancestor", master, "HEAD"):
        run.add(Stage("premerge", status="skip", detail="lane already contains origin/master"))
        yield None
        return
    cp = run_git(
        repo, ("merge-tree", "--write-tree", "--name-only", "--no-messages", "HEAD", master), 600
    )
    lines = cp.stdout.split("\n")
    if cp.returncode == 1:
        paths = [ln for ln in lines[1:] if ln.strip()]
        run.classification = "CONFLICT"
        raise LandAbort(
            f"this lane CONFLICTS with origin/master {master[:12]} — the integrator would have to "
            "resolve it for you:\n    "
            + "\n    ".join(paths[:20])
            + "\n  `git fetch && git merge origin/master`, resolve, rebuild, `land` again."
        )
    if cp.returncode != 0 or not lines[0].strip():
        run.add(Stage("premerge", status="warn", detail=f"merge-tree failed: {cp.stderr[-200:]}"))
        yield None
        return
    commit = _git(
        repo,
        "commit-tree",
        lines[0].strip(),
        "-p",
        "HEAD",
        "-p",
        master,
        "-m",
        "land preflight: lane merged with origin/master",
    )
    root = _stamps_dir(repo) / "vibeos-premerge"
    root.mkdir(exist_ok=True)
    _sweep_premerge(repo, root)
    wt = root / f"{repo.resolve().name}-{os.getpid()}"
    try:
        _git(repo, "worktree", "add", "--detach", str(wt), commit)
    except LandAbort as exc:
        run.add(Stage("premerge", status="warn", detail=f"could not check out: {exc}"[:300]))
        yield None
        return
    try:
        _provision_premerge(repo, wt)
    except (OSError, subprocess.SubprocessError, RuntimeError) as exc:
        run_git(repo, ("worktree", "remove", "--force", str(wt)), 600)
        shutil.rmtree(wt, ignore_errors=True)
        run.add(Stage("premerge", status="warn", detail=f"could not provision: {exc}"[:300]))
        yield None
        return
    run.add(Stage("premerge", status="ok", detail=f"lane + origin/master {master[:12]} at {wt}"))
    try:
        yield wt, master
    finally:
        run_git(repo, ("worktree", "remove", "--force", str(wt)), 600)
        shutil.rmtree(wt, ignore_errors=True)


def _provision_premerge(repo: Path, wt: Path) -> None:
    """What a fresh lane gets before its first check-tooling, minus the builds.

    Measured on a bare checkout (2026-10-16): every tooling check passes except the four
    `test_artifact_*` suites, which need the pinned toolchain and the `host-jobs` binary,
    and `pytest:tools/rustconv/tests`, which reads build-<arch> (`--skip-generated`).
    The toolchain is content-addressed and commit-independent, the one thing the fleet
    isolation law lets a worktree share, so it is linked to the lane's own target."""
    from . import hostjobs

    lane_tc = repo / ".build-tools" / "llvm-mingw"
    if lane_tc.exists():
        link = wt / ".build-tools" / "llvm-mingw"
        link.parent.mkdir(parents=True, exist_ok=True)
        link.symlink_to(lane_tc.resolve())
    hostjobs.ensure(wt)


def _sweep_premerge(repo: Path, root: Path) -> None:
    """Remove premerge worktrees whose owning `land` died before its `finally` ran."""
    for d in root.iterdir():
        pid = d.name.rsplit("-", 1)[-1]
        if not pid.isdigit() or Path(f"/proc/{pid}").exists():
            continue
        run_git(repo, ("worktree", "remove", "--force", str(d)), 600)
        shutil.rmtree(d, ignore_errors=True)
    run_git(repo, ("worktree", "prune"), 600)


def _lint_preflight(repo: Path, run: Run, args, *, merged: str = "") -> None:
    """Run `check-tooling` HERE, before this lane consumes a batched gate.

    The landing gate already runs this exact command, so this can never block anything the
    gate would have let through — it is the same verdict, moved from the most expensive
    place in the system to the cheapest. That matters because a batched gate is 3 builds
    + 4 boots of fleet load, and a red one bounces the whole train: the lint that
    rejected this lane on 2026-07-26 was a single line 101 characters long.

    Three properties make the move sound:
      * `check-tooling` is arch-independent, host-only and deterministic (gate.py's
        `_tooling_once` says so itself, and starts it before the first build);
      * it needs no build-<arch>, so no fleet-isolation question arises;
      * failing here is strictly earlier than failing there — never *more* strict.

    It runs twice when the lane is behind master: on the tip here, and with `merged=` on
    the lane MERGED with origin/master (`_premerge_worktree`), which is what the landing gate
    actually gates. The tip run alone missed every failure the merge introduces.

    FAIL OPEN if the check itself cannot run (missing venv, no ruff, timeout). A broken
    preflight must never be able to wedge the fleet's ability to land; only a check that
    ran and returned nonzero blocks.
    """
    tag, where = _merged_labels(merged)
    if getattr(args, "skip_lint", False):
        run.add(Stage(f"lint{tag}", status="skip", detail="--skip-lint: check-tooling not run"))
        return
    if _docs_only_vs_master(repo):
        run.add(
            Stage(f"lint{tag}", status="skip", detail="docs-only delta: nothing for it to lint")
        )
        return
    t0 = time.monotonic()
    try:
        cp = subprocess.run(
            # The premerge tree has no build-<arch>: skip only the suites that read one
            # (they still run on the tip, above, and in the landing gate).
            [str(repo / "vibeos-cli"), "check-tooling", *(["--skip-generated"] if merged else [])],
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=1800,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        run.add(Stage(f"lint{tag}", status="warn", detail=f"check-tooling could not run: {exc}"))
        return
    dt = time.monotonic() - t0
    if cp.returncode and any(
        line.startswith("DISK_ADMISSION_FAIL:") for line in (cp.stdout or "").splitlines()
    ):
        run.classification = "DISK_ADMISSION_FAIL"
        raise LandAbort(cp.stdout.strip())
    if cp.returncode == 0:
        run.add(Stage(f"lint{tag}", status="ok", detail=f"check-tooling clean ({dt:.0f}s)"))
        return
    # PERSIST what we captured before aborting. check-tooling prints the failing
    # test's own output after the summary, and this is the only copy of it: the
    # child writes no artifact dir, so discarding stdout here leaves a lane with a
    # ✗ line and nothing to debug. That is not hypothetical — a preflight abort on
    # 2026-09-06 named only `pytest:.` for a failure that did not reproduce, and
    # the evidence was gone.
    log = run.dir / f"preflight-check-tooling{tag}.log"
    try:
        log.write_text((cp.stdout or "") + (cp.stderr or ""), encoding="utf-8", errors="replace")
    except OSError:
        pass
    salient = [ln.strip() for ln in (cp.stdout or "").splitlines() if "✗" in ln]
    raise LandAbort(
        f"check-tooling FAILED{where} — fix it first, or the landing gate goes red for the whole "
        "batch (3 builds + 4 boots of fleet load) for it:\n    "
        + ("\n    ".join(salient) if salient else f"exit {cp.returncode}")
        + f"\n  captured output: {log}"
        + "\n  run `./vibeos-cli check-tooling` for the detail."
        + _merged_hint(merged)
        + "\n  override (report every use): ./vibeos-cli land --skip-lint"
    )


def _source_lint_preflight(repo: Path, run: Run, args, *, merged: str = "") -> None:
    """Run the landing gate's OWN source-lint partitions here, before the batched gate.

    `_lint_preflight` above is named "lint" but only ever ran `check-tooling`. The gate
    runs that AND `check --strict --scope source-early|source-deferred`, and those two
    are the other cheap half — pure source, no build, no boot. On 2026-09-19 they
    rejected the same lane twice (`scan_unpruned_walk`, a one-line fix each time) after
    the full queue wait, while `./vibeos-cli check` stayed green the whole time, because
    plain `check` does not run those partitions in STRICT mode. A gate downstream of the
    cost is not a gate.

    Uses gate's `_lint_once` rather than re-deriving the command, so this cannot drift
    from what the landing gate actually runs.

    NEVER MORE STRICT THAN THE GATE, which is the property that makes moving it sound:
    a partition that could not RUN here must not block. `source-deferred` contains
    `verify_signatures` and `gen_ctype_tables`, which hard-fail without this worktree's
    `build-<arch>` — a fresh lane legitimately has none. `_lint_once` already separates
    those: a crash, or a nonzero exit with no parseable finding lines, classifies as
    SCANNER_ERROR, and only a real NEW-findings verdict blocks. Everything else warns,
    matching the fail-open rule the check-tooling half already follows.
    """
    tag, where = _merged_labels(merged)
    if getattr(args, "skip_lint", False):
        return
    if _docs_only_vs_master(repo):
        for part in ("source-early", "source-deferred"):
            run.add(Stage(f"lint:{part}{tag}", status="skip", detail="docs-only delta"))
        return
    from .gate import _lint_once

    for part in ("source-early", "source-deferred"):
        try:
            result = _lint_once(repo, part, run.dir / f"preflight-lint-{part}{tag}")
        except (OSError, subprocess.SubprocessError) as exc:
            run.add(Stage(f"lint:{part}{tag}", status="warn", detail=f"could not run: {exc}"))
            continue
        if result["ok"]:
            run.add(
                Stage(f"lint:{part}{tag}", status="ok", detail=f"clean ({result['seconds']:.0f}s)")
            )
            continue
        if result["classification"] == "SCANNER_ERROR":
            # Infrastructure, not a finding — the landing gate has a built tree and we may
            # not. Say so and let the gate be the oracle.
            #
            # BUT ONLY FOR THE SCANNER THAT CRASHED. A partition is one classification
            # for many scanners, and SCANNER_ERROR from one of them used to wave through
            # a real NEW finding from another: on 2026-09-26 msvcrt-homebrew's preflight
            # reported `scan_const_contradictions: 4 new` right beside a marshalgen crash
            # (a stale build dir), submitted three times on "could not complete", and the
            # landing gate caught the finding ~80 minutes later.
            real = _real_findings(run.dir / f"preflight-lint-{part}{tag}")
            if real:
                raise LandAbort(
                    f"check --strict --scope {part}{where} found NEW findings (another scanner in "
                    "the partition also crashed, which does not excuse these):\n    "
                    + "\n    ".join(real[:8])
                    + f"\n  artifact: {run.dir / f'preflight-lint-{part}{tag}'}"
                    + f"\n  reproduce: ./vibeos-cli check --strict --scope {part}"
                    + _merged_hint(merged)
                    + "\n  override (report every use): ./vibeos-cli land --skip-lint"
                )
            run.add(
                Stage(
                    f"lint:{part}{tag}",
                    status="warn",
                    detail=f"could not complete here ({result['salient'] or 'scanner error'})",
                )
            )
            continue
        raise LandAbort(
            f"check --strict --scope {part} FAILED{where} — fix it first, or the "
            "landing gate goes red for the whole batch (3 builds + 4 boots of fleet load) for a "
            "source lint:\n    "
            + (result["salient"] or f"exit {result['exit']}")
            + f"\n  artifact: {run.dir / f'preflight-lint-{part}{tag}'}"
            + f"\n  reproduce: ./vibeos-cli check --strict --scope {part}"
            + _merged_hint(merged)
            + "\n  override (report every use): ./vibeos-cli land --skip-lint"
        )


def _arch_lint_preflight(repo: Path, run: Run, args) -> None:
    """Run the landing gate's per-arch BINARY lints here, for every arch this lane has built.

    The two source partitions above cannot see `check --scope arch`: scan_ud2,
    scan_self_recursive, scan_flex_array_panic and scan_export_thunks read the OBJECT
    CODE, so they only exist once a build does. That left one deterministic LINT_FAIL
    class that a lane could not catch before submitting: on 2026-10-08 a lane that
    stopped building a crate left its `tools/flex_array_allowlist.txt` entry behind,
    the scanner reported the entry STALE on all three arches, and the batch went red
    (queue-batch-01791509613339746824) — after three cold builds and the smokes. The
    lane had built every arch to smoke it; the scanner just never ran on those builds.

    Same contract as the other preflights: NEVER MORE STRICT THAN THE GATE. It runs
    `_lint_once(repo, <arch>, …)`, the gate's own call, and only for an arch whose
    `build-<arch>` is FRESH for this tree (provenance says so, the same oracle `smoke`
    uses to refuse stale media) — a stale build would grade a tree that is not the one
    being submitted. No build, a stale one, or a scanner crash all WARN and let the
    landing gate be the oracle; only a scanner that ran on a fresh build and reported a
    finding blocks.
    """
    if getattr(args, "skip_lint", False):
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("lint:arch", status="skip", detail="docs-only delta"))
        return
    import concurrent.futures as cf

    from . import archcfg, provenance
    from .gate import _lint_once

    # The three arches are independent read-only passes (the gate runs them the same way,
    # alongside its smokes), and scan_flex_array_panic alone is ~2 min per arch: serial,
    # this preflight was 16 of a direct landing's 16 minutes. Only the subprocesses
    # overlap; stages are still added in ARCHES order so the artifact reads the same.
    def one(arch: str) -> tuple[str, provenance.Staleness | dict | None, Exception | None]:
        st = provenance.check_staleness(repo, arch)
        if not st.fresh:
            return (arch, st, None)
        try:
            return (arch, _lint_once(repo, arch, run.dir / f"preflight-lint-{arch}"), None)
        except (OSError, subprocess.SubprocessError) as exc:
            return (arch, None, exc)

    with cf.ThreadPoolExecutor(max_workers=len(archcfg.ARCHES)) as ex:
        outcomes = dict(zip(archcfg.ARCHES, ex.map(one, archcfg.ARCHES), strict=True))

    for arch in archcfg.ARCHES:
        _arch, got, exc = outcomes[arch]
        if isinstance(got, provenance.Staleness):
            run.add(
                Stage(
                    f"lint:{arch}",
                    status="warn",
                    detail="skipped: build-"
                    + arch
                    + " not fresh for HEAD ("
                    + "; ".join(got.reasons[:2])
                    + ") — the landing gate runs it",
                )
            )
            continue
        if exc is not None or got is None:
            run.add(Stage(f"lint:{arch}", status="warn", detail=f"could not run: {exc}"))
            continue
        result = got
        if result["ok"]:
            run.add(Stage(f"lint:{arch}", status="ok", detail=f"clean ({result['seconds']:.0f}s)"))
            continue
        if result["classification"] == "SCANNER_ERROR":
            real = _real_findings(run.dir / f"preflight-lint-{arch}")
            if not real:
                run.add(
                    Stage(
                        f"lint:{arch}",
                        status="warn",
                        detail=f"could not complete here ({result['salient'] or 'scanner error'})",
                    )
                )
                continue
            salient = "\n    ".join(real[:8])
        else:
            real = _real_findings(run.dir / f"preflight-lint-{arch}")
            salient = (
                "\n    ".join(real[:8]) if real else (result["salient"] or f"exit {result['exit']}")
            )
        raise LandAbort(
            f"check --strict --scope arch --arch {arch} FAILED on this lane's own build — "
            "fix it first, or the landing gate goes red for the whole batch (3 cold "
            "builds + the smokes) for a binary lint:\n    "
            + salient
            + f"\n  artifact: {run.dir / f'preflight-lint-{arch}'}"
            + f"\n  reproduce: ./vibeos-cli check --strict --scope arch --arch {arch}"
            + "\n  override (report every use): ./vibeos-cli land --skip-lint"
        )


def _real_findings(root: Path) -> list[str]:
    """Scanners in a partition run under `root` that FAILED WITH FINDINGS, as opposed to
    crashing. A crash reports `SCANNER_ERROR: ...` in its stage detail; anything else
    that failed is a verdict. Unreadable summaries yield [] (fail open, as before)."""
    out: list[str] = []
    for summary in sorted(root.glob("**/summary.json")):
        try:
            stages = json.loads(summary.read_text()).get("stages", [])
        except (OSError, ValueError):
            continue
        for st in stages:
            detail = str(st.get("detail", ""))
            if st.get("status") != "fail" or detail.startswith("SCANNER_ERROR"):
                continue
            first = (st.get("salient") or [""])[0]
            out.append(f"{st.get('name')}: {detail}" + (f" — {first}" if first else ""))
    return out


def _concurrent_preflights(repo: Path, run: Run, args) -> None:
    """Run the four build-independent preflights at once, the way the gate runs its
    copies of them. Serially they cost a lane ~15 min before `submit` (check-tooling
    504s, source lints 9s + 148s, per-arch binary lints 163s, difftest 92s, measured
    2026-10-09); none depends on another's result, so the wall is the slowest one.
    Every abort is collected rather than the first one winning, so a lane fixes all
    of them in one pass instead of discovering them one resubmission at a time."""
    import concurrent.futures as cf

    steps = (_lint_preflight, _source_lint_preflight, _arch_lint_preflight, _difftest_preflight)
    aborts: list[str] = []
    with contextlib.ExitStack() as stack:
        premerge = None
        if not getattr(args, "skip_lint", False) and not _docs_only_vs_master(repo):
            premerge = stack.enter_context(_premerge_worktree(repo, run))
        calls = [functools.partial(step, repo, run, args) for step in steps]
        if premerge:
            # The build-independent pair again, on what the landing gate will actually gate.
            # Tooling receipts live in the shared git dir and are keyed on content, so
            # only the checks whose inputs the merge changed really re-run.
            wt, master = premerge
            calls += [
                functools.partial(step, wt, run, args, merged=master)
                for step in (_lint_preflight, _source_lint_preflight)
            ]
        with cf.ThreadPoolExecutor(max_workers=len(calls)) as ex:
            for fut in [ex.submit(call) for call in calls]:
                try:
                    fut.result()
                except LandAbort as exc:
                    aborts.append(str(exc))
    if aborts:
        raise LandAbort("\n\n".join(aborts))


def _difftest_preflight(repo: Path, run: Run, args) -> None:
    """Run the gate's host differential harnesses HERE, before this lane is batched.

    The other cheap, build-free, boot-free third of the gate. Without it, on 2026-09-25 a
    lane that deleted a Microsoft source file its own difftest still linked against went
    through lint preflight green, then turned SEVENTEEN of eighteen batched gates red over
    one evening -- splitting every batch it rode in, and (through a stale cargo cache in
    the integration tree) poisoning batches it was not even in. ~30s here would have
    stopped it at the door.

    Uses gate's `_difftest_once`, so this cannot drift from what the landing gate runs. Same
    rules as the two lint preflights: never stricter than the gate (it is the same
    command), approximate only in that it tests the lane tip rather than tip+master, and
    FAIL OPEN when the harness cannot run at all -- only a harness that ran and reported a
    divergence blocks.
    """
    if getattr(args, "skip_lint", False):
        run.add(Stage("difftest", status="skip", detail="--skip-lint: difftest not run"))
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("difftest", status="skip", detail="docs-only delta"))
        return
    from .gate import _difftest_once

    root = run.dir / "preflight-difftest"
    try:
        result = _difftest_once(repo, root)
    except (OSError, subprocess.SubprocessError) as exc:
        run.add(Stage("difftest", status="warn", detail=f"could not run: {exc}"))
        return
    if result["ok"]:
        run.add(Stage("difftest", status="ok", detail=f"clean ({result['seconds']:.0f}s)"))
        return
    raise LandAbort(
        "difftest FAILED — fix it first, or the landing gate goes red for the whole batch "
        "(3 builds + 4 boots of fleet load) for it:\n    "
        + (result["salient"] or f"exit {result['exit']}")
        + f"\n  console: {root / 'difftest-console.log'}"
        + "\n  reproduce: ./vibeos-cli difftest"
        + "\n  override (report every use): ./vibeos-cli land --skip-lint"
    )


def _merge_master(repo: Path, run: Run) -> str:
    """Merge origin/master into HEAD (no-op when already contained). Returns the new HEAD.
    A conflicting merge is aborted and reported — resolving it is the lane's call, not
    something to do while holding the fleet's land lock."""
    remote = _git(repo, "rev-parse", "origin/master")
    if _git_ok(repo, "merge-base", "--is-ancestor", remote, "HEAD"):
        run.add(Stage("merge", status="ok", detail=f"already contains origin/master {remote[:12]}"))
        return _git(repo, "rev-parse", "HEAD")
    cp = subprocess.run(
        ["git", "merge", "--no-edit", remote],
        cwd=str(repo),
        capture_output=True,
        text=True,
        timeout=900,
    )
    if cp.returncode != 0:
        conflicts = _git(repo, "diff", "--name-only", "--diff-filter=U", check=False)
        subprocess.run(["git", "merge", "--abort"], cwd=str(repo), capture_output=True, text=True)
        raise LandAbort(
            "merge of origin/master CONFLICTS (aborted, tree restored) — resolve it, "
            f"commit, then re-run land. Conflicts:\n    {conflicts or cp.stderr.strip()}"
        )
    head = _git(repo, "rev-parse", "HEAD")
    run.add(Stage("merge", status="ok", detail=f"merged origin/master {remote[:12]} → {head[:12]}"))
    return head


def _run_gate(repo: Path, run: Run, args) -> None:
    """Run the FULL gate as a subprocess and require exit 0. Full arch set only: a
    partial gate never mints the stamp, so it could not land anyway."""
    cmd = [str(repo / "vibeos-cli"), "gate", "--retries", str(getattr(args, "retries", 2))]
    if getattr(args, "clean", False):
        cmd.append("--clean")
    t0 = time.monotonic()
    cp = run.run(cmd, cwd=repo, stage="gate", env=dict(os.environ, VIBEOS_JOB_CLASS="integration"))
    dt = time.monotonic() - t0
    if cp.returncode != 0:
        run.add(
            Stage(
                "gate",
                exit=cp.returncode,
                seconds=dt,
                status="fail",
                detail="gate RED — nothing pushed",
                salient=[f"vibeos-cli gate exited {cp.returncode}; see its own artifact dir"],
            )
        )
        raise LandAbort("gate failed — fix the regression, then re-run land (nothing pushed)")
    run.add(Stage("gate", seconds=dt, status="ok", detail="all arches green"))


def _push(repo: Path, run: Run, head: str) -> None:
    """Fast-forward origin/master to HEAD. We hold the land lock, so this cannot lose a
    race; a non-fast-forward here would mean somebody bypassed the lock, and the loud
    failure is the correct outcome."""
    remote = _git(repo, "rev-parse", "origin/master")
    if not _git_ok(repo, "merge-base", "--is-ancestor", remote, head):
        raise LandAbort(
            f"origin/master ({remote[:12]}) is NOT an ancestor of HEAD ({head[:12]}) — "
            "somebody landed while we held the land lock (bypassed it?). Re-run land."
        )
    cp = run.run(["git", "push", "origin", f"{head}:refs/heads/master"], cwd=repo, stage="push")
    if cp.returncode != 0:
        run.add(
            Stage(
                "push",
                exit=cp.returncode,
                status="fail",
                detail="push rejected",
                salient=[(cp.stderr or cp.stdout or "").strip().splitlines()[-1:][0]]
                if (cp.stderr or cp.stdout)
                else [],
            )
        )
        raise LandAbort("push to origin/master rejected — see the push stage output")
    run.add(Stage("push", status="ok", detail=f"origin/master → {head[:12]}"))


def _report_queue(run: Run) -> None:
    holder = hostlock.land_holder()
    waiters = hostlock.land_queue()
    if holder:
        run.log(
            f"another lane is landing right now: pid={holder.get('pid')} "
            f"{holder.get('kind', '?')} in {holder.get('cwd', '?')}"
            + (f"; {len(waiters)} already queued ahead-or-with us" if waiters else ""),
            header="land-queue",
        )


def land(args, repo: Path) -> int:
    """`vibeos-cli land`. `--direct` is the integrator's landing; anything else is a lane
    that wants to land, and gets the message to send instead."""
    if not getattr(args, "direct", False):
        return _merge_request(repo)
    return _land_direct(args, repo)


def _merge_request(repo: Path) -> int:
    """Print the merge request for this lane's tip and refuse (exit 2).

    Refuse rather than land: a lane that lands itself is the second pusher the integrator
    exists to remove. Print the message rather than make the lane remember its shape, and
    fill in what the lane would otherwise look up by hand (and could misquote: a sha sent
    before the commit that prints it is how a merge request names the wrong tree)."""
    try:
        branch, head = _preflight(repo)
    except LandAbort as exc:
        print(f"LAND_REFUSED: {exc}")
        return 2
    run_git(repo, ("fetch", "origin"), 600)
    if _git_ok(repo, "merge-base", "--is-ancestor", head, "origin/master"):
        print(f"  nothing to land: {head[:12]} is already on origin/master.")
        return 0
    subjects = _git(repo, "log", "--format=%s", "origin/master..HEAD", check=False).splitlines()
    paths = _git(repo, "diff", "--name-only", "origin/master...HEAD", check=False).splitlines()
    tops = sorted(
        {p.split("/", 2)[0] if p.count("/") < 2 else "/".join(p.split("/")[:2]) for p in paths}
    )
    what = subjects[0] if subjects else "(no commits ahead of origin/master)"
    if len(subjects) > 1:
        what += f" (+{len(subjects) - 1} more commit(s))"
    touches = ", ".join(tops[:6]) + (", ..." if len(tops) > 6 else "")
    print(
        "Lanes do not land themselves: the integration session merges, gates and pushes\n"
        "(docs/engineering/active-lanes/integration.md). Send it this, via SendMessage to\n"
        "the integrator in ListAgents:\n\n"
        f"    merge {branch} @ {head} — {what}; touches {touches or 'nothing'}\n\n"
        "Then keep working. `./vibeos-cli land-status --tip <sha>` says when it is on master."
    )
    return 2


def _direct_gate_skip(repo: Path, tree: str) -> str | None:
    """Why `land --direct` may push this (already master-merged) tree ungated, or None.

    A fresh full-gate stamp, or a delta vs origin/master that is docs-only. The second is
    the pre-push hook's own exemption (every non-docs path on master was gated when it
    landed), so the push it allows is one the hook would have allowed anyway. Without it
    the integrator paid a 21-minute cold gate for one Markdown file (cpp-cluster's plan
    note, 2026-10-16): the preflights skipped it, the gate did not.
    """
    if fresh_full_stamp(repo, tree):
        return f"fresh green full-gate stamp already covers tree {tree[:12]}"
    if _docs_only_vs_master(repo):
        return "docs-only delta vs origin/master (the pre-push hook's exemption)"
    return None


def _land_direct(args, repo: Path) -> int:
    run = Run("land", "master")
    try:
        branch, head = _preflight(repo)
        _git(repo, "fetch", "origin")
        remote = _git(repo, "rev-parse", "origin/master")
        if remote == head:
            run.add(
                Stage("preflight", status="ok", detail="HEAD == origin/master: nothing to land")
            )
            return run.finalize(repo)
        behind = _git_ok(repo, "merge-base", "--is-ancestor", remote, head)
        stamp = fresh_full_stamp(repo, _git(repo, "rev-parse", "HEAD^{tree}"))
        run.add(
            Stage(
                "preflight",
                status="ok",
                detail=f"branch {branch} @ {head[:12]}; "
                f"origin/master {'contained' if behind else 'AHEAD — will merge'}; "
                f"stamp {'present' if stamp else 'needed'}",
            )
        )
        if getattr(args, "dry_run", False):
            run.log(
                "dry run: would "
                + ("merge origin/master, " if not behind else "")
                + ("skip the gate (fresh stamp), " if stamp and behind else "run the full gate, ")
                + "then push HEAD:master under the land lock",
                header="dry-run",
            )
            _report_queue(run)
            return run.finalize(repo)

        # BEFORE the land lock, not inside it: the direct path's gate is this lane's own,
        # but a lint failure still burns a whole gate, and there is no reason to hold a
        # fleet-wide lock for the ~40s this takes.
        _warm_preflight(repo, run, args)
        _concurrent_preflights(repo, run, args)

        _report_queue(run)
        timeout = float(getattr(args, "queue_timeout", 0) or hostlock.LAND_LOCK_TIMEOUT)
        t_queue = time.monotonic()
        with hostlock.land_lock(f"land:{branch}", timeout=timeout, repo=str(repo)):
            queued = time.monotonic() - t_queue
            run.add(
                Stage("land-lock", seconds=queued, status="ok", detail="held (fleet-wide, FIFO)")
            )
            # Re-fetch INSIDE the lock: whoever we queued behind just landed, so the
            # origin/master we resolved during preflight is stale by construction.
            _git(repo, "fetch", "origin")
            head = _merge_master(repo, run)
            tree = _git(repo, "rev-parse", "HEAD^{tree}")
            skip = _direct_gate_skip(repo, tree)
            if skip:
                run.add(Stage("gate", status="skip", detail=skip))
            else:
                _run_gate(repo, run, args)
            if getattr(args, "no_push", False):
                run.add(Stage("push", status="skip", detail="--no-push: gated but not landed"))
            else:
                _push(repo, run, head)
        run.extra["branch"] = branch
        run.extra["head"] = head
        run.extra["queued_s"] = round(queued, 1)
        return run.finalize(repo)
    except (hostlock.LandLockBusy, LandAbort) as e:
        run.classification = (
            "LAND_LOCK_BUSY" if isinstance(e, hostlock.LandLockBusy) else "LAND_ABORT"
        )
        run.add(Stage("land", status="fail", detail=str(e).splitlines()[0], salient=[str(e)]))
        return run.finalize(repo)
    except subprocess.SubprocessError as e:
        run.classification = "LAND_ABORT"
        run.add(Stage("land", status="fail", detail=f"git subprocess error: {e}"))
        return run.finalize(repo)


def _explain_tip(repo: Path, tip: str) -> int:
    """`land-status --tip <sha>`: where did THIS commit get to?

    About a specific commit, never "did this lane land anything recently": a lane that
    sent a newer sha has landed something without landing the one it is asking about. Three
    answers, each from git alone: on origin/master; merged into the integrator's current
    batch (the shared `integration` branch), so gating or about to; or neither."""
    try:
        tip = _git(repo, "rev-parse", "--verify", f"{tip}^{{commit}}")
    except LandAbort:
        print(f"  UNKNOWN COMMIT: {tip} does not resolve in this worktree.")
        return 2
    short = tip[:12]
    run_git(repo, ("fetch", "origin"), 600)
    if _git_ok(repo, "merge-base", "--is-ancestor", tip, "origin/master"):
        print(f"  LANDED: {short} is an ancestor of origin/master. The work is on mainline.")
        return 0
    batch = _git(repo, "rev-parse", "--verify", "-q", INTEGRATION_BRANCH, check=False)
    if batch and _git_ok(repo, "merge-base", "--is-ancestor", tip, batch):
        print(
            f"  IN BATCH: {short} is merged into the integrator's `{INTEGRATION_BRANCH}` @ "
            f"{batch[:12]}, which is not on origin/master yet: gating, or about to."
        )
        return 0
    print(
        f"  NOT LANDED: {short} is not on origin/master and not in the integrator's current "
        "batch. If you sent it, the integrator has not merged it yet (or bounced it: read "
        "its reply); if you did not, `./vibeos-cli land` prints the message to send."
    )
    return 1


def cmd_land_status(args, repo: Path) -> int:
    """`vibeos-cli land-status`: who holds the land lock, who waits on it, and the
    integrator's current batch. With `--tip <sha>`, answers about ONE commit instead."""
    tip = getattr(args, "tip", None)
    if tip:
        return _explain_tip(repo, tip)
    holder = hostlock.land_holder()
    waiters = hostlock.land_queue()
    batch = _git(repo, "rev-parse", "--verify", "-q", INTEGRATION_BRANCH, check=False)
    if getattr(args, "json", False):
        print(
            json.dumps(
                {"lander": holder, "lock_queue": waiters, INTEGRATION_BRANCH: batch or None},
                indent=2,
            )
        )
        return 0
    if holder:
        print(
            f"LANDING: pid={holder.get('pid')} {holder.get('kind', '?')} "
            f"in {holder.get('cwd', '?')}"
        )
    else:
        print("LANDING: none (the land lock is free)")
    for i, w in enumerate(waiters):
        print(f"  waiting #{i + 1}: pid={w.get('pid')} {w.get('kind', '?')}")
    if batch:
        print(f"  {INTEGRATION_BRANCH}: {batch[:12]} (the integrator's current batch)")
    return 0


__all__ = ["STAMP_MAX_AGE_S", "LandAbort", "cmd_land_status", "fresh_full_stamp", "land"]
