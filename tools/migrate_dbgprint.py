#!/usr/bin/env python3
"""
Migrate Rust call sites from C variadic FFI (KdbPrintf/KdbpPrint/KdpDprintf)
to the Rust-native macros from `crate::dbgprint`. See docs in
`sdk/lib/rust/ntoskrnl-rust/src/dbgprint.rs` for rationale.

For each .rs file argued (or the kernel default set):
  - Rewrite each `<F>(b"...".as_ptr() as <TYPE>, ...args)` call to
    `crate::<macro>!(b"...", ...args)`.
  - Drop matching `extern "C" { fn <F>(Format: <TYPE>, ...); }` lines/blocks
    (only when *every* original call has been rewritten — otherwise leave
    the extern intact so the file still builds).

Usage:
  tools/migrate_dbgprint.py <file.rs> [<file.rs> ...]
  tools/migrate_dbgprint.py --apply-default     # all known callers
  tools/migrate_dbgprint.py --dry-run <file.rs> # show stats, don't write
"""

import argparse
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# Function -> (macro path, accepted cast-target types).
TARGETS = {
    "KdbpPrint": ("crate::kdbp_print", ["PCHAR", "PSTR", "PCSTR", "*const i8", "*mut i8"]),
    "KdbPrintf": ("crate::kdb_printf", ["PCHAR", "PSTR", "PCSTR", "*const i8", "*mut i8"]),
    "KdpDprintf": ("crate::kdp_dprintf", ["PCHAR", "PSTR", "PCSTR", "*const i8", "*mut i8"]),
    "DbgPrint": (
        "crate::dbg_print",
        ["PCHAR", "PSTR", "PCSTR", "*const i8", "*mut i8", "*const u8", "*mut u8"],
    ),
}

# Skip files that *implement* the migration (their docstrings show old patterns).
SKIP = {"dbgprint.rs"}


# Default file set: every Rust file declaring an extern for one of TARGETS.
def find_default_files() -> list[Path]:
    root = REPO / "sdk/lib/rust/ntoskrnl-rust/src"
    out = []
    for p in root.rglob("*.rs"):
        if p.name in SKIP:
            continue
        try:
            txt = p.read_text()
        except Exception:
            continue
        if any(re.search(rf"\bfn\s+{name}\s*\(", txt) for name in TARGETS):
            out.append(p)
    return sorted(out)


def _has_active_call(src: str, fname: str) -> bool:
    """True if `src` contains a call site for `fname` that is *not* a comment
    and *not* the extern decl line itself."""
    decl_re = re.compile(rf"\bfn\s+{re.escape(fname)}\s*\(")
    call_re = re.compile(rf"\b{re.escape(fname)}\s*\(")
    for line in src.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        if decl_re.search(line):
            continue
        if call_re.search(line):
            return True
    return False


def migrate_file(path: Path, dry_run: bool = False) -> tuple[int, int]:
    """Return (rewrites, remaining_extern_lines_dropped)."""
    src = path.read_text()
    original = src
    total_rewrites = 0

    for fname, (macro, cast_types) in TARGETS.items():
        # Build a regex matching: <fname>( [b]"...".as_ptr() [as CAST] [, or )]
        # Accept both b"..." and "..." literals; cast is optional (some sites
        # have raw `.as_ptr()` returning *const u8).
        cast_alts = "|".join(re.escape(c) for c in cast_types)
        pattern = re.compile(
            rf"\b{re.escape(fname)}\(\s*"
            rf'(b?"(?:[^"\\]|\\.)*")'  # fmt literal
            rf"\s*\.as_ptr\(\)"  # .as_ptr()
            rf"(?:\s+as\s+(?:{cast_alts}))?"  # optional cast
            rf"(\s*[,)])",  # terminator
            re.DOTALL,
        )

        # The regex captures the trailing `,` or `)`. Either way it acts as
        # the macro call's matching token: `,` keeps the args flowing in, `)`
        # closes the call. Emit the captured punctuation verbatim — adding a
        # second `)` here would yield `macro!(...))` (the bug from v0).
        def repl(m, macro=macro):
            return f"{macro}!({m.group(1)}{m.group(2)}"

        src, n = pattern.subn(repl, src)
        total_rewrites += n

        # If no remaining ACTIVE call sites (skipping comments + the extern
        # decl line itself), drop the matching extern decl.
        if not _has_active_call(src, fname):
            # Match a single-line: `    fn KdbpPrint(Format: <CAST>, ...);`
            # Allow `pub` prefix and any cast type by being permissive on the
            # arg type.
            line_pat = re.compile(
                rf"^\s*(?:pub\s+)?fn\s+{re.escape(fname)}\s*\([^)]*\.\.\.\s*\)\s*;\s*\n",
                re.MULTILINE,
            )
            src, _ = line_pat.subn("", src)

    if src != original and not dry_run:
        path.write_text(src)
    return total_rewrites, src != original


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="*", help=".rs files to migrate")
    ap.add_argument(
        "--apply-default",
        action="store_true",
        help="apply to all kernel files referencing a target FFI",
    )
    ap.add_argument("--dry-run", action="store_true", help="report changes without writing")
    args = ap.parse_args()

    if args.apply_default:
        files = find_default_files()
    else:
        files = [Path(f).resolve() for f in args.files]
    if not files:
        ap.error("no files (use --apply-default or list .rs files)")

    total = 0
    touched = 0
    for f in files:
        n, _ = migrate_file(f, args.dry_run)
        if n:
            print(f"  {f.relative_to(REPO)}: {n} rewrites" + (" (dry)" if args.dry_run else ""))
            total += n
            touched += 1
    print(
        f"-- {total} call sites rewritten across {touched} files"
        + (" (dry)" if args.dry_run else "")
    )


if __name__ == "__main__":
    main()
