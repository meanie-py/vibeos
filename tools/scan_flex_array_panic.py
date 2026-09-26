#!/usr/bin/env python3
"""Find bindgen flexible-array bounds checks that survived into a built object.

bindgen models a C flexible array (`T arr[ANYSIZE_ARRAY]` / `arr[1]`) as `[T; 1]`,
so a Rust `arr[i]` with a runtime `i` emits a bounds check that PANICS for every
i >= 1 while the C intentionally over-allocates. Source grep cannot separate that
from an index into a genuinely 1-element array, and it misses the shape that is
hardest to see by eye:

    *RootCell = addr_of_mut!((*p).List[i]) as PHCELL_INDEX

which reads as address-taking. It is not: the index inside the place expression
still bounds-checks, and `&raw mut` does not suppress it. cmlib's CmpSelectLeaf
shipped three of those in the kernel.

The OBJECT CODE can tell them apart. A `panic_bounds_check` call is preceded by a
push of the ARRAY LENGTH, so a length of 1 names the flexible-array shape and any
larger length is a real fixed array to leave alone.

It grades the BUILT ARTIFACT, so a stale `build-<arch>` reports the state of the
last build and not of the tree -- do a build before believing a drop to zero.

A HIT IS A SHAPE, NOT A BUG. The discriminator this reader cannot see is whether
the C over-allocates: `PRIVILEGE_SET`/`LPCP_DATA_INFO` do (real bugs), while
`PKNODE KeNodeBlock[1] = { &KiNode0 };` really is one element and its check merely
survives because the guard is a runtime variable. Read the C declaration first.

There is one SYSTEMATIC false-positive source, and it is a direct consequence of
this being i386-only: an array sized by an arch-dependent constant that happens to
equal 1 on 32-bit. `MmSystemPageDirectory[PPE_PER_PAGE]` is `[PFN_NUMBER; 512]` on
amd64/arm64 and `[PFN_NUMBER; 1]` on i386 (mm.rs:1265), so it is flagged here and
would not be flagged at all on a 64-bit read. Check the declaration's LENGTH
EXPRESSION before the length: a literal `1` is the bindgen flexible-array shape, a
named constant is an arch-sized real array. Three of the first six sites graded
fell this way.

THE SYMBOL IS WHERE THE CHECK ENDED UP, NOT WHERE IT WAS WRITTEN. A check is
attributed to the function it was INLINED INTO, which can contain no index at all:
`PathIsExeAW` is a four-line dispatcher (`if unicode { PathIsExeW(p) } else {
PathIsExeA(p) }`) and its only reachable array is `static EXTS: [[WCHAR; 4]; 7]`,
a genuine 7-element table. The flagged check came from a callee. So when a symbol
names no length-1 candidate, that is EXPECTED and not a sign the finding is bogus
-- read the disassembly's call context, or rebuild the crate with inlining
suppressed to re-attribute it. Sites of this shape cost far more per site than the
rest; 95 sites across 15 modules were graded by reading declarations, and the
residue that resisted (`KspInstallInterface`, `WSPSelect`, `DoAdminUnlock`,
`LsaApLogonUserEx2`, two `rtl_strings` symbols) all had exactly this shape.

TWO GRADING AIDS AND WHAT EACH IS WORTH. Pairing bindgen's length-1 FIELD
declarations against non-constant index expressions on those fields finds the
source site the object code cannot name -- but it matches BY FIELD NAME, and a
name is not one of the two tests. Twice that produced a confident false positive:
kernel32's `cStreamName` is `[WCHAR; 296]` in WIN32_FIND_STREAM_DATA against a
`[WCHAR; 1]` of the same name elsewhere, and `WellKnownSids[i].Sid.SubAuthority`
is MAX_SID's real `[DWORD; 15]` sitting two lines from a genuine `SID.SubAuthority`
in the same function. The aid narrows candidates; the struct still has to be
identified and its C declaration read. The object scanner and the source aid are
complementary, not redundant: the object side proves a check survived inlining,
the source side says where it is, and neither alone was sufficient.

A FLEXIBLE-ARRAY HELPER THAT ALREADY EXISTS IS A SEARCH PATTERN. Three files had
the class recognised locally and the fix left half-applied: afd's `select.rs` used
`AFD_HANDLES(x)` two lines from sites still spelled `Handles[i]`; shell32's
`printers_enum.rs` had the correct walk four lines above the broken one in the same
if/else; advapi32's `security.rs:3592` walked the DESTINATION correctly while the
source expression on the next line still indexed. Where such a helper exists, grep
for the siblings that did not get it.

PREFER `(&raw mut (*X).Field[0]).add(i)` TO A CAST. A constant index 0 into a
`[T; 1]` is checked at compile time, so it emits no panic branch, and it yields the
element type WITHOUT naming it. The `.cast::<Elem>()` spelling names the type, and
naming it is a semantic claim at every site: `Privileges` and `CurrentPrivileges`
in lsasrv hold different `_LUID` types, the field access had been deriving the
right one, and hardcoding one broke the build.

CORPUS IS DECLARED, NOT DISCOVERED. Globbing `build-<arch>` would grade whatever
binaries a worktree happens to carry -- a verdict that depends on state outside
the diff (field manual, "A gate can be RED for reasons that are not in the tree
at all"). The corpus is the set of workspace crates that declare a staticlib, and
a member missing from the build dir is reported, never skipped silently.
"""

from __future__ import annotations

import argparse
import os
import re
import struct
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_WS = REPO / "sdk" / "lib" / "rust"
OBJDUMP = REPO / ".build-tools" / "llvm-mingw" / "bin" / "llvm-objdump"

# The immediate pushed just before the call is the array length. i386 pushes it;
# on other arches the length lands in an argument register, so this reader is
# i386-only by construction and says so rather than silently grading nothing.
# ARGUMENT ORDER, and getting it wrong is why this reader over-reported. The callee is
# `core::panicking::panic_bounds_check(index, len, &Location)`, and cdecl pushes
# right-to-left, so the emission order is `push loc; push len; push index` and the LAST
# push is the INDEX. Keying on the last push therefore selected `index == 1`, not
# `len == 1`, while the docstring claimed the latter. Measured over the whole i386
# corpus: index==1 matched 10 sites, len==1 matched 5, and NOTHING had len==1 with a
# different index -- so the old criterion missed nothing and over-reported by five.
#
# The five it added were not near-misses. Each had its length in a REGISTER rather than
# an immediate, i.e. a runtime-length slice -- which fails the grading protocol's second
# test ("is the declared length a LITERAL 1") by construction. A `[T; 1]` cannot have a
# dynamic length. Reporting them sent graders to `rtl_strings` functions that index a
# plain slice and declare no array at all.
#
# objdump appends `# imm = 0x570` to the 5-byte push form, so anchoring on end-of-line
# also skipped every LARGE immediate -- precisely the &Location argument.
_PUSH_CONST = re.compile(r"pushl\s+\$0x([0-9a-f]+)\s*(?:#.*)?$")
_PUSH_ANY = re.compile(r"\bpushl\s")
# `\t\t\t00000414:  IMAGE_REL_I386_DIR32\t.rdata` — the Location argument pushed just
# before a bounds check is a &core::panic::Location, and it lives at this offset into
# the object's .rdata.
_RDATA_RELOC = re.compile(r"^\s*([0-9a-f]+):\s+IMAGE_REL_I386_DIR32\s+\.rdata\s*$")
_SYMBOL = re.compile(r"^[0-9a-f]+ <(.+)>:$")
_CARRY = re.compile(r"pushl|calll|IMAGE_REL|movl|leal")
_PANIC = "panic_bounds_check"


ALLOWLIST = REPO / "tools" / "flex_array_allowlist.txt"


def graded_ok() -> set[tuple[str, str]]:
    """(crate, symbol) pairs graded as correct bounds checks -- see the allowlist file."""
    out = set()
    if not ALLOWLIST.is_file():
        return out
    for line in ALLOWLIST.read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line or ":" not in line:
            continue
        crate, sym = line.split(":", 1)
        out.add((crate.strip(), sym.strip()))
    return out


def declared_corpus() -> list[str]:
    """Crate names that declare a staticlib -- the workspace IS the declaration."""
    out = []
    for cargo in sorted(RUST_WS.glob("*/Cargo.toml")):
        text = cargo.read_text()
        if "staticlib" not in text:
            continue
        m = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
        if m:
            out.append(m.group(1))
    return out


def find_artifact(build: Path, crate: str) -> Path | None:
    want = "lib" + crate.replace("-", "_") + ".a"
    for dirpath, dirnames, filenames in os.walk(build):
        # A crate's own cargo target dir under the build tree is intermediate
        # output, not the artifact the link consumes. PRUNE it rather than
        # filtering its paths afterwards: a post-filter has already descended
        # into the one tree a concurrent build is rewriting, so it can raise
        # FileNotFoundError out of the walk before it ever inspects a path.
        dirnames[:] = sorted(d for d in dirnames if not d.endswith("-target"))
        if want in filenames:
            return Path(dirpath) / want
    return None


_MEMBER = re.compile(r"^(?:\S*\((?P<mem>[^)]+)\)|(?P<obj>\S+)):\s+file format ")


def _section_by_member(path: Path, section: str) -> dict[str, bytes]:
    """{archive member -> raw bytes of `section`}.

    PER MEMBER, and that is the whole point. The first version dumped the ARCHIVE and
    concatenated every member's .rdata into one buffer, then indexed it with an offset
    that is section-relative WITHIN ONE MEMBER. ks-rust alone carries five 21-byte source
    paths across its members, so the wrong-base buffer decoded to a real-looking answer
    every time -- the failure was invisible precisely because the result stayed plausible."""
    proc = subprocess.run(
        [str(OBJDUMP), "-s", "-j", section, str(path)],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return {}
    out: dict[str, bytearray] = {}
    cur: bytearray | None = None
    for line in proc.stdout.splitlines():
        m = _MEMBER.match(line)
        if m:
            cur = out.setdefault(m.group("mem") or m.group("obj"), bytearray())
            continue
        if cur is None:
            continue
        m = re.match(r"^\s*([0-9a-f]+)\s((?:[0-9a-f]{2,8}\s){1,4})\s", line)
        if m:
            cur.extend(bytes.fromhex(m.group(2).replace(" ", "")))
    return {k: bytes(v) for k, v in out.items()}


def _path_candidates(rdata: bytes, slen: int) -> list[str]:
    """Every NUL-delimited .rdata string of exactly `slen` bytes that looks like one of
    our source paths."""
    out, seen = [], set()
    for m in re.finditer(rb"[ -~]{4,}", rdata):
        blob, base = m.group(), m.start()
        for piece_off, piece in _split_pieces(blob, base):
            if len(piece) == slen and b"-rust/src/" in piece and piece not in seen:
                seen.add(piece)
                out.append(piece.decode())
            del piece_off
    return out


def _split_pieces(blob: bytes, base: int):
    off = 0
    for piece in blob.split(b"\x00"):
        yield base + off, piece
        off += len(piece) + 1


def resolve_location(rdata: bytes, off: int, addend: int | None = None) -> str:
    """`file:line:col` for the &core::panic::Location at `off` into .rdata.

    The struct is {ptr, len, line, col}. In an unlinked object the ptr is relocated, so
    the filename must be recovered from .rdata itself.

    NEVER GUESS BETWEEN EQUALS. The first version of this searched for the first string
    whose length matched `len` and returned it -- and ks-rust has BOTH `ks-rust/src/
    filter.rs` and `ks-rust/src/swenum.rs` in one object, each exactly 21 bytes, so it
    confidently named the wrong file and sent the grader to a function containing no
    array index at all. A tool that points at the wrong source is worse than one that
    declines, because the reader has no way to tell. So: prefer the relocation addend
    when it lands on a plausible string, else accept a unique candidate, else say the
    answer is ambiguous and list them."""
    if len(rdata) < off + 16:
        return ""
    _, slen, line, col = struct.unpack_from("<IIII", rdata, off)
    if not 1 <= slen <= 512 or not 1 <= line < 1_000_000:
        return ""
    if addend is not None and addend >= 0 and addend + slen <= len(rdata):
        cand = rdata[addend : addend + slen]
        if b"-rust/src/" in cand and cand.isascii() and b"\x00" not in cand:
            return f"{cand.decode()}:{line}:{col}"
    names = _path_candidates(rdata, slen)
    if len(names) == 1:
        return f"{names[0]}:{line}:{col}"
    if names:
        return f"AMBIGUOUS among {', '.join(sorted(names))} — at :{line}:{col}"
    return f"<unresolved file>:{line}:{col}"


# How many instructions of lead-up to keep for --explain. The useful signal is the
# address computation feeding the check plus any relocation naming a global, and
# those sit within a few instructions of the push.
_EXPLAIN_WINDOW = 14


def scan_object(path: Path, *, explain: bool = False) -> list[tuple[str, int, list[str]]]:
    """[(symbol, array_len, context)] for every bounds check whose array length is 1.

    `context` is empty unless `explain`. WHY IT EXISTS: the object scanner attributes a
    check to the symbol it was INLINED INTO, so a flagged symbol routinely contains no
    index at all -- `rtl_strings::determine_dos_path_name_type_ustr` indexes a SLICE and
    declares no `[T; 1]` anywhere. Grading those from source is not slow, it is
    impossible: the source of the flagged symbol is the wrong source. The disassembly
    around the check is the only place the real array is named, so the reader that found
    the site is the right thing to ask, rather than a hand-run objdump recipe."""
    proc = subprocess.run(
        [str(OBJDUMP), "-dr", str(path)], capture_output=True, text=True, check=False
    )
    if proc.returncode != 0:
        raise RuntimeError(f"{OBJDUMP} failed on {path}: {proc.stderr.strip()[:200]}")
    by_member = _section_by_member(path, ".rdata") if explain else {}
    rdata = b""
    sym = "?"
    # The last three pushes, newest last, as (constant value or None).
    pushes: list[int | None] = []
    window: list[str] = []
    hits = []
    for line in proc.stdout.splitlines():
        m = _MEMBER.match(line)
        if m:
            rdata = by_member.get(m.group("mem") or m.group("obj"), b"")
            sym, pushes, window = "?", [], []
            continue
        m = _SYMBOL.match(line)
        if m:
            sym = m.group(1)
            pushes = []
            window = []
            continue
        window.append(line.rstrip())
        del window[:-_EXPLAIN_WINDOW]
        if _PUSH_ANY.search(line):
            m = _PUSH_CONST.search(line.rstrip())
            pushes.append(int(m.group(1), 16) if m else None)
            del pushes[:-3]
            continue
        if _PANIC in line:
            # pushes == [loc, len, index]; only a CONSTANT len of 1 is a [T; 1].
            if len(pushes) == 3 and pushes[1] == 1:
                loc_off = pushes[0]
                where = (
                    resolve_location(rdata, loc_off, struct.unpack_from("<I", rdata, loc_off)[0])
                    if (explain and loc_off is not None and len(rdata) >= loc_off + 16)
                    else ""
                )
                ctx = list(window) if explain else []
                if where:
                    ctx = [f"    -> {where}   (from the panic Location)", *ctx]
                hits.append((sym, 1, ctx))
            pushes = []
            continue
        if not _CARRY.search(line):
            pushes = []
    return hits


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", default="i386", help="only i386 encodes the length as a push")
    ap.add_argument("--lint", action="store_true", help="exit 1 when any site is found")
    ap.add_argument(
        "--explain",
        metavar="SUBSTR",
        help="dump the disassembly around each hit whose crate or symbol contains SUBSTR "
        "-- the flagged symbol is where the check was INLINED TO, so this is where the "
        "real array is named",
    )
    args = ap.parse_args(argv)

    if args.arch != "i386":
        print(
            f"scan_flex_array_panic: {args.arch} passes the array length in a register, "
            "not a push; this reader is i386-only. Refusing to grade nothing.",
            file=sys.stderr,
        )
        return 2
    if not OBJDUMP.is_file():
        print(f"MISSING TOOL: {OBJDUMP}", file=sys.stderr)
        return 2
    build = REPO / f"build-{args.arch}"
    if not build.is_dir():
        print(
            f"MISSING BUILD: {build}\nfix: ./vibeos-cli build --arch {args.arch}", file=sys.stderr
        )
        return 2

    corpus = declared_corpus()
    missing, findings = [], []
    for crate in corpus:
        art = find_artifact(build, crate)
        if art is None:
            missing.append(crate)
            continue
        for sym, _, ctx in scan_object(art, explain=bool(args.explain)):
            findings.append((crate, sym))
            if args.explain and (args.explain in crate or args.explain in sym):
                print(f"\n=== {crate}: {sym}")
                for line in ctx:
                    print(f"    {line}")

    graded = graded_ok()
    seen = sorted({(c, s) for c, s in findings})
    ungraded = [p for p in seen if p not in graded]
    for crate, sym in ungraded:
        print(f"{crate}: {sym}: bounds check on a 1-element array (bindgen flexible array?)")
    for crate, sym in sorted(p for p in seen if p in graded):
        print(f"{crate}: {sym}: GRADED OK (allowlisted) — correct check on a real 1-element array")
    # A stale entry means the site was fixed (drop the entry) or the reader stopped seeing
    # it (the gate has silently lost coverage). Either way it is not a clean result.
    stale = sorted(graded - set(seen))
    if stale:
        print(
            "STALE ALLOWLIST ENTRIES (no longer found — remove them, or find out why the "
            "reader went blind): " + ", ".join(f"{c}: {s}" for c, s in stale),
            file=sys.stderr,
        )
    print(
        f"\n{len(set(findings))} site(s) in {len({c for c, _ in findings})} module(s); "
        f"corpus {len(corpus)} declared, {len(missing)} not built",
        file=sys.stderr,
    )
    if missing:
        # Not a silent skip: an unbuilt member is missing COVERAGE, and saying so
        # is the difference between "clean" and "did not look".
        print(f"NOT BUILT (no coverage): {' '.join(sorted(missing))}", file=sys.stderr)
    return 1 if (args.lint and (ungraded or stale)) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
