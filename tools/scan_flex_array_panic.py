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

The OBJECT CODE can tell them apart. `panic_bounds_check(index, len, &Location)` is
handed the ARRAY LENGTH, so a constant length of 1 names the flexible-array shape and
any larger length is a real fixed array to leave alone. Where the length lives is the
only per-arch part: i386 PUSHES it (cdecl, the middle of three pushes), amd64 puts it
in `edx` (Win64's second argument register) and arm64 in `w1`. Each reader takes the
last constant written there in the call's own argument setup and forgets it at
anything else, so a length from some earlier, unrelated computation is never
attributed to the call.

EVERY ARCH, BECAUSE THE CORPUS IS NOT THE SAME BINARY THREE TIMES. This reader was
i386-only for its first nine sites, on the reasoning that only i386 encodes the length
as an immediate -- true of the PUSH, and irrelevant: the other two load it as an
immediate into a register just as reliably. What i386 alone cannot see is code that is
not in the i386 build: amd64 and arm64 build `atapi-legacy-rust` where i386 builds
`atapi-rust`, and `#[cfg(target_arch)]` bodies (all of arm64's HAL, every 64-bit-only
path) do not exist in an i386 object at all. A site there was invisible by construction.

It grades the BUILT ARTIFACT, so a stale `build-<arch>` reports the state of the
last build and not of the tree -- do a build before believing a drop to zero.

A HIT IS A SHAPE, NOT A BUG. The discriminator this reader cannot see is whether
the C over-allocates: `PRIVILEGE_SET`/`LPCP_DATA_INFO` do (real bugs), while
`PKNODE KeNodeBlock[1] = { &KiNode0 };` really is one element and its check merely
survives because the guard is a runtime variable. Read the C declaration first.

There is one SYSTEMATIC false-positive source, and it is arch-shaped: an array sized by
an arch-dependent constant that happens to equal 1 on one arch.
`MmSystemPageDirectory[PPE_PER_PAGE]` is `[PFN_NUMBER; 512]` on amd64/arm64 and
`[PFN_NUMBER; 1]` on i386 (mm.rs:1265), so it is flagged on i386 and nowhere else.
Check the declaration's LENGTH EXPRESSION before the length: a literal `1` is the
bindgen flexible-array shape, a named constant is an arch-sized real array. Three of
the first six sites graded fell this way. A finding on ONE arch only is the tell.

A SECOND false-positive shape exists only on the register-passing arches: a constant
length that is BRANCH-DERIVED. On a runtime-length slice LLVM may emit `cmp $1, %len;
je` into a block calling panic_bounds_check(1, 1) -- on that path it has just proved the
length is 1, so it materializes the constant. rtl-rust's `format_ipv6` and
`get_checksum` both do this on amd64. The tell is a sibling panic(0, 0) block nearby
(a [T; 1] cannot fail with length 0), and the source indexing a `&[T]`, not a field.

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
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

if __package__:
    from .binary_facts import BinaryFacts
else:
    from binary_facts import BinaryFacts

REPO = Path(__file__).resolve().parent.parent
RUST_WS = REPO / "sdk" / "lib" / "rust"
OBJDUMP = REPO / ".build-tools" / "llvm-mingw" / "bin" / "llvm-objdump"

ARCHES = ("i386", "amd64", "arm64")

# i386. ARGUMENT ORDER, and getting it wrong is why this reader over-reported. The
# callee is `core::panicking::panic_bounds_check(index, len, &Location)`, and cdecl
# pushes right-to-left, so the emission order is `push loc; push len; push index` and
# the LAST push is the INDEX. Keying on the last push therefore selected `index == 1`,
# not `len == 1`, while the docstring claimed the latter. Measured over the whole i386
# corpus: index==1 matched 10 sites, len==1 matched 5, and NOTHING had len==1 with a
# different index -- so the old criterion missed nothing and over-reported by five.
#
# The five it added were not near-misses. Each had its length in a REGISTER rather than
# an immediate, i.e. a runtime-length slice -- which fails the grading protocol's second
# test ("is the declared length a LITERAL 1") by construction. A `[T; 1]` cannot have a
# dynamic length. Reporting them sent graders to `rtl_strings` functions that index a
# plain slice and declare no array at all. The same holds on every arch: only a
# CONSTANT length counts.
#
# objdump appends `# imm = 0x570` to the 5-byte push form, so anchoring on end-of-line
# also skipped every LARGE immediate -- precisely the &Location argument.
_PUSH_CONST = re.compile(r"pushl\s+\$0x([0-9a-f]+)\s*(?:#.*)?$")
_PUSH_ANY = re.compile(r"\bpushl\s")
# `\t\t\t00000414:  IMAGE_REL_I386_DIR32\t.rdata` — the Location argument pushed just
# before a bounds check is a &core::panic::Location, and it lives at this offset into
# the object's .rdata.
_SYMBOL = re.compile(r"^[0-9a-f]+ <(.+)>:$")
_CARRY = re.compile(r"pushl|calll|IMAGE_REL|movl|leal")
_PANIC = "panic_bounds_check"

# amd64 / arm64. One instruction per line, `addr: <bytes> <mnemonic> <operands>`.
_RELOC = re.compile(r"^\s+[0-9a-f]+:\s+(IMAGE_REL_\S+)\s+(\S+)\s*$")
_X64_INSN = re.compile(r"^\s*[0-9a-f]+:\s+(?:[0-9a-f]{2} )+\s*(\S+)\s*([^#]*?)\s*(?:#.*)?$")
_A64_INSN = re.compile(r"^\s*[0-9a-f]+:\s+[0-9a-f]{8}\s+(\S+)\s*([^/]*?)\s*(?://.*)?$")
# The rest of a call's own argument setup: these may sit between the length and the
# call without ending the run. Anything else -- a branch, a store, an arithmetic op on
# some other value -- does.
_X64_CARRY = {"movl", "movq", "movb", "movw", "leaq", "leal", "xorl", "xorq", "callq"}
_A64_CARRY = {"mov", "movz", "movk", "movn", "orr", "adrp", "add", "bl"}
_X64_LEN_REGS = ("%edx", "%rdx", "%dx", "%dl")
_A64_LEN_REGS = ("w1", "x1")
_RUST_HASH = re.compile(r"17h[0-9a-f]{16}E$")
_LLVM_SUFFIX = re.compile(r"\.llvm\.\d+$")
# v0 mangling (`_RNvNtCs6vQPevndiDL_7ex_rust7sysinfo…`, which the i386 cargo cell emits)
# carries a crate DISAMBIGUATOR where legacy mangling carries a trailing hash.
_V0_DISAMBIGUATOR = re.compile(r"Cs[0-9A-Za-z]+_")


def normalize_symbol(sym: str, arch: str, *, decorated: bool = True) -> str:
    """The arch-neutral name of a symbol: the key the allowlist is written in.

    The same Rust function is `__ZN…17h<hash>E` on i386 and `_ZN…17h<other>E` on the
    64-bit targets (i386 decorates every C name with `_`, and the legacy-mangling hash
    covers the target), and a C-ABI export is `_Name@8` against `Name`. Keying the
    allowlist on raw names would need one entry per arch for one graded site, and
    would go stale on every rebuild that moved a hash."""
    if decorated and arch == "i386" and sym[:1] in ("_", "@"):
        sym = sym[1:]  # `_cdecl`/`_stdcall@N` and fastcall `@name@N`
    sym = _LLVM_SUFFIX.sub("", re.sub(r"@\d+$", "", sym))
    if sym.startswith("_R"):
        return _V0_DISAMBIGUATOR.sub("C", sym)
    return _RUST_HASH.sub("E", sym)


ALLOWLIST = REPO / "tools" / "flex_array_allowlist.txt"


def graded_ok(arch: str) -> set[tuple[str, str]]:
    """(crate, normalized symbol) pairs graded as correct bounds checks ON `arch`.

    An entry may be scoped: `@i386 crate: symbol` applies to i386 alone, `@amd64,arm64
    …` to those two, and an unscoped entry to every arch. Scoping is what makes the
    stale check honest per arch: a graded i386 site whose crate amd64 does not even
    build must not read as "the reader went blind" on amd64."""
    out = set()
    if not ALLOWLIST.is_file():
        return out
    for line in ALLOWLIST.read_text().splitlines():
        line = line.split("#", 1)[0].strip()
        if not line or ":" not in line:
            continue
        if line.startswith("@"):
            scope, _, line = line[1:].partition(" ")
            if arch not in scope.split(","):
                continue
        crate, sym = line.split(":", 1)
        # Written NORMALIZED, so no decoration to strip: stripping i386's `_` again
        # would turn `_ZN…` into `ZN…` and the entry would read as stale on i386 only.
        out.add((crate.strip(), normalize_symbol(sym.strip(), arch, decorated=False)))
    return out


def is_graded(graded: set[tuple[str, str]], crate: str, sym: str) -> bool:
    """`* : sym` grades a Rust function wherever it is linked: a translated library such
    as `network` is compiled into several images, and its mangled name already says
    which crate it is. The grading is about the function, not the image."""
    return (crate, sym) in graded or ("*", sym) in graded


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


def artifact_index(build: Path) -> dict[str, Path]:
    """Every `lib*.a` under the build tree -> the first path the walk meets it at.

    ONE walk for the whole corpus. This was a walk PER CRATE -- 240 full descents of
    the build tree, ~8 s of a run whose cached verdicts take well under one."""
    out: dict[str, Path] = {}
    for dirpath, dirnames, filenames in os.walk(build):
        # A crate's own cargo target dir under the build tree is intermediate
        # output, not the artifact the link consumes. PRUNE it rather than
        # filtering its paths afterwards: a post-filter has already descended
        # into the one tree a concurrent build is rewriting, so it can raise
        # FileNotFoundError out of the walk before it ever inspects a path.
        dirnames[:] = sorted(d for d in dirnames if not d.endswith("-target"))
        for name in filenames:
            if name.startswith("lib") and name.endswith(".a"):
                out.setdefault(name, Path(dirpath) / name)
    return out


def find_artifact(build: Path, crate: str, index: dict[str, Path] | None = None) -> Path | None:
    want = "lib" + crate.replace("-", "_") + ".a"
    return (artifact_index(build) if index is None else index).get(want)


def declared_cargo_images(build: Path, arch: str) -> list[tuple[str, Path]]:
    """[(cargo crate, linked image)] for every cargo-linked module this arch's build
    graph produces.

    THE BOOT PATH LIVES HERE. ntoskrnl, hal, win32k, ntdll, smss and csrss are not
    staticlibs a CMake target links: each is a cargo `-cargo` crate that links its own
    PE under `sdk/lib/rust/target/<crate>-<arch>-cargo-link/`, pulling the translated
    libraries in as rlibs. A reader of `.a` files therefore reported every one of them
    NOT BUILT, and a CSR capture-buffer panic of exactly this class shipped in csrss
    through that hole. The images are LINKED, so the panic call names its callee on the
    call line itself and there are no relocations -- the same register discipline reads
    them.

    Declared by `build.ninja`, not discovered by listing the target dir: that dir keeps
    every image any earlier configuration ever produced, and a verdict must not depend
    on what a worktree happens to have lying around."""
    ninja = build / "build.ninja"
    if not ninja.is_file():
        return []
    rx = re.compile(
        rf"sdk/lib/rust/target/([A-Za-z0-9_-]+)-{arch}-cargo-link/[A-Za-z0-9_.-]+"
        r"/release/[A-Za-z0-9_.-]+\.(?:exe|dll|sys|efi)\b"
    )
    found = {m.group(0): m.group(1) for m in rx.finditer(ninja.read_text(errors="replace"))}
    return sorted((crate, REPO / rel) for rel, crate in found.items())


_MEMBER = re.compile(r"^(?:\S*\((?P<mem>[^)]+)\)|(?P<obj>\S+)):\s+file format ")
_SECHDR = re.compile(r"^\s*(\d+)\s+(\S+)\s+([0-9a-f]+)\s")
_SYMTAB = re.compile(r"^\[\s*\d+\]\(sec\s+(-?\d+)\).*\s0x([0-9a-f]+)\s+(\S+)\s*$")
_CONTENTS = re.compile(r"^Contents of section (\S+):")


@dataclass
class _Member:
    """One archive member's read-only data, indexed the way its relocations name it.

    PER MEMBER, and that is the whole point. The first version dumped the ARCHIVE and
    concatenated every member's .rdata into one buffer, then indexed it with an offset
    that is section-relative WITHIN ONE MEMBER. ks-rust alone carries five 21-byte source
    paths across its members, so the wrong-base buffer decoded to a real-looking answer
    every time -- the failure was invisible precisely because the result stayed plausible.

    PER SECTION, for the same reason one level down. arm64 objects put every anonymous
    constant in its OWN `.rdata` section -- dozens per member, all with the same name --
    and a `&Location` is relocated against the anonymous SYMBOL, not against `.rdata`.
    Concatenating those would repeat the first defect exactly: a right-looking offset
    into the wrong base."""

    rdata: dict[int, bytes] = field(default_factory=dict)  # section number -> bytes
    syms: dict[str, tuple[int, int]] = field(default_factory=dict)  # name -> (section, value)

    def section_of(self, sym: str) -> tuple[bytes, int] | None:
        if sym == ".rdata" and len(self.rdata) == 1:
            return next(iter(self.rdata.values())), 0
        hit = self.syms.get(sym)
        if hit is None or hit[0] not in self.rdata:
            return None
        return self.rdata[hit[0]], hit[1]


def _objdump(*args: str) -> str:
    proc = subprocess.run([str(OBJDUMP), *args], capture_output=True, text=True, check=False)
    return proc.stdout if proc.returncode == 0 else ""


def _members(path: Path) -> dict[str, _Member]:
    """{archive member -> its .rdata sections and symbols}. Only --explain pays for this."""
    out: dict[str, _Member] = {}
    order: dict[str, list[int]] = {}
    names: set[str] = set()
    raw: dict[str, dict[int, bytearray]] = {}
    cur = ""
    for line in _objdump("-h", str(path)).splitlines():
        m = _MEMBER.match(line)
        if m:
            cur = m.group("mem") or m.group("obj")
            out.setdefault(cur, _Member())
            order.setdefault(cur, [])
            continue
        m = _SECHDR.match(line)
        # `.rdata` AND `.rdata$<name>`: ThinLTO promotes a constant shared across
        # codegen units into its own named section (`.rdata$anon.<hash>.0.llvm.<n>`),
        # and on arm64 that is where every source-path string ends up. Empty sections
        # are skipped because `-s` prints nothing for them, and the mapping below is
        # by order.
        is_rdata = m is not None and (m.group(2) == ".rdata" or m.group(2).startswith(".rdata$"))
        if m and cur and is_rdata and int(m.group(3), 16):
            # -h numbers sections from 0; the symbol table numbers them from 1.
            order[cur].append(int(m.group(1)) + 1)
            names.add(m.group(2))
    for line in _objdump("-t", str(path)).splitlines():
        m = _MEMBER.match(line)
        if m:
            cur = m.group("mem") or m.group("obj")
            continue
        m = _SYMTAB.match(line)
        if m and cur in out:
            out[cur].syms.setdefault(m.group(3), (int(m.group(1)), int(m.group(2), 16)))
    # `-s -j <name>...` prints every selected section in section order, same-named ones
    # included, so the Nth "Contents of section .rdata*" block is the Nth header above.
    buf: bytearray | None = None
    nth: dict[str, int] = {}
    selected = [a for n in sorted(names) for a in ("-j", n)]
    for line in (_objdump("-s", *selected, str(path)) if selected else "").splitlines():
        m = _MEMBER.match(line)
        if m:
            cur = m.group("mem") or m.group("obj")
            buf = None
            continue
        if _CONTENTS.match(line):
            i = nth.get(cur, 0)
            nth[cur] = i + 1
            secs = order.get(cur, [])
            buf = None
            if i < len(secs):
                buf = bytearray()
                raw.setdefault(cur, {})[secs[i]] = buf
            continue
        if buf is None:
            continue
        m = re.match(r"^\s*([0-9a-f]+)\s((?:[0-9a-f]{2,8}\s){1,4})\s", line)
        if m:
            buf.extend(bytes.fromhex(m.group(2).replace(" ", "")))
    for name, secs in raw.items():
        out[name].rdata = {k: bytes(v) for k, v in secs.items()}
    return out


def _is_source_path(b: bytes) -> bool:
    """A workspace source path as a panic Location spells it: `<crate>/src/<..>.rs`.
    Not `-rust/src/` -- the workspace has crates named `network` and `rtl-strings`."""
    return b"/src/" in b and b.endswith(b".rs")


def _path_candidates(blobs: list[bytes], slen: int) -> list[str]:
    """Every NUL-delimited .rdata string of exactly `slen` bytes that looks like one of
    our source paths."""
    out, seen = [], set()
    for rdata in blobs:
        for m in re.finditer(rb"[ -~]{4,}", rdata):
            for piece in m.group().split(b"\x00"):
                if len(piece) == slen and _is_source_path(piece) and piece not in seen:
                    seen.add(piece)
                    out.append(piece.decode())
    return out


def resolve_location(
    member: _Member, ref: tuple[str, int], *, ptr64: bool, others: list[_Member] | None = None
) -> str:
    """`file:line:col` for the &core::panic::Location that `ref` = (symbol, addend) names.

    The struct is {ptr, len, line, col}, pointer-sized ptr and len. In an unlinked
    object the ptr is relocated, so the filename must be recovered from .rdata itself.

    NEVER GUESS BETWEEN EQUALS. The first version of this searched for the first string
    whose length matched `len` and returned it -- and ks-rust has BOTH `ks-rust/src/
    filter.rs` and `ks-rust/src/swenum.rs` in one object, each exactly 21 bytes, so it
    confidently named the wrong file and sent the grader to a function containing no
    array index at all. A tool that points at the wrong source is worse than one that
    declines, because the reader has no way to tell. So: prefer the in-place relocation
    addend when it lands on a plausible string, else accept a unique candidate, else say
    the answer is ambiguous and list them."""
    found = member.section_of(ref[0])
    if found is None and ref[0] != ".rdata":
        # ThinLTO PROMOTES a constant shared across codegen units to a global
        # (`anon.<hash>.8.llvm.<n>`) and emits it in ONE member, so a check in another
        # member is relocated against a symbol it does not define. Follow the name;
        # `.rdata` is never followed, because every member has its own.
        for other in others or []:
            found = other.section_of(ref[0])
            if found is not None:
                member = other
                break
    if found is None:
        return f"<location relocated against {ref[0]}, defined in no member read>"
    rdata, base = found
    off = base + ref[1]
    fmt, size = ("<QQII", 24) if ptr64 else ("<IIII", 16)
    if off < 0 or len(rdata) < off + size:
        return ""
    addend, slen, line, col = struct.unpack_from(fmt, rdata, off)
    if not 1 <= slen <= 512 or not 1 <= line < 1_000_000:
        return ""
    if addend >= 0 and addend + slen <= len(rdata):
        cand = rdata[addend : addend + slen]
        if _is_source_path(cand) and cand.isascii() and b"\x00" not in cand:
            return f"{cand.decode()}:{line}:{col}"
    names = _path_candidates(list(member.rdata.values()), slen)
    if not names:
        # arm64 keeps each path string in its own section, often in another member.
        names = _path_candidates([b for o in others or [] for b in o.rdata.values()], slen)
    if len(names) == 1:
        return f"{names[0]}:{line}:{col}"
    if names:
        return f"AMBIGUOUS among {', '.join(sorted(names))} — at :{line}:{col}"
    return f"<unresolved file>:{line}:{col}"


def _image_rdata(path: Path) -> tuple[int, bytes]:
    """(VMA, bytes) of a LINKED image's .rdata: one section at one address."""
    base, buf = None, bytearray()
    for line in _objdump("-s", "-j", ".rdata", str(path)).splitlines():
        m = re.match(r"^\s*([0-9a-f]+)\s((?:[0-9a-f]{2,8}\s){1,4})\s", line)
        if m:
            if base is None:
                base = int(m.group(1), 16)
            buf.extend(bytes.fromhex(m.group(2).replace(" ", "")))
    return (base or 0, bytes(buf))


def resolve_va(image: tuple[int, bytes], va: int, *, ptr64: bool) -> str:
    """`file:line:col` for a &Location at virtual address `va` in a linked image, whose
    file pointer the linker has already resolved to an address in the same .rdata."""
    base, rdata = image
    fmt, size = ("<QQII", 24) if ptr64 else ("<IIII", 16)
    off = va - base
    if not rdata or off < 0 or len(rdata) < off + size:
        return f"<location at 0x{va:x}, outside .rdata>"
    fptr, slen, line, col = struct.unpack_from(fmt, rdata, off)
    soff = fptr - base
    if 1 <= slen <= 512 and soff >= 0 and soff + slen <= len(rdata):
        name = rdata[soff : soff + slen]
        if name.isascii():
            return f"{name.decode()}:{line}:{col}"
    return f"<unresolved file>:{line}:{col}"


# How many instructions of lead-up to keep for --explain. The useful signal is the
# address computation feeding the check plus any relocation naming a global, and
# those sit within a few instructions of the call.
_EXPLAIN_WINDOW = 14


@dataclass
class _Setup:
    """The argument setup seen so far for the next call, on one arch.

    `length` is the constant last written to the length slot, or None when the slot
    holds anything else (or nothing we saw). `loc` is the (symbol, addend) of the
    &Location, kept for --explain."""

    length: int | None = None
    loc: tuple[str, int] | None = None
    pushes: list[int | None] = field(default_factory=list)  # i386 only, newest last
    pending: int | None = None  # 64-bit: displacement/immediate awaiting its relocation
    after_call: bool = False
    page: int | None = None  # arm64, linked: the adrp page of x2


def _feed_i386(st: _Setup, line: str, linked: bool = False) -> str:
    """'' to keep going, 'reset' to drop the setup, 'call' when the panic call lands."""
    if _PUSH_ANY.search(line):
        m = _PUSH_CONST.search(line.rstrip())
        st.pushes.append(int(m.group(1), 16) if m else None)
        del st.pushes[:-3]
        return ""
    if _PANIC in line:
        # pushes == [loc, len, index]; only a CONSTANT len of 1 is a [T; 1].
        if len(st.pushes) == 3:
            st.length = st.pushes[1]
            if st.pushes[0] is not None:
                # Linked: the pushed Location is an absolute address.
                st.loc = ("@va" if linked else ".rdata", st.pushes[0])
        return "call"
    return "" if _CARRY.search(line) else "reset"


def _feed_x64(st: _Setup, line: str, arch: str, linked: bool = False) -> str:
    m = _RELOC.match(line)
    if m:
        kind, target = m.groups()
        if st.after_call:
            # The relocation of a call names its callee: the panic is the verdict, any
            # other callee clobbered every argument register we were tracking.
            return "call" if _PANIC in target else "reset"
        if st.pending is not None and (arch == "amd64" or kind.endswith("PAGEOFFSET_12A")):
            st.loc, st.pending = (target, st.pending), None
        return ""
    if st.after_call:
        # A call with no relocation after it: a LINKED image, whose call line already
        # named its callee (below). Whatever it was, it clobbered the argument registers.
        return "reset"
    m = (_X64_INSN if arch == "amd64" else _A64_INSN).match(line)
    if not m:
        return "" if not line.strip() else "reset"
    mn, ops = m.group(1), [o.strip() for o in m.group(2).split(",")]
    if mn not in (_X64_CARRY if arch == "amd64" else _A64_CARRY):
        return "reset"
    if mn in ("callq", "bl"):
        # In a linked image the callee is resolved onto the call line itself
        # (`callq 0x140019a67 <…panic_bounds_check…>`); in an object it is named by the
        # relocation on the next line, so wait for that.
        if _PANIC in line:
            return "call"
        st.after_call = True
        return ""
    if arch == "amd64":
        dst, src = ops[-1], ops[0]
        if dst in _X64_LEN_REGS:
            imm = re.fullmatch(r"\$0x([0-9a-f]+)", src) if mn.startswith("mov") else None
            st.length = int(imm.group(1), 16) if imm else None
        elif mn.startswith("lea") and dst == "%r8" and linked and "# 0x" in line:
            # A linked image: objdump resolved the target address onto the line.
            st.loc = ("@va", int(line.rsplit("# 0x", 1)[1].split()[0], 16))
        elif mn.startswith("lea") and dst == "%r8":
            # objdump omits a zero displacement: `leaq (%rip), %r8`.
            disp = re.fullmatch(r"(-?0x[0-9a-f]+)?\(%rip\)", src)
            st.pending = int(disp.group(1) or "0", 16) if disp else None
    else:
        dst = ops[0]
        if dst in _A64_LEN_REGS:
            rest = ", ".join(ops[1:])
            imm = (
                re.fullmatch(r"(?:[wx]zr, )?#0x([0-9a-f]+)", rest)
                if mn in ("mov", "movz", "orr")
                else None
            )
            st.length = int(imm.group(1), 16) if imm else None
        elif mn == "adrp" and dst == "x2" and linked:
            page = re.match(r"0x([0-9a-f]+)", ops[1]) if len(ops) > 1 else None
            st.page = int(page.group(1), 16) if page else None
        elif mn == "add" and dst == "x2":
            imm = re.fullmatch(r"#0x([0-9a-f]+)", ops[-1])
            st.pending = int(imm.group(1), 16) if imm else None
            if linked and st.page is not None and st.pending is not None:
                st.loc = ("@va", st.page + st.pending)
    return ""


def scan_object(
    path: Path, *, arch: str = "i386", explain: bool = False
) -> list[tuple[str, int, list[str]]]:
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
    # A LINKED image (a cargo-linked PE) has no relocations: calls name their callee on
    # the call line and the &Location is a resolved address.
    linked = path.suffix.lower() != ".a"
    members = {} if linked or not explain else _members(path)
    image = _image_rdata(path) if linked and explain else (0, b"")
    member = _Member()
    sym = "?"
    st = _Setup()
    window: list[str] = []
    hits = []
    for line in proc.stdout.splitlines():
        m = _MEMBER.match(line)
        if m:
            member = members.get(m.group("mem") or m.group("obj"), _Member())
            sym, st, window = "?", _Setup(), []
            continue
        m = _SYMBOL.match(line)
        if m:
            sym, st, window = m.group(1), _Setup(), []
            continue
        window.append(line.rstrip())
        del window[:-_EXPLAIN_WINDOW]
        verdict = (
            _feed_i386(st, line, linked) if arch == "i386" else _feed_x64(st, line, arch, linked)
        )
        if verdict == "call":
            if st.length == 1:
                where = ""
                if explain and st.loc and st.loc[0] == "@va":
                    where = resolve_va(image, st.loc[1], ptr64=arch != "i386")
                elif explain and st.loc:
                    where = resolve_location(
                        member, st.loc, ptr64=arch != "i386", others=list(members.values())
                    )
                ctx = list(window) if explain else []
                if where:
                    ctx = [f"    -> {where}   (from the panic Location)", *ctx]
                hits.append((sym, 1, ctx))
            st = _Setup()
        elif verdict == "reset":
            st = _Setup()
    return hits


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", required=True, choices=ARCHES)
    ap.add_argument("--lint", action="store_true", help="exit 1 when any site is found")
    ap.add_argument(
        "--explain",
        metavar="SUBSTR",
        help="dump the disassembly around each hit whose crate or symbol contains SUBSTR "
        "-- the flagged symbol is where the check was INLINED TO, so this is where the "
        "real array is named",
    )
    args = ap.parse_args(argv)

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
    images = declared_cargo_images(build, args.arch)
    missing, findings = [], []
    work: list[tuple[str, Path]] = []
    index = artifact_index(build)
    for crate in corpus:
        art = find_artifact(build, crate, index)
        if art is None:
            missing.append(crate)
        else:
            work.append((crate, art))
    for crate, image in images:
        if image.is_file():
            work.append((crate, image))
        else:
            missing.append(f"{crate}({image.name})")
    # Content-addressed per artifact, like every other post-link scanner: three objdump
    # passes over ~290 archives and images cost 80-96s serially on EVERY invocation,
    # and a landing invokes this reader up to five times per arch on the same bytes.
    # `--explain` wants the disassembly window, which is not a fact worth storing.
    facts = (
        None
        if args.explain
        else BinaryFacts(
            REPO,
            namespace="flex-array-panic",
            implementation=[Path(__file__)],
            tools=[OBJDUMP],
            options={"arch": args.arch},
        )
    )

    def hits_for(item: tuple[str, Path]) -> list[tuple[str, int, list[str]]]:
        _crate, art = item

        def extract():
            return [list(h) for h in scan_object(art, arch=args.arch, explain=bool(args.explain))]

        raw = facts.get(art, extract) if facts is not None else extract()
        return [(sym, int(length), list(ctx)) for sym, length, ctx in raw]

    with ThreadPoolExecutor(max_workers=8) as pool:
        for (crate, _art), hits in zip(work, pool.map(hits_for, work), strict=True):
            for sym, _, ctx in hits:
                key = normalize_symbol(sym, args.arch)
                findings.append((crate, key))
                if args.explain and (args.explain in crate or args.explain in key):
                    print(f"\n=== {crate}: {key}")
                    for line in ctx:
                        print(f"    {line}")

    graded = graded_ok(args.arch)
    seen = sorted(set(findings))
    ungraded = [p for p in seen if not is_graded(graded, *p)]
    for crate, sym in ungraded:
        print(f"{crate}: {sym}: bounds check on a 1-element array (bindgen flexible array?)")
    for crate, sym in sorted(p for p in seen if is_graded(graded, *p)):
        print(f"{crate}: {sym}: GRADED OK (allowlisted) — a correct check; see its entry")
    # A stale entry means the site was fixed (drop the entry) or the reader stopped seeing
    # it (the gate has silently lost coverage). Either way it is not a clean result.
    seen_syms = {sym for _, sym in seen}
    stale = sorted(
        (c, sym)
        for c, sym in graded
        if (sym not in seen_syms if c == "*" else (c, sym) not in seen)
    )
    if stale:
        print(
            f"STALE ALLOWLIST ENTRIES on {args.arch} (no longer found — remove them, scope "
            "them to the arches that still have the site, or find out why the reader went "
            "blind): " + ", ".join(f"{c}: {s}" for c, s in stale),
            file=sys.stderr,
        )
    print(
        f"\n{args.arch}: {len(seen)} site(s) in {len({c for c, _ in seen})} module(s); "
        f"corpus {len(corpus)} staticlibs + {len(images)} cargo-linked images declared, "
        f"{len(missing)} not built",
        file=sys.stderr,
    )
    if missing:
        # Not a silent skip: an unbuilt member is missing COVERAGE, and saying so
        # is the difference between "clean" and "did not look".
        print(f"NOT BUILT (no coverage): {' '.join(sorted(missing))}", file=sys.stderr)
    return 1 if (args.lint and (ungraded or stale)) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
