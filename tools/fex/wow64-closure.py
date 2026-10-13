#!/usr/bin/env python3
"""Compute the transitive import closure of an i386 PE, for staging into SysWOW64.

Why this exists
---------------
A WoW64 guest's DLLs are loaded by OUR loader (ntdll-rust ldrpe.rs). When a DLL
it needs is absent, `LdrpLoadImportModule` fails and the loader prints

    LDR: Skipping missing import <dll> for <module> (status c0000135)

and then leaves that module's IAT slots UNBOUND. The image still loads, so the
failure is silent until the guest actually CALLS through one of those slots —
at which point it jumps to a file-relative RVA and dies somewhere unrelated.
(This killed gui32 with a guest AV @0x76AA4 on 2026-07-11, which is why the
"stage only user32+gdi32 and let the rest go missing" strategy was adopted: an
import you never load can't leave a half-bound module behind.)

That strategy stops working the moment a guest is linked against a real CRT:
msvcrt drags in advapi32 -> rpcrt4/secur32 -> ... and the guest DOES call into
them. The fix is not "stage more" — a bigger PARTIAL set has exactly the same
hazard, just later. The fix is to stage the COMPLETE closure, so no module is
ever left half-bound. This tool computes that closure, so it can't rot when an
import edge changes.

The import table is not the whole graph
--------------------------------------
An import-table walk sees only STATIC edges. A module loaded at runtime through
LoadLibrary is invisible to it, so the closure came out "complete" while missing
a module the guest genuinely needs — the tool hard-failed on a partial STATIC
closure and silently shipped a partial DYNAMIC one. That asymmetry is the same
landmine this file exists to defuse, just entered from the other side.

The concrete case: ws2_32 does not import its transport provider. It reads the
Winsock catalog out of the registry and LoadLibraryA's whatever the catalog
names (ws2_32-rust dprovide.rs, `(*Provider).DllHandle = LoadLibraryA(...)`).
So `msafd.dll` — every socket's actual WSP* implementation — was built, was
required, and was never staged. A 32-bit socket() could not work, and nothing
said so.

And the chain does not stop at the provider. msafd then reads its transport's
`HelperDllName` and LoadLibraryW's the WSH_* helper (msafd-rust
misc/helpers.rs). With msafd staged but `wshtcpip.dll` still missing, the guest
looked FIXED — msafd mapped, DllMain ran, DLL_PROCESS_ATTACH returned 1 — and
every socket() still failed with WSAEINVAL, because SockGetTdiName's
LoadLibraryW returned NULL and the error surfaces a full frame away from the
DLL that is actually absent. So the traversal follows dynamic edges
TRANSITIVELY: a module admitted as somebody's runtime target is itself asked
what it loads at runtime.

Those edges are DERIVED, not hand-listed: both hops are declared in
media/inf/*.inf, so this parses the same declarations the running system obeys.
Hand-listing would rot the moment someone adds a provider; parsing cannot.

Output: one "<name><TAB><path>" line per DLL, for mkramdisk.sh to mcopy.
Exits non-zero (loudly) if any import cannot be resolved to a built file —
a silently-omitted DLL is precisely the landmine we're defusing.
"""

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

# PE extensions ReactOS actually ships as importable modules. Missing one here
# means a phantom "not built" report (winspool.drv is a .drv, not a .dll).
PE_SUFFIXES = (".dll", ".drv", ".ocx", ".acm", ".cpl")

# Present in SysWOW64 already, staged explicitly by mkramdisk.sh.
PRESTAGED = {"ntdll.dll", "kernel32.dll", "user32.dll", "gdi32.dll", "ntdll_vista.dll"}

# Modules that load OTHER modules at runtime by consulting a registry table,
# mapped to the INF that declares that table and to the declarations THAT
# module reads. The loader never names the target in its import table, so only
# the declaration reveals the edge.
#
# The chain is more than one hop, which is why this is keyed by READER and not
# just by file: ws2_32 reads the Winsock catalog to find the transport PROVIDER
# (msafd), and msafd then reads its transport's HelperDllName to find the WSH_*
# HELPER (wshtcpip). Staging only the provider gets you a msafd that loads
# fine and then fails every socket() with WSAEINVAL, because
# SockGetTdiName -> LoadLibraryW(helper) returned NULL. Both hops are declared
# in the same INF, but by different values read by different modules.
DYNAMIC_LOADERS = {
    "ws2_32.dll": (
        "media/inf/nettcpip.inf",
        ("LibraryPath", "PackedCatalogItem", "AutodialDLL"),
    ),
    "msafd.dll": ("media/inf/nettcpip.inf", ("HelperDllName",)),
    # winmm opens every driver its Drivers32 key names (midimap, beepmidi, the
    # msacm32.drv wave mapper) when a program first touches audio or MIDI;
    # sndrec32 and mplay32 did, and each was refused in an x86-64 guest.
    "winmm.dll": ("boot/bootdata/hivesft.inf", ("Drivers32",)),
}

# A declaration spells its DLL either as a plain string value or packed into a
# binary blob (a WSAPROTOCOL_INFO whose leading field is the NUL-terminated
# path). Both forms appear in one file, so both are read.
_INF_STRING_DECLS = {
    "LibraryPath": re.compile(r'"LibraryPath",0x00000000,"([^"]+)"'),
    "HelperDllName": re.compile(r'"HelperDllName",0x00000000,"([^"]+)"'),
    # ws2_32 rasdial.rs tolerates this one being absent (it null-checks the
    # handle), so it is not a crash waiting to happen the way the helper was.
    # It is staged anyway: it is DECLARED, it is BUILT, and "the importer
    # happens to cope" is the reasoning that left msafd unstaged.
    "AutodialDLL": re.compile(r'"AutodialDLL",0x00000000,"([^"]+)"'),
    # Anchored at the line start, so a commented-out driver (";HKLM,...") is
    # not an edge.
    "Drivers32": re.compile(
        r'^HKLM,"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Drivers32",'
        r'"[^"]+",0x00000000,"([^"]+)"',
        re.M,
    ),
}
_INF_PACKED_DECLS = {
    "PackedCatalogItem": re.compile(r'"PackedCatalogItem",0x00000001,\\\n((?:[0-9A-Fa-f, \\\n]+))'),
}


def declared_modules(project_dir: Path, inf_rel: str, kinds: tuple) -> dict:
    """Map lowercase DLL name -> how it was declared, from one INF.

    `kinds` selects which declarations to read, because a single INF declares
    edges belonging to DIFFERENT readers (see DYNAMIC_LOADERS). Returns a dict
    rather than a set so an unresolved module can name the exact declaration
    that demanded it, the way a missing import names its importer.
    """
    inf = project_dir / inf_rel
    if not inf.is_file():
        # Fail loud rather than silently computing a static-only closure: a
        # missing INF means the derivation is not happening, not that there
        # are no dynamic edges.
        sys.exit(f"wow64-closure: catalog INF not found: {inf}")
    text = inf.read_text(errors="replace")

    found = {}
    for kind in kinds:
        if kind in _INF_STRING_DECLS:
            for m in _INF_STRING_DECLS[kind].finditer(text):
                name = Path(m.group(1).replace("\\", "/")).name.lower()
                found[name] = f"{inf_rel} {kind}"
        elif kind in _INF_PACKED_DECLS:
            for m in _INF_PACKED_DECLS[kind].finditer(text):
                octets = re.findall(r"\b([0-9A-Fa-f]{2})\b", m.group(1))
                raw = bytes(int(o, 16) for o in octets).split(b"\x00", 1)[0]
                path = raw.decode("latin1").replace("\\", "/")
                if path:
                    found[Path(path).name.lower()] = f"{inf_rel} {kind}"
        else:
            sys.exit(f"wow64-closure: unknown INF declaration kind: {kind}")
    if not found:
        # A reader listed in DYNAMIC_LOADERS whose declarations yield nothing
        # means the INF changed shape under the regex — the exact silent
        # partial-closure this derivation exists to prevent.
        sys.exit(
            f"wow64-closure: {inf_rel} declared no {'/'.join(kinds)} entry — "
            "the INF changed shape and the dynamic edge is no longer derived"
        )
    return found


def find_objdump(project_dir: Path) -> str:
    bundled = project_dir / ".build-tools/llvm-mingw/bin/llvm-objdump"
    if bundled.exists():
        return str(bundled)
    from shutil import which

    found = which("llvm-objdump") or which("objdump")
    if not found:
        sys.exit("wow64-closure: no llvm-objdump found (need .build-tools/llvm-mingw)")
    return found


def stripped_twins(build_dir: Path) -> dict:
    """Build-relative posix path -> its stripped twin, for every module the
    strip step recorded (see index_build for why this is keyed by path)."""
    twins: dict[str, Path] = {}
    stripped_dir = build_dir / "boot/stripped"
    manifest = stripped_dir / "strip-manifest.json"
    if manifest.is_file():
        marker = f"/{build_dir.name}/"
        for key in json.loads(manifest.read_text()):
            if marker not in key:
                continue
            twin = stripped_dir / f"{hashlib.sha1(key.encode()).hexdigest()[:12]}-{Path(key).name}"
            if twin.is_file():
                twins[key.split(marker, 1)[1]] = twin
    return twins


def index_build(build_dir: Path, no_strip: bool = False) -> dict:
    """Map lowercase module name -> path, preferring the stripped variant.

    The build tree carries both a fat debug copy and a stripped copy
    (boot/stripped/<hash>-<name>). The stripped set is what the livecd ships and
    is ~2x smaller in aggregate; the ramdisk has finite room, so prefer it.
    With no_strip, keep the fat paths (stable across rebuilds).
    """
    # The stripped twin of a module is named after ITS OWN source path:
    # strip_cd_media.py writes `sha1(<path>)[:12]-<name>` and records each path
    # in strip-manifest.json. So the twin is looked up by path, never by name.
    # Keying by name alone was a real bug: the amd64 tree ALSO ships the i386
    # SysWOW64 set, stripped into the same directory, so build-amd64 holds both
    # an amd64 and an i386 `<hash>-kernel32.dll` (and three same-named
    # fusion.dll modules besides), and whichever iterdir() listed last won. The
    # amd64 guest DLL set (sysamd64) shipped an i386 kernel32 that way, and the
    # x86-64 guest's loader failed to bind it on an import only i386 has.
    # Manifest paths are the build's canonical mount (/tmp/vibeos/<build dir>/
    # ...), so they are matched on the part below the build directory's name.
    twins = {} if no_strip else stripped_twins(build_dir)

    index = {}
    for p in build_dir.rglob("*"):
        if p.suffix.lower() not in PE_SUFFIXES:
            continue
        if "CMakeFiles" in p.parts or "stripped" in p.parts:
            continue
        # tools/conformance/mk_writable_livecd.py stages a COPY of the livecd tree in a
        # `livecd-rw-*` temp dir inside this build dir, and deletes it when the ISO is done.
        # A copy is never the module's authority, and it can vanish between this index and
        # the read: the merge-queue gate builds i386 conformance media while the amd64
        # restage runs this, and the closure died on FileNotFoundError for the copy's
        # url.dll (two batches rejected CMAKE_CONFIGURE_FAIL, 2026-10-10).
        if any(part.startswith("livecd-rw-") for part in p.relative_to(build_dir).parts):
            continue
        # A module's BUILD DIRECTORY carries the module's full name, suffix and
        # all -- dll/win32/hhctrl.ocx/ is a directory, and the real 670K
        # hhctrl.ocx lives inside it. rglob yields the directory before its
        # children, so without this test setdefault() pins the name to the
        # FOLDER and the binary never gets a look-in. add_cd_file then stages a
        # directory onto the livecd, the guest loader cannot map it, and the
        # importing module is left half-bound -- the exact crash-at-first-call
        # this closure exists to prevent, arriving through the tool meant to
        # stop it. 10 such directories exist in a build-i386 tree.
        if not p.is_file():
            continue
        index.setdefault(p.name.lower(), p)

    # Overlay each module's own stripped twin over it.
    for name, fat in index.items():
        twin = twins.get(fat.relative_to(build_dir).as_posix())
        if twin is not None:
            index[name] = twin
    return index


# A module name as it appears in a string literal. Matched against the build
# index, which is the authority on what is a real module.
_PE_NAME_TOKEN = re.compile(r"[A-Za-z0-9_.\-]+\.(?:dll|ocx|drv|acm|cpl)", re.I)
_ASCII_RUN = re.compile(rb"[\x20-\x7e]{2,120}")
_WIDE_RUN = re.compile(rb"(?:[\x20-\x7e]\x00){2,120}")


def string_referenced(pe: Path, index: dict) -> set:
    """Module names appearing as LITERALS in `pe`, restricted to built modules.

    The import walk sees static edges and declared_modules() sees the dynamic
    edges an INF declares. Neither sees a plain `LoadLibraryW(L"riched20.dll")`
    in an application's own code, and four such modules -- riched20 (wordpad,
    charmap, eventvwr), gdiplus and mapi32 (mspaint), aclui (regedit) -- were
    built, were reached at runtime, and were never staged. A LoadLibrary that
    returns NULL does not announce itself the way a missing import does, so
    this class is quieter than the one the closure was built for.

    By default only ROOTS are scanned: shell32 alone names every control panel
    applet it can launch, so scanning libraries admits ~90 more modules. But a
    library's own runtime loads are real too: comctl32's Animate control
    LoadLibrary's msvfw32.dll at create time, and with msvfw32 unstaged every
    guest SysAnimate32 window failed to create (user32:class). So
    --library-literals scans every staged module. It costs ~25 MB STRIPPED
    (70 vs 45 MB, 180 modules, all resolvable, measured 2026-09-24), which the
    amd64 livecd takes. The arm64 FEX ramdisk is a fixed 140 MB image and stays
    roots-only.

    Both encodings are scanned because these literals are usually UTF-16. A
    literal can abut its neighbour in the string pool with no separator
    (regedit's reads as "Aaclui.dll"), so every suffix is tried and the index
    settles it -- a false hit would have to BE a real module name.
    """
    data = pe.read_bytes()
    found = set()

    def harvest(text: str) -> None:
        for m in _PE_NAME_TOKEN.finditer(text):
            tok = m.group().lower()
            while tok:
                if tok in index:
                    found.add(tok)
                    break
                tok = tok[1:]

    for m in _ASCII_RUN.finditer(data):
        harvest(m.group().decode("ascii"))
    for m in _WIDE_RUN.finditer(data):
        harvest(m.group()[::2].decode("ascii"))
    return found


_DELAY_NAME = re.compile(rb"[A-Za-z0-9_.\-]+\.(?:dll|ocx|drv|acm|cpl)\x00", re.I)


def delay_imports_of(pe: Path) -> list:
    """DLL names from `pe`'s delay-import descriptors.

    A delay import is as real an edge as an ordinary one: the first call
    through it LoadLibrary's the DLL, and if the DLL is absent the call
    raises, far from here. `objdump -p` does not show them for our images,
    because sdk/tools/gen-delaylib.py emits the ImgDelayDescr tables as plain
    .rdata and the linker never fills the PE delay-import directory. When
    delay-load was switched on (sdk/cmake/gcc.cmake) the import-table walk
    therefore lost every delay edge at once: the closure for user32_winetest
    fell from ~64 modules to 29, all of them silently.

    So the descriptors are found by shape, which works on stripped copies too
    (mkramdisk stages those, so the ___DELAY_DESC_* symbols cannot be relied
    on): {Attributes 1 (RVA form), DllName, hmod, IAT, INT as in-image RVAs,
    then three zero DWORDs}, with DllName pointing at a NUL-terminated module
    name. A standard delay directory has the same layout, so it is covered as
    well.
    """
    import struct

    data = pe.read_bytes()
    if data[:2] != b"MZ" or len(data) < 0x40:
        return []
    pe_off = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe_off : pe_off + 4] != b"PE\0\0":
        return []
    nsec, opt_size = struct.unpack_from("<H12xH", data, pe_off + 6)
    opt = pe_off + 24
    image_size = struct.unpack_from("<I", data, opt + 56)[0]
    secs = []
    for i in range(nsec):
        vsize, va, rsize, rptr, chars = struct.unpack_from(
            "<IIII12xI", data, opt + opt_size + 40 * i + 8
        )
        secs.append((va, max(vsize, rsize), rptr, rsize, chars))

    def file_off(rva: int):
        for va, span, rptr, rsize, _ in secs:
            if va <= rva < va + span and rva - va < rsize:
                return rptr + rva - va
        return None

    names = []
    for _va, _, rptr, rsize, chars in secs:
        if chars & 0x20000000:  # IMAGE_SCN_MEM_EXECUTE: descriptors live in data
            continue
        end = rptr + rsize - 32
        for off in range(rptr, end + 1, 4):
            if data[off] != 1:
                continue
            d = struct.unpack_from("<8I", data, off)
            if d[0] != 1 or d[5] or d[6] or d[7]:
                continue
            if not all(0 < r < image_size for r in d[1:5]):
                continue
            n = file_off(d[1])
            if n is None:
                continue
            m = _DELAY_NAME.match(data, n)
            if m:
                names.append(m.group()[:-1].decode("ascii").lower())
    return names


def forward_targets_of(pe: Path) -> list:
    """DLLs that `pe`'s export FORWARDERS point at.

    A forwarder is a load-time edge in the other direction: a module that binds
    advapi32!RegDeleteTreeW, which advapi32 forwards to advapi32_vista, makes the
    loader load advapi32_vista. The import walk never sees that edge, so when
    tools/win10_surface.py `forwards` gave the 0x502 core DLLs forwards to their
    *_vista siblings, SysWOW64 staged advapi32 and user32 without
    advapi32_vista/user32_vista, and every 32-bit guest died at load with
    STATUS_ENTRYPOINT_NOT_FOUND (msvcrt binds GetDpiForSystem through user32).
    Every forwarder counts, not just the ones some staged module binds today:
    the next rebuild may bind another.
    """
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from win10_surface import PEError, read_pe_file

    try:
        exports = read_pe_file(pe).exports
    except (PEError, OSError):
        return []
    return sorted(
        {e.forward.split(".", 1)[0].lower() + ".dll" for e in exports.values() if e.forward}
    )


def imports_of(objdump: str, pe: Path) -> list:
    out = subprocess.run([objdump, "-p", str(pe)], capture_output=True, text=True).stdout
    return [
        m.group(1).lower()
        for m in (re.match(r"\s*DLL Name:\s*(\S+)", ln) for ln in out.splitlines())
        if m
    ]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("roots", nargs="+", help="root PE(s) whose closure to compute")
    ap.add_argument("--build-dir", default="build-i386")
    ap.add_argument(
        "--include-prestaged",
        action="store_true",
        help="also emit ntdll/kernel32/user32/gdi32 (normally staged by name)",
    )
    ap.add_argument(
        "--no-strip",
        action="store_true",
        help="emit the fat build paths instead of boot/stripped/<hash>- copies "
        "(for consumers with their own strip step, e.g. the amd64 livecd flow, "
        "where the hashed names would go stale across i386 rebuilds)",
    )
    ap.add_argument(
        "--library-literals",
        action="store_true",
        help="also follow module names that appear as string literals in every "
        "staged LIBRARY, not just the roots (see string_referenced)",
    )
    ap.add_argument(
        "--emit-roots",
        action="store_true",
        help="also emit each root itself (its stripped twin when there is one), "
        "for a consumer staging programs together with their DLLs",
    )
    args = ap.parse_args()

    project_dir = Path(__file__).resolve().parent.parent.parent
    build_dir = Path(args.build_dir)
    if not build_dir.is_absolute():
        build_dir = project_dir / build_dir
    if not build_dir.is_dir():
        sys.exit(f"wow64-closure: no such build dir: {build_dir}")

    objdump = find_objdump(project_dir)
    index = index_build(build_dir, no_strip=args.no_strip)

    seen, missing = set(), {}
    out = {}
    stack = []
    dynamic = {}

    def admit(name: str, why: str) -> None:
        """Bring one module into the closure, however it was demanded.

        Static imports and catalog providers land here identically on purpose:
        a module reached only at runtime is no less required than one named in
        an import table, so it gets the same resolution and the same loud
        failure when it is missing.
        """
        if name in seen:
            return
        seen.add(name)
        if name not in index:
            missing.setdefault(name, []).append(why)
            return
        if name not in PRESTAGED or args.include_prestaged:
            out[name] = index[name]
        stack.append(index[name])

    def follow_dynamic(name: str) -> None:
        """Admit the modules `name` loads at runtime, if it is a known loader.

        Called whenever a module enters the closure, so the chain composes: the
        provider msafd admitted here is itself a loader, and admitting it drags
        in its own helper on the next hop.
        """
        spec = DYNAMIC_LOADERS.get(name)
        if spec is None:
            return
        inf_rel, kinds = spec
        for target, decl in sorted(declared_modules(project_dir, inf_rel, kinds).items()):
            dynamic.setdefault(target, decl)
            admit(target, f"{name} via {decl}")
            follow_dynamic(target)

    root_twins = {} if args.no_strip else stripped_twins(build_dir)
    for r in args.roots:
        p = Path(r)
        if not p.is_absolute():
            p = build_dir / r
        if not p.exists():
            sys.exit(f"wow64-closure: root not found: {p}")
        if args.emit_roots:
            try:
                rel = p.resolve().relative_to(build_dir.resolve()).as_posix()
            except ValueError:
                rel = None
            out[p.name.lower()] = root_twins.get(rel, p) if rel else p
        stack.append(p)
        follow_dynamic(p.name.lower())
        # The root's own runtime loads. admit() pushes each onto the stack, so
        # the newcomer's STATIC imports are then walked like any other module --
        # staging riched20 without its own closure would just move the
        # half-bound crash one module along.
        for name in sorted(string_referenced(p, index)):
            admit(name, f"{p.name} via string literal")
            follow_dynamic(name)

    while stack:
        cur = stack.pop()
        deps = imports_of(objdump, cur) + delay_imports_of(cur) + forward_targets_of(cur)
        if args.library_literals:
            deps += sorted(string_referenced(cur, index))
        for dep in deps:
            admit(dep, cur.name)
            # A runtime loader drags in its targets whenever it enters the
            # closure, including when it entered as somebody else's import.
            follow_dynamic(dep)

    if missing:
        # FAIL LOUD: a DLL we can't resolve is a module that will load half-bound
        # and crash at first call, far from here. Never stage a partial closure.
        for dep, importers in sorted(missing.items()):
            print(
                f"wow64-closure: UNRESOLVED import {dep} "
                f"(wanted by {', '.join(sorted(set(importers)))})",
                file=sys.stderr,
            )
        print(
            "wow64-closure: refusing to emit a PARTIAL closure — a half-bound module "
            "crashes at first call, not at load. Build the missing module(s) first.",
            file=sys.stderr,
        )
        return 1

    for name in sorted(out):
        print(f"{name}\t{out[name]}")
    total = sum(p.stat().st_size for p in out.values())
    print(
        f"wow64-closure: {len(out)} module(s), {total / 1e6:.1f} MB",
        file=sys.stderr,
    )
    for name, decl in sorted(dynamic.items()):
        print(f"wow64-closure: dynamic provider {name} (declared in {decl})", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
