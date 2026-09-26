#!/usr/bin/env python3
"""Generate tools/cohab-registry.json — a SOURCE-LEVEL approximation of which C/C++
translation units have been cohab-flipped to Rust.

WHY THIS EXISTS / WHY IT IS NOT THE REAL ORACLE
------------------------------------------------
`verify_cohab_coverage.py` explains at length (read its docstring first) why the OBJECT
FILE, not the source tree, is ground truth for a cohab flip: only the linked object proves
what a Rust module actually EXPORTS, after every `-D`, every `#ifdef`, and every ABI
decoration. That tool needs a build. This script deliberately does NOT build anything — it
is a read-only, source-level approximation for use BEFORE a build exists (e.g. auditing an
upstream merge), and it inherits every blind spot that implies:

  * It cannot see a Rust module that exports a symbol under the WRONG NAME (msvcrt math.c's
    three silently-dead flips: `_rotl64`/`_rotr64` under a bare name, `math_error` needing
    `-Dmath_error=_invoke_matherr`, the `__adj_fdiv_m16i@4` stdcall decorations). A "paired"
    entry here does NOT prove the flip actually resolves at link time.
  * It cannot see a sibling-file provider (cnt-rust's `cabi/math.rs` owns symbols that look
    like they belong to a DIFFERENT flipped module) or a C shim TU compiled into the module
    target (msvcrt's `rust_file_varargs_shim.c` covers file.c's va_start trampolines). Its
    pairing is "which .rs plausibly owns this .c", not "what actually resolves it".
    Function-name matching via `#[no_mangle]` grep is exactly the technique that tool's
    docstring says produced 81 FALSE findings for this reason — treat any MEDIUM/LOW here
    as unverified.
    the same caveat.
  * A source file the SAME NAME as an unrelated crate's file (e.g. `utils.c` / `utils.rs`)
    is exactly the false-pairing mode this registry exists to AVOID — see the "unpaired"
    section of this docstring's caller (the report). This script never pairs on bare stem
    match alone; see METHOD below.
  * `file(GLOB ...)` source lists cannot be disabled by commenting one entry out (the glob
    still picks the file up unless it is moved/renamed), so a module built this way is
    invisible to this script's method entirely. Not scanned; not counted either way.
  * A file can be commented out for reasons that have NOTHING to do with Rust (arch
    exclusion, a stale port, an experimental removal). This script requires Rust-signal
    text (a `-rust` crate name, an `.rs` path, or the word "Rust") within the same
    comment block before it will call something a cohab flip at all.

METHOD
------
1. Walk every CMakeLists.txt / *.cmake file.
2. Track a per-file "block hint": the most recent COMMENTED line in an unbroken run of
   comment lines that mentions a `<name>-rust` crate and/or an `.rs` path (e.g.
   `# lpc/ — replaced by Rust (ntoskrnl-rust/src/lpc/)`). A blank line or a non-comment
   line resets the hint — it only applies to the contiguous comment block it appears in.
3. Within that same walk, a commented line whose first token (after stripping `#` and an
   optional `${REACTOS_SOURCE_DIR}/` prefix) is a bare path ending in `.c`/`.cpp` is a
   DISABLED SOURCE ENTRY. Two evidence sources are checked, inline first:
     a. INLINE annotation on the same line — `(→ crate-rust src/x/y.rs — ...)` or
        `— replaced by Rust (crate-rust src/x/y.rs)` — highest specificity, confidence HIGH
        if the resolved .rs file exists on disk.
     b. BLOCK hint (see step 2) if there is no inline annotation — confidence HIGH if the
        hint names a crate that exists, but the .rs file within it is not individually
        confirmed (a directory-level flip, e.g. all of `lpc/`).
     c. Neither: fall back to deriving `<module-dir-name>-rust` and checking it exists —
        confidence MEDIUM (stem/dir-name guess, no textual evidence at all).
     d. If (c) finds nothing either: rust owner is null, confidence LOW. This is the
        "honest unpaired" case — no invented pairing.
4. A prose line that mentions Rust but does NOT match the bare-path pattern (e.g. a
   sentence, or a directory-only entry like `lpc/`) is NEVER itself counted as a disabled
   TU — it only seeds the hint for step 3.
5. "Confirm actually disabled": for every resolved path, grep the same file (and, cheaply,
   the whole tree) for an UNCOMMENTED occurrence of the identical path string. A hit means
   the same source is ALSO compiled somewhere (a second module reusing the file, an
   arch-specific inclusion elsewhere, or this run's comment-detection false-negative) —
   flagged as `also_compiled_uncommented: true` and capped at confidence MEDIUM regardless
   of what step 3 found, because "disabled" is now in question.

USAGE
    python3 tools/gen_cohab_registry.py > tools/cohab-registry.json
    python3 tools/gen_cohab_registry.py --report   # human summary to stderr, no JSON
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

PATH_RE = re.compile(
    r"""^\#+\s*
        (?P<srcdir>\$\{REACTOS_SOURCE_DIR\}/)?
        (?P<path>[A-Za-z0-9_./-]+\.(?:c|cpp))
        \b
        \s*(?P<rest>.*)$
    """,
    re.VERBOSE,
)

CRATE_RE = re.compile(r"\b([A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)*-rust)\b")
RS_PATH_RE = re.compile(r"\b(src/[A-Za-z0-9_./-]+\.rs)\b")
RS_SIGNAL_RE = re.compile(r"\brust\b", re.IGNORECASE)

RUST_ROOT = ROOT / "sdk" / "lib" / "rust"


def _iter_cmake_files():
    yield from ROOT.rglob("CMakeLists.txt")
    yield from ROOT.rglob("*.cmake")


def _resolve_disabled_path(raw: str, cmake_file: Path, had_srcdir: bool) -> Path | None:
    """Resolve a commented-out source reference to a repo-relative path.

    `${REACTOS_SOURCE_DIR}` is the repo root, so a `${REACTOS_SOURCE_DIR}/x/y.c` reference
    resolves against ROOT regardless of which CMakeLists it appears in. A bare relative name
    (no prefix) resolves against the CMake FILE's own directory, matching CMake's own rule.
    """
    if had_srcdir:
        return Path(raw)
    candidate = (cmake_file.parent / raw).resolve()
    try:
        rel = candidate.relative_to(ROOT)
    except ValueError:
        return None
    return rel


def _crate_and_rs_from_text(text: str) -> tuple[str | None, str | None]:
    crate = None
    rs = None
    m = CRATE_RE.search(text)
    if m:
        crate = m.group(1)
    m = RS_PATH_RE.search(text)
    if m:
        rs = m.group(1)
    return crate, rs


def _full_rs_path(crate: str, rs: str | None) -> Path | None:
    """Resolve a comment's `src/x/y.rs` reference to an actual file under the crate.

    Usually the comment's path IS the crate-relative path (dhcpcsvc-rust's
    `src/dhcp/adapter.rs` really lives at `dhcpcsvc-rust/src/dhcp/adapter.rs`). But some
    crates mirror the ORIGINAL module's own `src/` subdirectory inside their own `src/`
    (ws2_32-rust's real layout is `src/src/addrconv.rs`, not `src/addrconv.rs`), so a
    literal join misses a real file that exists under a different depth. Fall back to a
    basename search within the crate before concluding the file is missing.
    """
    crate_dir = RUST_ROOT / crate
    if rs is None or not crate_dir.is_dir():
        return None
    direct = crate_dir / rs
    if direct.is_file():
        return direct
    basename = Path(rs).name
    matches = [p for p in crate_dir.rglob(basename) if p.is_file()]
    if len(matches) == 1:
        return matches[0]
    return direct  # not found; caller checks .is_file()


def _derive_dir_crate(rel_path: Path) -> str | None:
    """`<module-dir-name>-rust` guess, e.g. ntoskrnl/lpc/reply.c -> lpc -> lpc-rust."""
    parent_name = rel_path.parent.name
    if not parent_name:
        return None
    guess = f"{parent_name}-rust"
    if (RUST_ROOT / guess).is_dir():
        return guess
    return None


def scan() -> dict:
    entries: dict[str, dict] = {}
    warnings: list[str] = []

    # Pass 1: collect every commented disabled-source candidate, with hint tracking.
    for cmake_file in _iter_cmake_files():
        try:
            lines = cmake_file.read_text(errors="replace").splitlines()
        except OSError:
            continue
        rel_cmake = cmake_file.relative_to(ROOT)
        hint_crate: str | None = None
        hint_rs: str | None = None
        hint_line: str | None = None

        for lineno, line in enumerate(lines, start=1):
            stripped = line.strip()
            if not stripped:
                hint_crate = hint_rs = hint_line = None
                continue
            if not stripped.startswith("#"):
                hint_crate = hint_rs = hint_line = None
                continue

            m = PATH_RE.match(stripped)
            # A real disabled-source line is BARE (nothing after the path) or is followed
            # immediately by annotation punctuation (`(...)`, `— ...`, `-> ...`, `→ ...`).
            # A path-shaped word at the START OF A PROSE SENTENCE ("smss_rust_stub.c was an
            # empty TU...") matches the same regex but is not a source-list entry at all —
            # require the punctuation gate so a paragraph never gets read as a CMake list.
            if m and m.group("rest") and not re.match(r"^[(\-—]|^->|^→", m.group("rest")):
                m = None
            if m:
                raw_path = m.group("path")
                rest = m.group("rest") or ""
                rel = _resolve_disabled_path(
                    raw_path, cmake_file, had_srcdir=bool(m.group("srcdir"))
                )
                if rel is None:
                    continue
                key = str(rel)

                inline_crate, inline_rs = _crate_and_rs_from_text(rest)
                evidence_kind = None
                crate = rs_ref = None
                if inline_crate:
                    crate, rs_ref = inline_crate, inline_rs
                    evidence_kind = "inline"
                    evidence_text = f"{rel_cmake}:{lineno}: {stripped}"
                elif hint_crate:
                    crate, rs_ref = hint_crate, hint_rs
                    evidence_kind = "block-hint"
                    evidence_text = f"{rel_cmake}: {hint_line!r} (block header)"
                else:
                    guess = _derive_dir_crate(rel)
                    if guess:
                        crate, rs_ref = guess, None
                        evidence_kind = "dir-name-guess"
                        evidence_text = f"derived '{guess}' from parent dir '{rel.parent.name}'"
                    else:
                        # No Rust signal ANYWHERE near this disabled line — this is an
                        # ordinary disabled/unbuilt C file (arch exclusion, missing
                        # dependency, dead feature), not a cohab-flip candidate. Skipping
                        # it is what keeps this registry from being "every commented-out
                        # source in the tree" (libtiff codecs, sndrec32 helpers, etc.) —
                        # exactly the noise this tool exists to filter OUT.
                        warnings.append(f"skip (no rust signal): {key} @ {rel_cmake}:{lineno}")
                        continue

                entry = entries.setdefault(
                    key,
                    {
                        "c_path": key,
                        "exists_on_disk": (ROOT / key).is_file(),
                        "disabled_at": [],
                        "rust_crate": None,
                        "rust_files": [],
                        "evidence": [],
                        "confidence": "LOW",
                        "also_compiled_uncommented": False,
                    },
                )
                entry["disabled_at"].append(f"{rel_cmake}:{lineno}")
                entry["evidence"].append({"kind": evidence_kind, "detail": evidence_text})
                if crate and not entry["rust_crate"]:
                    entry["rust_crate"] = crate
                    crate_exists = (RUST_ROOT / crate).is_dir()
                    entry["_crate_exists"] = crate_exists
                    if rs_ref:
                        full = _full_rs_path(crate, rs_ref)
                        entry["rust_files"].append(str(full.relative_to(ROOT)) if full else rs_ref)
                        entry["_rs_confirmed"] = full is not None and full.is_file()
                    else:
                        entry["_rs_confirmed"] = False
                    entry["_evidence_kind"] = evidence_kind
                continue

            # Not a path line: maybe it seeds/updates the block hint.
            if RS_SIGNAL_RE.search(stripped):
                c, r = _crate_and_rs_from_text(stripped)
                if c:
                    hint_crate, hint_rs, hint_line = c, r, stripped
            # else: prose with no rust signal — leave hint as-is (still same comment run)

    # Pass 2: build an index of UNCOMMENTED source references, to check "also compiled".
    # Deliberately full-relative-path only, NOT basename — this codebase is full of
    # `api.c` / `util.c` / `hash.c` / `event.c` / `rpcserver.c` in unrelated directories
    # (dhcpcsvc, umpnpmgr, tcpsvcs, dnsrslvr all use one of these), so a basename-only
    # index flagged fully-flipped modules as "also compiled elsewhere" on pure name
    # coincidence with some unrelated TU. A full resolved path is specific enough that a
    # real hit means the same file, not a namesake.
    uncommented_refs: set[str] = set()
    for cmake_file in _iter_cmake_files():
        try:
            lines = cmake_file.read_text(errors="replace").splitlines()
        except OSError:
            continue
        for line in lines:
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            for m in re.finditer(
                r"(?P<srcdir>\$\{REACTOS_SOURCE_DIR\}/)?(?P<path>[A-Za-z0-9_./-]+\.(?:c|cpp))\b",
                stripped,
            ):
                resolved = _resolve_disabled_path(
                    m.group("path"), cmake_file, had_srcdir=bool(m.group("srcdir"))
                )
                if resolved is not None:
                    uncommented_refs.add(str(resolved))

    for key, entry in entries.items():
        if key in uncommented_refs:
            entry["also_compiled_uncommented"] = True

    # Pass 3: confidence.
    for entry in entries.values():
        evk = entry.pop("_evidence_kind", None)
        crate_exists = entry.pop("_crate_exists", False)
        rs_confirmed = entry.pop("_rs_confirmed", False)
        if entry["also_compiled_uncommented"]:
            confidence = "MEDIUM"
        elif evk == "inline" and crate_exists and (rs_confirmed or not entry["rust_files"]):
            confidence = "HIGH" if rs_confirmed else "MEDIUM"
        elif evk == "block-hint" and crate_exists:
            confidence = "HIGH"
        elif evk == "dir-name-guess" and crate_exists:
            confidence = "MEDIUM"
        else:
            confidence = "LOW"
        if entry["rust_crate"] is None:
            confidence = "LOW"
        entry["confidence"] = confidence

    return {"entries": entries, "warnings": warnings}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--report", action="store_true", help="print summary to stderr only")
    args = ap.parse_args()

    result = scan()
    entries = result["entries"]

    ordered = {k: entries[k] for k in sorted(entries)}
    for e in ordered.values():
        e["disabled_at"].sort()
        e["evidence"].sort(key=lambda x: x["detail"])
        e["rust_files"].sort()

    by_conf: dict[str, int] = {"HIGH": 0, "MEDIUM": 0, "LOW": 0}
    for e in ordered.values():
        by_conf[e["confidence"]] += 1

    if not args.report:
        print(json.dumps(ordered, indent=2, sort_keys=True))

    print(
        f"gen_cohab_registry: {len(ordered)} disabled-with-Rust-signal TUs "
        f"(HIGH={by_conf['HIGH']} MEDIUM={by_conf['MEDIUM']} LOW={by_conf['LOW']})",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
