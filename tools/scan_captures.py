#!/usr/bin/env python3
"""scan_captures.py — post-opt LLVM-IR audit for the captures(none) miscompile class.

The class (docs/engineering/rust-kernel-patterns.md "LLVM Ignores Volatile Stores"):
LLVM's attribute pass deduces `readnone captures(none)` on output-pointer params
(volatile stores are ignored during deduction) -> callers pass `ptr poison` ->
stores through poison are UB -> the out-write folds away. The binary either
BUGCHECKs far from the cause (0x50: consumer derefs the never-written pointer)
or silently hangs (a status/wait value never set). scan_ud2 only catches the
bodies that collapse all the way to `unreachable`; the elided-store shape is
invisible in the binary. THIS tool reads the truth at the only layer where the
fold is visible: post-optimization LLVM IR (`--emit=llvm-ir`, one .ll per CGU).

Findings:
  [POISON_ARG]        call/invoke passing `ptr poison`/`ptr undef` — a COMMITTED
                      fold: the caller has concluded the callee ignores the arg.
                      Auto-downgraded to benign when the callee param's source
                      name is `_`-prefixed (Rust unused convention). CAVEAT: a
                      `let _ = param;` discard (faithful-to-C unused params,
                      e.g. MiLoadImageSection file_name/ldr_entry) is ALSO
                      legal poison but keeps its source name — triage new
                      findings against the Rust source before believing them.
  [READNONE_OUTPARAM] exported (unmangled) fn defined with a ptr param attributed
                      `readnone captures(none)` — the LATENT shape. Legit for
                      pure pointer-compare fns; baseline-diff separates stock
                      from regressions.
  [UNREACHABLE_BODY]  define whose first real instruction is `unreachable` —
                      IR-level twin of scan_ud2 (catches it pre-link, per-CGU).

Healthy out-params read `writeonly captures(none) initializes(...)` — those are
proof the store SURVIVED and are never reported.

Canonical usage — `tools/audit-captures.sh <arch> [--lint]`, which builds with
VIBEOS_EMIT_LLVM_IR=1 (reactos-link appends --emit=llvm-ir to every kernel
crate's RUSTFLAGS — the SHIPPING pipeline: nightly + kernel json + build-std)
and scans sdk/lib/rust/target/ntoskrnl-cargo-*-rustlib/<triple>/release/deps.
2026-07-05 amd64 verdict: ZERO live folds — every finding triaged benign
(unused-param poison, std idiom, debug-only params); the volatile-blind
deduction root is FIXED in LLVM >= 21 (rust-kernel-patterns.md "RESOLVED AT
THE TOOLCHAIN"). Re-run after every rustc/LLVM bump.

Exit code: --lint exits 1 if any POISON_ARG or UNREACHABLE_BODY exists (the
committed shapes); READNONE_OUTPARAM alone is reported but not fatal (stock is
baseline-managed like the other lints).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# fns whose poison args are LLVM plumbing, not the miscompile
_INTRINSIC_PREFIX = "@llvm."
# std crates: poison/unreachable inside them is upstream idiom, not our bug
_STD_CRATES = {"core", "alloc", "compiler_builtins"}
# panic-plumbing callees legally take poison fmt args (core::panicking::*)
_PANIC_CALLEE_RE = re.compile(r"panicking|assert_failed")
# by-design unreachable std shapes (instantiated into our crates too)
_BENIGN_UNREACHABLE_RE = re.compile(r"NeverShortCircuit|zero_div_fn")

_DEFINE_RE = re.compile(r"^define\s.*?@([\w$.]+)\((.*)\)[^)]*\{?\s*$")
_CALL_NAME_RE = re.compile(r"(?:call|invoke)\s+.*?@([\w$.]+)\(")
# value tokens only — `noundef` is an attribute and must not match
_POISON_TOKEN_RE = re.compile(r"\b(?:poison|undef)\b(?<!noundef)")


def _split_params(paramstr: str) -> list[str]:
    """Split an IR param/arg list at top-level commas (parens/brackets nest)."""
    out: list[str] = []
    depth = 0
    cur: list[str] = []
    for ch in paramstr:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    tail = "".join(cur).strip()
    if tail:
        out.append(tail)
    return out


def _is_exported(name: str) -> bool:
    """Unmangled = #[no_mangle]/extern export.

    Internal Rust fns are mangled: legacy `_ZN...` (stable) or v0 `_R...`
    (nightly, e.g. the build-std kernel pipeline)."""
    return not (name.startswith("_ZN") or name.startswith("_R") or name.startswith("anon."))


class Findings:
    def __init__(self) -> None:
        # (crate, caller, callee, idxs) pending classification
        self.poison_raw: set[tuple[str, str, str, tuple[int, ...]]] = set()
        # (crate, fn, param#, param name) pending stub-classification
        self.readnone_raw: set[tuple[str, str, int, str]] = set()
        self.poison: set[str] = set()
        self.poison_benign: set[str] = set()
        self.readnone: set[str] = set()
        self.unreachable: set[str] = set()
        # fn name -> param names (from its define, any CGU)
        self.defines: dict[str, list[str]] = {}
        # fns whose whole body is a bare `ret` — NOT_IMPLEMENTED-style stubs:
        # readnone on their params is correct, not a fold
        self.stub_fns: set[str] = set()

    def finalize(self) -> None:
        """Classify poison callsites: an arg landing on a callee param whose
        source name starts with `_` (Rust unused-param convention) is a legal,
        harmless poison — LLVM proved what the source already declared. Only
        poison into a *named* (used) param, or into a callee we can't see the
        definition of, is a committed miscompile candidate."""
        for crate, caller, callee, idxs in self.poison_raw:
            params = self.defines.get(callee)
            where = ",".join(map(str, idxs))
            line = f"[POISON_ARG] {crate}: @{caller} -> @{callee} arg#{where}"
            if params is not None and all(
                i < len(params) and re.match(r"^_[A-Za-z]", params[i]) for i in idxs
            ):
                self.poison_benign.add(line + " (benign: param unused in source)")
            elif crate in _STD_CRATES:
                self.poison_benign.add(line + " (benign: std-internal idiom)")
            elif _PANIC_CALLEE_RE.search(callee):
                self.poison_benign.add(line + " (benign: panic plumbing)")
            else:
                self.poison.add(line)
        for crate, fn, i, name in self.readnone_raw:
            if fn in self.stub_fns:
                continue  # bare-ret stub: params genuinely unused
            self.readnone.add(f"[READNONE_OUTPARAM] {crate}: @{fn} param#{i} %{name}")

    def total_fatal(self) -> int:
        return len(self.poison) + len(self.unreachable)


def scan_file(path: Path, f: Findings, crate: str) -> None:
    current_fn = "?"
    prev_was_define = False
    try:
        text = path.read_text(errors="replace")
    except OSError as e:  # unreadable .ll is a real problem, not a skip
        sys.exit(f"scan_captures: cannot read {path}: {e}")

    for line in text.splitlines():
        stripped = line.strip()

        m = _DEFINE_RE.match(line)
        if m:
            current_fn = m.group(1)
            prev_was_define = True
            params = _split_params(m.group(2))
            # record param source-names for poison classification
            names = []
            for p in params:
                nm = re.search(r"%([\w.$]+)\s*$", p)
                names.append(nm.group(1) if nm else "")
            f.defines[current_fn] = names
            # latent shape: exported fn, ptr param deduced readnone+captures(none)
            if _is_exported(current_fn):
                for i, p in enumerate(params):
                    if (
                        p.startswith("ptr")
                        and "readnone" in p
                        and ("captures(none)" in p or "nocapture" in p)
                        and not re.match(r"^_[A-Za-z]", names[i])
                    ):
                        f.readnone_raw.add((crate, current_fn, i, names[i]))
            continue

        # first real instruction after the define line: classifies the body
        if prev_was_define:
            if stripped.endswith(":") or not stripped:  # entry label / blank
                continue
            if stripped == "unreachable" or stripped.startswith("unreachable"):
                if crate not in _STD_CRATES and not _BENIGN_UNREACHABLE_RE.search(current_fn):
                    f.unreachable.add(f"[UNREACHABLE_BODY] {crate}: @{current_fn}")
            elif stripped.startswith("ret "):
                # whole body is a single `ret` — a NOT_IMPLEMENTED-style stub
                f.stub_fns.add(current_fn)
            prev_was_define = False
            # fall through: this line could also be a call

        if ("call" in line or "invoke" in line) and _POISON_TOKEN_RE.search(line):
            cm = _CALL_NAME_RE.search(line)
            callee = cm.group(1) if cm else "<indirect>"
            if f"@{callee}".startswith(_INTRINSIC_PREFIX):
                continue
            # arg indices carrying poison/undef ptrs (word-bounded: `noundef`
            # is an attribute, not a value)
            args_m = re.search(r"@[\w$.]+\((.*)\)", line) if cm else None
            idxs = []
            if args_m:
                for i, a in enumerate(_split_params(args_m.group(1))):
                    if a.startswith("ptr") and _POISON_TOKEN_RE.search(a):
                        idxs.append(i)
            if not idxs:
                continue  # matched inside metadata/aggregate only — skip
            f.poison_raw.add((crate, current_fn, callee, tuple(idxs)))


def crate_of(ll: Path) -> str:
    # deps/ntoskrnl_rust-<meta>.<cgu>.rcgu.ll -> ntoskrnl_rust
    return ll.name.split("-", 1)[0]


def main() -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    ap.add_argument("paths", nargs="+", help=".ll files or dirs containing them")
    ap.add_argument("--lint", action="store_true", help="exit 1 on committed shapes")
    ap.add_argument(
        "--crates",
        help="comma-separated crate-name filter (default: all workspace .ll found)",
    )
    args = ap.parse_args()

    lls: list[Path] = []
    for p in map(Path, args.paths):
        if p.is_dir():
            lls.extend(sorted(p.rglob("*.ll")))
        elif p.suffix == ".ll":
            lls.append(p)
        else:
            sys.exit(f"scan_captures: not a .ll file or directory: {p}")
    if not lls:
        sys.exit("scan_captures: no .ll files found — emit with RUSTFLAGS=--emit=llvm-ir")

    want = set(args.crates.split(",")) if args.crates else None
    f = Findings()
    scanned = 0
    for ll in lls:
        crate = crate_of(ll)
        if want and crate not in want:
            continue
        scan_file(ll, f, crate)
        scanned += 1

    f.finalize()
    for group in (
        sorted(f.poison),
        sorted(f.unreachable),
        sorted(f.readnone),
        sorted(f.poison_benign),
    ):
        for line in group:
            print(line)

    print(
        f"scan_captures: {scanned} CGU files; "
        f"{len(f.poison)} POISON_ARG ({len(f.poison_benign)} benign), "
        f"{len(f.unreachable)} UNREACHABLE_BODY, "
        f"{len(f.readnone)} READNONE_OUTPARAM",
        file=sys.stderr,
    )
    if args.lint and f.total_fatal():
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
