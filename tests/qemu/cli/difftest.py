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
    names = re.findall(r'"([^"]+)"', m.group(1))
    return sorted({n.rsplit("/", 1)[-1] for n in names if n.rsplit("/", 1)[-1].endswith(_SUFFIX)})


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


def run_difftests(repo: Path, run: Run, crates: list[str] | None = None) -> bool:
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
        cp = run.run(
            ["cargo", "test", "--release", "-p", name],
            cwd=ws,
            env=dict(os.environ),
            stage=name,
        )
        out = (cp.stdout or "") + (cp.stderr or "")
        n = graded(out)
        ok = cp.returncode == 0 and n > 0
        all_ok &= ok
        if cp.returncode != 0:
            detail = f"exit {cp.returncode}"
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
                salient=[] if ok else _salient(out) or [f"{name}: exit 0 but 0 cases compared"],
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

    run = Run("difftest", "host")
    ok = run_difftests(repo, run, only)
    if not ok:
        run.classification = "DIFFTEST_DIVERGENCE"
    return run.finalize(repo)
