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
import re
import shutil
from pathlib import Path

from .artifacts import Run, Stage

RUST_WS = ("sdk", "lib", "rust")

#: A workspace member is a difftest harness iff its directory name ends in this.
#: Naming *is* the registration — see the module docstring.
_SUFFIX = "-difftest"


def discover(repo: Path) -> list[str]:
    """Workspace members whose name marks them a differential harness.

    Parsed out of the `[workspace] members` list rather than globbed off disk so
    a crate that exists but was never wired into the workspace does not silently
    look covered.
    """
    manifest = repo.joinpath(*RUST_WS) / "Cargo.toml"
    if not manifest.is_file():
        return []
    text = manifest.read_text()
    m = re.search(r"^\s*members\s*=\s*\[(.*?)\]", text, re.S | re.M)
    if not m:
        return []
    names = [n.rsplit("/", 1)[-1] for n in re.findall(r'"([^"]+)"', m.group(1))]
    return sorted({n for n in names if n.endswith(_SUFFIX) or _host_tests(repo, n)})


def _host_tests(repo: Path, name: str) -> bool:
    """A shared crate whose own `#[test]`s are known-answer checks run here too:

        [package.metadata.difftest]
        host-tests = true

    win32macros-rust's 22 are computed from the C macro by hand, and they are what caught
    RTL_CONSTANT_STRING pointing Buffer at a stack copy. Nothing ran them, so the
    `as u8` crate-repair once wrote into IsRecognizedPartition! landed on master with two of
    them failing. Opt-in by manifest for the same reason the suffix is the registration: a
    shared crate's tests may need the host to build it, and the crate is the one that knows.
    """
    m = re.search(
        r"^\s*\[package\.metadata\.difftest\]\s*$(.*?)(?=^\s*\[|\Z)",
        _manifest(repo, name),
        re.S | re.M,
    )
    return bool(m and re.search(r"^\s*host-tests\s*=\s*true\b", m.group(1), re.M))


def _manifest(repo: Path, name: str) -> str:
    path = repo.joinpath(*RUST_WS) / name / "Cargo.toml"
    return path.read_text() if path.is_file() else ""


def crate_target(repo: Path, name: str) -> str | None:
    """The `--target` triple a harness declares, or None for the host.

    `msvcrt-seh-difftest` grades `except_i386.c`, which is inside `#ifdef __i386__` and whose
    Rust counterpart is `#![cfg(target_arch = "x86")]` — so it only exists in a 32-bit binary.
    Cargo builds one crate for one target per invocation, so that crate declares its triple in
    its own manifest:

        [package.metadata.difftest]
        target = "i686-unknown-linux-gnu"

    DISCOVERED from the manifest for the same reason the crate set is: a driver that carried
    the list would silently build the next such harness for the host, where its `#[cfg]`s
    compile to nothing and it grades zero rows.
    """
    manifest = repo.joinpath(*RUST_WS) / name / "Cargo.toml"
    if not manifest.is_file():
        return None
    m = re.search(
        r"^\s*\[package\.metadata\.difftest\]\s*$(.*?)(?=^\s*\[|\Z)",
        manifest.read_text(),
        re.S | re.M,
    )
    if not m:
        return None
    t = re.search(r'^\s*target\s*=\s*"([^"]+)"', m.group(1), re.M)
    return t.group(1) if t else None


def _salient(out: str, limit: int = 6) -> list[str]:
    """The lines a human needs from a failed harness: the assertion and its two
    values, not the linker command line."""
    keep: list[str] = []
    for ln in out.splitlines():
        s = ln.strip()
        if not s:
            continue
        if (
            s.startswith("error")
            or "panicked at" in s
            or s.startswith("assertion")
            or s.startswith("left:")
            or s.startswith("right:")
            or "FAILED" in s
            or "test result: FAILED" in s
        ):
            keep.append(s[:200])
        if len(keep) >= limit:
            break
    return keep


def graded(out: str) -> int:
    """How many cases did this harness actually grade?

    Each crate prints its count in its own dialect, and every one of them also
    emits `test result: ok. 0 passed` for its empty lib/doc-test targets — so a
    zero total means the harness ran, exited 0, and compared nothing. That is
    the vacuous pass, and the caller treats it as a failure: a suite that cannot
    fail is worse than no suite, because green stops anyone looking again.
    """
    pats = (
        r"difftest result: ok\. (\d+) passed",
        r"(\d+) mbcs cases passed",
        r"(\d+) create_mbcinfo codepages passed",
        r"(\d+) printf shell rows passed",
        r"(\d+) wave16 cases passed",
        r"(\d+) scanf cases passed",
        # (?<![a-z]) or "difftest result: ok. 81 passed" matches TWICE — the
        # libtest form is a literal substring of the difftest form, so cnt's 81
        # cases would report as 162.
        r"(?<![a-z])test result: ok\. (\d+) passed",
    )
    return sum(int(m.group(1)) for p in pats for m in re.finditer(p, out))


#: Where msvcrt-difftest keeps the headers its build.rs force-includes.
_REFSTUB = ("sdk", "lib", "rust", "msvcrt-difftest", "refstub")


def generated_checks(repo: Path) -> list[tuple[str, list[str]]]:
    """`(label, argv)` for every generated msvcrt-difftest artifact's `--check`.

    DISCOVERED, not listed, for the same reason the crates are: the set has grown
    from three artifacts to seven in two months, once per reference TU, and a
    hardcoded list is how the eighth arrives uncovered. `refstub/msvcrt_<stem>_protos.h`
    IS the registration — the file's existence names the source it was derived from.
    """
    py = "python3"
    checks: list[tuple[str, list[str]]] = [
        # The constants and the layout guardian, in one invocation: one generator
        # writes both, so one `--check` re-derives both.
        (
            "consts",
            [py, str(repo / "tools" / "gen_msvcrt_difftest_consts.py"), "--check"],
        )
    ]
    for h in sorted(repo.joinpath(*_REFSTUB).glob("msvcrt_*_protos.h")):
        stem = h.name[len("msvcrt_") : -len("_protos.h")]
        checks.append(
            (
                f"protos:{stem}",
                [py, str(repo / "tools" / "gen_msvcrt_ref_protos.py"), f"{stem}.c", "--check"],
            )
        )
    # The 32-bit harness's i386 layout guardian. Same argument as the two above: it is
    # generated from headers this repo also owns, so it can go stale without anyone editing
    # it — and a stale SEH layout does not fail, it regrades a structure walk against offsets
    # msvcrt.dll no longer has.
    if (repo / "sdk/lib/rust/msvcrt-seh-difftest/refstub/seh_layout.h").is_file():
        checks.append(
            ("layout:seh-i386", [py, str(repo / "tools" / "gen_msvcrt_seh_layout.py"), "--check"])
        )
    return checks


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
    for name in names:
        # Separate invocation per crate — see the module docstring on feature
        # unification. Sharing one `cargo test -p a -p b` breaks the link.
        argv = ["cargo", "test", "--release", "-p", name]
        target = crate_target(repo, name)
        if target:
            argv += ["--target", target]
        if cases:
            argv += ["--test", test_target, "--", "--exact", *cases]
        cp = run.run(argv, cwd=ws, env=dict(os.environ), stage=name)
        out = (cp.stdout or "") + (cp.stderr or "")
        n = graded(out)
        ok = cp.returncode == 0 and n > 0 and (not cases or n == len(cases))
        all_ok &= ok
        if cp.returncode != 0:
            detail = f"exit {cp.returncode}"
        elif cases and n != len(cases):
            detail = f"graded {n} cases, requested exactly {len(cases)}"
        elif n == 0:
            detail = "graded NOTHING"
        else:
            detail = f"{n} cases"
        run.add(
            Stage(
                name=name,
                exit=cp.returncode,
                status="ok" if ok else "fail",
                detail=detail,
                salient=[] if ok else _salient(out) or [f"{name}: {detail}"],
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
