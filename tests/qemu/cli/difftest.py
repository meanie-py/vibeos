"""`vibeos-cli difftest` — run every host differential harness.

The difftest crates compile the GENUINE ReactOS C as a host translation unit
with its public symbols renamed to `ref_*`, link that beside the shipping Rust,
and fuzz the two for parity in one process. They are the only oracle that grades
a translation against the code it replaced rather than against someone's reading
of it, and they are cheap: the whole set is well under a minute on a warm target
dir, no QEMU, no build-<arch>.

Nothing ran them. They were reachable only by typing `cargo test` inside a crate
dir, so nobody did, and `rtl-difftest` sat 2/87 red on master long enough that
`git log` no longer says which commit broke it. One of the two was a genuine
divergence in eleven `Rtl*String` converters (the allocation-failure path left
the caller's stale `Buffer` beside a freshly-written `Length`); the other was a
hand-typed `STATUS_UNMAPPABLE_CHARACTER` in the reference's own header, wrong
in the digits — an oracle quietly grading against a value Windows never returns.
An oracle nobody runs decays into a liability, so this is the front door and the
gate runs it.

Two things this must keep doing, both learned the hard way:

* **One cargo invocation PER crate.** cnt-difftest depends on cnt-rust with
  `features = ["cabi"]`; msvcrt-difftest deliberately does not, because it
  grades msvcrt-crt-rust's own wrappers. Build them together and cargo's feature
  unification switches `cabi` on for the shared cnt-rust, whose 208 `#[no_mangle]`
  shims then collide with the wrappers at link time (`duplicate symbol: mbtowc`).
  `cargo test --workspace` cannot run these; it never could.
* **Discover the crates, never list them.** A hardcoded list is how a harness
  goes missing — someone adds `foo-difftest` and no gate ever learns about it.
  Membership comes from the workspace manifest.
"""

from __future__ import annotations

import os
import shutil
from pathlib import Path

from tools import qemu_infra_compat as native

from .artifacts import Run, Stage

# The decisions are native: tools/vibeos-test/src/difftest.rs (discovery, the per-crate cargo
# plan, the graded-case count and verdict). This facade runs the plan through `Run`, which
# owns the artifact dir and the stages.

RUST_WS = ("sdk", "lib", "rust")


def _call(op: str, **kwargs):
    return native.call("difftest_" + op, kwargs)


def discover(repo: Path) -> list[str]:
    """Workspace members that are differential harnesses (see difftest.rs)."""
    return _call("discover", repo=repo)


def crate_target(repo: Path, name: str) -> str | None:
    """The `--target` triple a harness declares in its manifest, or None for the host."""
    return _call("crate_target", repo=repo, name=name)


def crate_feature_sets(repo: Path, name: str) -> list[str]:
    """Extra `--features` builds a harness declares, each its own invocation."""
    return _call("crate_feature_sets", repo=repo, name=name)


def _salient(out: str, limit: int = 6) -> list[str]:
    return _call("salient", out=out, limit=limit)


def graded(out: str) -> int:
    """How many cases a harness actually graded; zero is the vacuous pass."""
    return _call("graded", out=out)


def generated_checks(repo: Path) -> list[tuple[str, list[str]]]:
    """`(label, argv)` for every generated msvcrt-difftest artifact's `--check`."""
    return [(label, argv) for label, argv in _call("generated_checks", repo=repo)]


def check_generated(repo: Path, run: Run) -> bool:
    """Re-derive every generated refstub artifact and refuse a stale one.

    ── why this belongs HERE and not in `vibeos-cli check` ──────────────────

    These artifacts have had a `--check` mode since the day they were written and
    nothing ran it, so a PSDK header that moved a constant, or a reference `.c`
    that changed a signature, stayed silent until some row happened to reach it —
    and a row that reaches a stale constant does not report "stale", it reports a
    DIVERGENCE in the Rust. Wrong file, hours later.

    `check` is the wrong home because re-derivation needs the pinned
    cross-compiler AND `build-amd64` (the prototypes come from the built object,
    not the source text: `_MSVCR_VER` opens and closes whole families, so what got
    EMITTED is the authority). `check` runs on a fresh worktree that has no build
    dir. `difftest` already hard-requires one — msvcrt-difftest's build.rs reads
    `build-amd64/.../*.obj` — so here the dependency costs nothing new.

    And it runs BEFORE the harnesses, deliberately. A stale header does not make
    one row wrong, it makes every row over that TU suspect, so the useful failure
    is "your oracle is stale", not forty divergences.

    FLEET ISOLATION: the generators resolve `build-amd64` under THIS repo root and
    hard-fail if it is absent. That refusal is the correct behaviour and must not
    be softened into a skip — a check that quietly does nothing when the build dir
    is missing is exactly the shape of the four waves this closes.
    """
    all_ok = True
    for label, argv in generated_checks(repo):
        cp = run.run(argv, cwd=repo, env=dict(os.environ), stage=f"drift:{label}")
        out = (cp.stdout or "") + (cp.stderr or "")
        ok = cp.returncode == 0
        all_ok &= ok
        run.add(
            Stage(
                name=f"drift:{label}",
                exit=cp.returncode,
                status="ok" if ok else "fail",
                detail="in sync" if ok else "STALE — regenerate",
                salient=[]
                if ok
                else [ln.strip()[:200] for ln in out.splitlines() if ln.strip()][:6],
            )
        )
    return all_ok


def run_difftests(
    repo: Path,
    run: Run,
    crates: list[str] | None = None,
    *,
    cases: list[str] | None = None,
    test_target: str = "diff",
) -> bool:
    """Run each harness in its own cargo invocation. Returns True iff all green."""
    ws = repo.joinpath(*RUST_WS)
    names = crates if crates is not None else discover(repo)
    if not names:
        run.add(Stage("discover", status="fail", detail="no *-difftest members in the workspace"))
        return False
    if shutil.which("cargo") is None:
        run.add(Stage("cargo", status="fail", detail="cargo not on PATH"))
        return False

    all_ok = True
    plan = _call("plan", repo=repo, crates=names, cases=cases or [], test_target=test_target)
    for build in plan:
        name = build["name"]
        cp = run.run(build["argv"], cwd=ws, env=dict(os.environ), stage=name)
        out = (cp.stdout or "") + (cp.stderr or "")
        v = _call("verdict", name=name, out=out, rc=cp.returncode, cases=len(cases or []))
        all_ok &= v["ok"]
        run.add(
            Stage(
                name=name,
                exit=cp.returncode,
                status="ok" if v["ok"] else "fail",
                detail=v["detail"],
                salient=v["salient"],
            )
        )
    return all_ok


def difftest(args, repo: Path) -> int:
    """`vibeos-cli difftest [--only NAME[,NAME]]`."""
    only = None
    raw = getattr(args, "only", None)
    if raw:
        known = discover(repo)
        only = [c.strip() for c in raw.split(",") if c.strip()]
        unknown = [c for c in only if c not in known]
        if unknown:
            # Fail loud rather than silently grading a subset: a typo'd --only
            # that quietly ran nothing would report green over zero harnesses.
            print(f"unknown difftest crate(s): {', '.join(unknown)}")
            print(f"known: {', '.join(known) or '(none)'}")
            return 2

    cases = getattr(args, "case", None)
    if cases and (only is None or len(only) != 1 or len(set(cases)) != len(cases)):
        print("--case requires exactly one --only crate and distinct exact case names")
        return 2
    run = Run("difftest", "host")
    # The oracle's own inputs first — see check_generated. A stale generated header
    # makes every row over its TU suspect, and the harnesses still run afterwards so
    # one command reports both facts rather than hiding the second behind the first.
    fresh = check_generated(repo, run)
    if cases:
        ok = run_difftests(
            repo, run, only, cases=cases, test_target=getattr(args, "test_target", "diff")
        )
    else:
        ok = run_difftests(repo, run, only)
    if not fresh:
        run.classification = "DIFFTEST_STALE_ORACLE"
    elif not ok:
        run.classification = "DIFFTEST_DIVERGENCE"
    return run.finalize(repo)
