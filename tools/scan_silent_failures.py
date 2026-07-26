#!/usr/bin/env python3
"""
scan_silent_failures.py — mechanize the "fail loud, never silently degrade" law
for kernel Rust (AST-aware).

The arm64 explorer-merge regression hid for weeks behind a bare `loop {}` panic
handler: win32k panicked every execution and the thread just spun — RUNNING,
census-blind, no serial output. The census/lockdep/panic-print work made the
*kernel's* failures loud; this lint makes silent-failure SHAPES unlandable
anywhere in the no-std workspace:

  [SILENT_LOOP]   `loop {}` (empty or comment-only body) — a silent busy-spin.
                  156 of these exist today, mostly copy-pasted panic handlers in
                  driver crates: every one is a repeat of the arm64 ghost waiting
                  to happen. Fix: print a self-report (DbgPrint / debugcon) before
                  spinning, or annotate a sanctioned halt.
  [UNWRAP]        `.unwrap()` / `.expect(...)` in kernel code — panics on
                  Err/None. The POWNED paint-mask bug WAS a `try_into().unwrap()`
                  that Err'd every execution. Panics now self-report, but a
                  recoverable condition still kills the kernel. (const/static
                  initializers are exempt: those evaluate at compile time.)
  [SWALLOWED_ERR] `Err(_) => {}` / empty `if let Err` arm — an error path that
                  does literally nothing. The dumb-fallback pattern in match form.

Scope: every crate under sdk/lib/rust whose root declares no_std (incl.
cfg_attr'd), minus /target/, tests/examples/benches, build.rs, bindgen_*.rs,
and #[cfg(test)] / #[test] subtrees (host-test unwraps are idiomatic).

Parsing is tree-sitter (NOT regex): multi-line bodies, comment-only blocks, and
cfg(test) module scoping are not regular. Results are cached per content-hash
under tools/.cache/ (a file's findings depend only on its bytes + this script).

Usage:
    python3 tools/scan_silent_failures.py                 # sweep the workspace
    python3 tools/scan_silent_failures.py path/to/file.rs # specific files
    python3 tools/scan_silent_failures.py --lint          # terse (CLI baseline gate)
    python3 tools/scan_silent_failures.py --no-cache      # force full re-scan

Suppress a sanctioned site with a `nolint` comment on the same line:
    loop {}  // nolint: silent-failure (post-BUGCHECK halt — report already printed)
Bare `// nolint` or `// nolint: silent-failure` suppress; a nolint scoped to
another tool does not. A sanctioned site MUST have already printed its report —
the annotation asserts "this spin is loud", it doesn't waive the law.
Exit 1 if any finding. Wired into `vibeos-cli check` (baseline-diff gate).
"""

import hashlib
import json
import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from tree_sitter import Language, Parser  # noqa: E402

RUST_WS = REPO / "sdk" / "lib" / "rust"
CACHE_PATH = REPO / "tools" / ".cache" / "silent_failures.json"

# nolint on the finding line: bare, or scoped to this tool.
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*silent-failure\b.*)?\s*(?:\(.*)?$|//\s*nolint\s*$")
_SCOPED_NOLINT_RE = re.compile(r"//\s*nolint:\s*([a-z-]+)")

EXCLUDE_PARTS = {"target", "tests", "examples", "benches"}
EXCLUDE_NAMES = {"build.rs"}


def _nolint_suppresses(line_text: str) -> bool:
    m = _SCOPED_NOLINT_RE.search(line_text)
    if m:
        return m.group(1) == "silent-failure"
    return bool(re.search(r"//\s*nolint\b", line_text))


def discover_no_std_crates(ws: Path) -> list[Path]:
    """Crate src dirs whose root source declares no_std (incl. #![cfg_attr(..., no_std)])."""
    out = []
    for manifest in sorted(ws.rglob("Cargo.toml")):
        if any(p in EXCLUDE_PARTS for p in manifest.parts):
            continue
        src = manifest.parent / "src"
        if not src.is_dir():
            continue
        for root_name in ("lib.rs", "main.rs"):
            root = src / root_name
            if root.is_file():
                try:
                    head = root.read_text(errors="replace")[:4096]
                except OSError:
                    continue
                if "no_std" in head:
                    out.append(src)
                break
    return out


def iter_rust_files(roots: list[Path]):
    """Rust sources this lint's finding can actually apply to.

    An `//! ORPHAN-BY-DESIGN:` file is in no `mod` chain, so nothing compiles it and no
    `.unwrap()` in it can panic at runtime — which is the entire claim this lint makes.
    Reporting one is not just noise: it puts a parked translation's findings between the
    reader and the live code, and the lint re-fires the moment the file is wired (wiring
    means removing the banner). Same banner `scan_orphan_modules` defines, honoured the
    same way `scan_cc_mismatch` honours it.
    """
    for root in roots:
        for p in sorted(root.rglob("*.rs")):
            if any(part in EXCLUDE_PARTS for part in p.parts):
                continue
            if p.name in EXCLUDE_NAMES or p.name.startswith("bindgen_"):
                continue
            try:
                if "ORPHAN-BY-DESIGN:" in p.read_text(errors="replace")[:2048]:
                    continue
            except OSError:
                continue
            yield p


# --- AST walk ----------------------------------------------------------------

_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)

_COMMENT_TYPES = {"line_comment", "block_comment"}
_ITEM_SKIP_TYPES = {"const_item", "static_item"}  # compile-time eval — unwrap is safe


def _attr_has_test(node, src: bytes) -> bool:
    return b"test" in src[node.start_byte : node.end_byte]


def _block_is_empty(block, src: bytes) -> bool:
    """True if a { } block holds no code (only comments/braces)."""
    for c in block.children:
        if c.type in ("{", "}") or c.type in _COMMENT_TYPES:
            continue
        return False
    return True


def _expr_is_unit_or_empty(node, src: bytes) -> bool:
    if node.type == "block":
        return _block_is_empty(node, src)
    return node.type == "unit_expression"


def scan_source(src: bytes) -> list[tuple[int, str, str]]:
    """Return (line, kind, message) findings for one file's bytes."""
    tree = _PARSER.parse(src)
    findings: list[tuple[int, str, str]] = []
    lines = src.decode(errors="replace").splitlines()

    def line_of(node) -> int:
        return node.start_point[0] + 1

    def suppressed(node) -> bool:
        ln = node.start_point[0]
        return ln < len(lines) and _nolint_suppresses(lines[ln])

    def walk(node, pending_attrs):
        # Skip test-gated and compile-time-eval subtrees.
        if node.type in _ITEM_SKIP_TYPES:
            return
        if node.type in ("mod_item", "function_item") and any(
            _attr_has_test(a, src) for a in pending_attrs
        ):
            return

        if node.type == "loop_expression":
            body = node.child_by_field_name("body")
            if body is not None and _block_is_empty(body, src) and not suppressed(node):
                findings.append(
                    (
                        line_of(node),
                        "SILENT_LOOP",
                        "empty `loop {}` — silent busy-spin; print a self-report first",
                    )
                )

        elif node.type == "call_expression":
            fn = node.child_by_field_name("function")
            if fn is not None and fn.type == "field_expression":
                field = fn.child_by_field_name("field")
                if field is not None:
                    name = src[field.start_byte : field.end_byte].decode(errors="replace")
                    if name in ("unwrap", "expect") and not suppressed(node):
                        findings.append(
                            (
                                line_of(field),
                                "UNWRAP",
                                f".{name}() — panics on Err/None in kernel code",
                            )
                        )
            # write_bytes(ptr, val, size_of::<T>()) where ptr is NOT *mut u8:
            # `count` is in ELEMENTS of the pointee, so a C-memset translation
            # `memset(&x, 0, sizeof(x))` zeroes sizeof(T)² BYTES — a stack/heap
            # smasher (2026-07-06: QueueUserAPC zeroed its caller's return
            # address → explorer died at rip=0; same shape ×17 across
            # kernel32/classpnp/ks/mscms/mountmgr). Byte-cast the pointer
            # (`as *mut u8`) or pass count=1.
            if fn is not None and not suppressed(node):
                fname = src[fn.start_byte : fn.end_byte]
                if (
                    fname.endswith(b"write_bytes")
                    or fname.endswith(b"copy_nonoverlapping")
                    or fname.endswith(b"ptr::copy")
                ):
                    args = node.child_by_field_name("arguments")
                    if args is not None:
                        argn = [c for c in args.children if c.is_named]
                        if len(argn) == 3:
                            # For the copy fns check BOTH pointer args; for
                            # write_bytes arg1 is the fill byte (never a ptr).
                            _is_copy = not fname.endswith(b"write_bytes")
                            ptxt = src[argn[0].start_byte : argn[0].end_byte]
                            if _is_copy:
                                ptxt = ptxt + b" " + src[argn[1].start_byte : argn[1].end_byte]
                            ctxt = src[argn[2].start_byte : argn[2].end_byte]
                            # Pointee-size-1 casts (u8/i8/CHAR/c_void — c_void is
                            # size 1 in Rust) make count==bytes; a `/ size_of`
                            # division means the count is deliberate element math.
                            _size1 = any(
                                t in ptxt
                                for t in (
                                    b"*mut u8",
                                    b"*const u8",
                                    b"*mut i8",
                                    b"*const i8",
                                    b"*mut CHAR",
                                    b"*const CHAR",
                                    b"*mut UCHAR",
                                    b"*const UCHAR",
                                    b"PUCHAR",
                                    b"PCHAR",
                                    b"LPBYTE",
                                    b"PBYTE",
                                    b"*mut BYTE",
                                    b"*const BYTE",
                                    b"c_void",
                                    b"PVOID",
                                    b"<u8>",
                                )
                            )
                            if b"size_of" in ctxt and not _size1 and b"/" not in ctxt:
                                findings.append(
                                    (
                                        line_of(node),
                                        "WRITE_BYTES_SIZEOF",
                                        "write_bytes(ptr, _, size_of::<T>()) — count is in "
                                        "ELEMENTS, zeroes sizeof(T)^2 bytes; cast ptr `as *mut u8`",
                                    )
                                )

        elif node.type == "match_arm":
            pat = node.child_by_field_name("pattern")
            val = node.child_by_field_name("value")
            if pat is not None and val is not None:
                ptext = src[pat.start_byte : pat.end_byte]
                if (
                    ptext.lstrip().startswith(b"Err(")
                    and _expr_is_unit_or_empty(val, src)
                    and not suppressed(node)
                ):
                    findings.append(
                        (
                            line_of(node),
                            "SWALLOWED_ERR",
                            "Err(...) arm does nothing — error silently discarded",
                        )
                    )

        elif node.type == "if_expression":
            cond = node.child_by_field_name("condition")
            cons = node.child_by_field_name("consequence")
            if (
                cond is not None
                and cons is not None
                and cond.type == "let_condition"
                and b"Err(" in src[cond.start_byte : cond.end_byte].split(b"=", 1)[0]
                and _block_is_empty(cons, src)
                and node.child_by_field_name("alternative") is None
                and not suppressed(node)
            ):
                findings.append(
                    (
                        line_of(node),
                        "SWALLOWED_ERR",
                        "`if let Err(...) {}` — error silently discarded",
                    )
                )

        # Track attribute_items so a following mod/fn sees its attributes.
        attrs: list = []
        for c in node.children:
            if c.type == "attribute_item":
                attrs.append(c)
                continue
            walk(c, attrs)
            attrs = []

    walk(tree.root_node, [])
    return findings


# --- cache -------------------------------------------------------------------


def _script_hash() -> str:
    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()[:16]


def _load_cache() -> dict:
    try:
        data = json.loads(CACHE_PATH.read_text())
        if data.get("gen") == _script_hash():
            return data.get("files", {})
    except (OSError, ValueError):
        pass
    return {}


def _save_cache(files: dict) -> None:
    try:
        CACHE_PATH.parent.mkdir(parents=True, exist_ok=True)
        CACHE_PATH.write_text(json.dumps({"gen": _script_hash(), "files": files}))
    except OSError:
        pass


# --- main --------------------------------------------------------------------


def main(argv: list[str]) -> int:
    args = [a for a in argv if not a.startswith("--")]
    no_cache = "--no-cache" in argv or os.environ.get("VIBEOS_SCAN_NO_CACHE") == "1"
    lint_mode = "--lint" in argv

    if args:
        files = [Path(a).resolve() for a in args]
    else:
        files = list(iter_rust_files(discover_no_std_crates(RUST_WS)))

    cache = {} if no_cache else _load_cache()
    new_cache: dict = {}
    total = 0
    out_lines: list[str] = []
    for f in files:
        try:
            data = f.read_bytes()
        except OSError:
            continue
        h = hashlib.sha256(data).hexdigest()
        rel = str(f.relative_to(REPO)) if f.is_relative_to(REPO) else str(f)
        ent = cache.get(rel)
        if ent and ent.get("h") == h:
            findings = [tuple(x) for x in ent["f"]]
        else:
            findings = scan_source(data)
        new_cache[rel] = {"h": h, "f": findings}
        for line, kind, msg in findings:
            out_lines.append(f"{rel}:{line}: [{kind}] {msg}")
            total += 1

    for ln in sorted(out_lines):
        print(ln)
    if not lint_mode or total:
        print(f"scan_silent_failures: {total} findings across {len(files)} files", file=sys.stderr)
    if not no_cache:
        _save_cache(new_cache)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
