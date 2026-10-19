#!/usr/bin/env python3
"""WS5 — cmake/ninja-free orchestrator for the cargo-driven PE cells.

This is the thin "driver" the cargo-drive-cmake-off roadmap (WS5, Phase 3) calls
for: it replaces the `add_cargo_module()` ninja custom-commands in
`sdk/cmake/gcc.cmake` with a plain dependency-ordered walk that

  1. self-hosts the host tools a cell's post-link needs (`native-pefixup`),
  2. runs cargo per cell (the exact invocation cmake emits — stable + the builtin
     triple on amd64/arm64; nightly + `-Z build-std` + the softfloat kernel json
     on i386),
  3. copies the cell's `.exe` out, and
  4. applies the KIND post-link (native-pefixup mode / i386 lld-forwarder fix /
     efi strip) — matching `set_module_type` / the per-KIND POST_BUILD.

…with **no** cmake configure and **no** ninja. It depends only on the cmake-off
work already landed: the cargo cells build their C floor / asm / resources /
import-libs in-cargo (WS1/WS2/WS3 + the committed-flag + spec2def/native-hpp
self-hosting), so a configured `build-<arch>/` is no longer a prerequisite for
the cells listed here.

MVP scope: `ntoskrnl` / `ntkrnlmp` (the kernel EXEs) prove the chain end-to-end;
the `CELLS` table + topo walk generalize to the full 44-cell set as cells are
verified cmake-off (add a `Cell(...)` with its KIND and any cargo-cell deps).

Usage:
    tools/cargo_orchestrate.py --arch i386 ntoskrnl
    tools/cargo_orchestrate.py --arch amd64 ntoskrnl ntkrnlmp
    tools/cargo_orchestrate.py --arch i386 --list
    tools/cargo_orchestrate.py --arch i386 ntoskrnl --dry-run
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# arch → the cargo custom-target json stem (== cargo's target/<stem>/ output dir)
ARCH_STEM = {
    "i386": "i686-kernel",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}
# i386 user-mode cells take the builtin triple, like amd64 (gcc.cmake:add_cargo_module).
I386_USER_STEM = "i686-pc-windows-gnu"
USER_KINDS = ("win32dll", "win32exe")
# arch → whether the produced PE is 64-bit (drives pefixup's -D_TARGET_PE64)
ARCH_PE64 = {"i386": False, "amd64": True, "arm64": True}


@dataclass(frozen=True)
class Cell:
    """One cargo-driven PE module — the data `add_cargo_module()` encodes."""

    target: str  # identity / CLI name
    crate: str  # cargo bin crate (the `<name>-cargo` dir under sdk/lib/rust)
    output: str  # produced filename (ntoskrnl.exe, win32k.sys, …)
    kind: str  # kernelexe|kerneldll|kernelmodedriver|win32dll|win32exe|efi|native
    deps: tuple[str, ...] = field(default=())  # cargo cells that must build first
    arches: tuple[str, ...] = ("i386", "amd64", "arm64")  # arches this cell exists on


# Verified cmake-off cells. KINDs mirror the add_cargo_module() calls. **deps is
# empty for every cell**: the inter-cell DAG the roadmap feared (import-lib
# ordering) is fully dissolved — each cell synthesizes the import libs it needs
# from `.spec` via the self-hosted spec2def+dlltool (`.import_lib()`), so no cell
# needs another cell's built `.dll`/`.a`. They build in any order / in parallel.
#
# The usermode spine (ntdll → csr servers → the boot exes) is cmake-off via the
# committed-`.res` fallback (WS3) + self-hosted import libs; ntdll's cntpr_residue
# + static_libs are empty (no c_from_compile_db, no cmake `.a`). `winsrv` (the
# last hold-out) is now cmake-off too: its 3 former cmake static libs are gone —
# libuuid.a → a 3-GUID c_shim (winsrv_guids.c), libwin32ksys.a → asm_archive over
# sys-stubs.S, libchkstk.a → dropped (inert). So the whole spine is cmake-off.
CELLS: dict[str, Cell] = {
    c.target: c
    for c in (
        # kernel EXEs — synth hal/kdcom/bootvid import libs in-cargo
        Cell("ntoskrnl", "ntoskrnl-cargo", "ntoskrnl.exe", "kernelexe"),
        Cell("ntkrnlmp", "ntkrnlmp-cargo", "ntkrnlmp.exe", "kernelexe"),
        # usermode spine — DLLs
        Cell("ntdll", "ntdll-cargo", "ntdll.dll", "win32dll"),
        Cell("ntdll_vista", "ntdll_vista-cargo", "ntdll_vista.dll", "win32dll"),
        Cell("csrsrv", "csrsrv-cargo", "csrsrv.dll", "win32dll"),
        Cell("basesrv", "basesrv-cargo", "basesrv.dll", "win32dll"),
        Cell("winsrv", "winsrv-cargo", "winsrv.dll", "win32dll"),
        # usermode spine — EXEs
        Cell("csrss", "csrss-cargo", "csrss.exe", "native"),
        Cell("smss", "smss-cargo", "smss.exe", "native"),
        Cell("lsass", "lsass-cargo", "lsass.exe", "win32exe"),
        Cell("winlogon", "winlogon-cargo", "winlogon.exe", "win32exe"),
        Cell("services", "services-cargo", "services.exe", "win32exe"),
        Cell("userinit", "userinit-cargo", "userinit.exe", "win32exe"),
        # ---- drivers (audit 2026-06-21) -------------------------------------
        # 24/25 driver cells are cmake-off-clean: rust_staticlib + self-hosted
        # .import_lib() from .spec, no cmake static_libs / c_from_compile_db /
        # resource_from_rc (hal carries only a committed .res). The kernel's
        # import-dep DLLs (hal/kdcom/bootvid) + the leaf drivers all qualify.
        # win32k joined 2026-06-21 (libwin32k_residue.a compiled in-cargo).
        Cell("hal", "hal-cargo", "hal.dll", "kerneldll"),
        # halmp.dll = the same HAL body with hal-rust's `smp` feature, shipped
        # alongside hal.dll so one media boots either flavor (freeldr `/HAL=`).
        # Both cells drive the SHARED `hal-link` recipe, so they differ by
        # exactly one cargo feature and the output name.
        Cell("halmp", "halmp-cargo", "halmp.dll", "kerneldll"),
        # kdcom.dll has 4 alternative provider crates (kdcom/kdgdb/kdcom-serial/
        # kdrosdbg) selected by config; kdcom-cargo is the default the standard
        # build ships. The others are cmake-off-clean too (same shape).
        Cell("kdcom", "kdcom-cargo", "kdcom.dll", "kerneldll"),
        Cell("bootvid", "bootvid-cargo", "bootvid.dll", "kerneldll"),
        Cell("beep", "beep-cargo", "beep.sys", "kerneldll"),
        Cell("null", "null-cargo", "null.sys", "kerneldll"),
        Cell("nmidebug", "nmidebug-cargo", "nmidebug.sys", "kerneldll"),
        Cell("pci", "pci-cargo", "pci.sys", "kerneldll"),
        Cell("pciide", "pciide-cargo", "pciide.sys", "kerneldll"),
        Cell("classpnp", "classpnp-cargo", "classpnp.sys", "kerneldll"),
        Cell("condrv", "condrv-cargo", "condrv.sys", "kerneldll"),
        Cell("hidparse", "hidparse-cargo", "hidparse.sys", "kerneldll"),
        Cell("mouclass", "mouclass-cargo", "mouclass.sys", "kerneldll"),
        Cell("kbdclass", "kbdclass-cargo", "kbdclass.sys", "kerneldll"),
        Cell("i8042prt", "i8042prt-cargo", "i8042prt.sys", "kerneldll"),
        Cell("swenum", "swenum-cargo", "swenum.sys", "kerneldll"),
        Cell("usbd", "usbd-cargo", "usbd.sys", "kerneldll"),
        Cell("processr", "processr-cargo", "processr.sys", "kerneldll"),
        Cell("framebuf", "framebuf-cargo", "framebuf.dll", "kerneldll"),
        Cell("netstack", "netstack-cargo", "netstack.sys", "kerneldll"),
        # arm64-only video/input drivers
        Cell("sysfb", "sysfb-cargo", "sysfb.sys", "kerneldll", arches=("arm64",)),
        Cell("arm64input", "arm64input-cargo", "arm64input.sys", "kerneldll", arches=("arm64",)),
        # win32k — cmake-off as of 2026-06-21 (libwin32k_residue.a compiled
        # in-cargo: c_from_compile_db + native-gendib + asm_archive). KIND
        # kernelmodedriver. byte-identical to cmake ×3 (bar the PE timestamp).
        Cell("win32k", "win32k-cargo", "win32k.sys", "kernelmodedriver"),
        # ---- boot loaders ---------------------------------------------------
        # Already cmake-off via WS1 DEEP (blrtl/libcntpr c_from_compile_db
        # committed-flags; no cmake static_libs — libmini_hal.a drop-tested out).
        # uefildr (efi, all 3 arches); rosload (native exe, i386+amd64 — arm64 is
        # UEFI-only). The efi post-link strips via the cross llvm-mingw strip.
        Cell("uefildr", "uefildr-cargo", "uefildr.efi", "efi"),
        Cell("rosload", "rosload-cargo", "rosload.exe", "native", arches=("i386", "amd64")),
    )
}

# The spine cells proven to build with NO configured build-<arch>/ (cmake-off).
# The kernel EXEs are proven separately. Used by --spine. winsrv joined
# 2026-06-21 (its 3 cmake static libs removed).
CMAKE_OFF_SPINE = (
    "ntdll",
    "ntdll_vista",
    "csrsrv",
    "basesrv",
    "winsrv",
    "csrss",
    "smss",
    "lsass",
    "winlogon",
    "services",
    "userinit",
)

# Drivers proven cmake-off (2026-06-21 audit). 24 were already clean
# (rust_staticlib + self-hosted import libs); win32k joined once its
# libwin32k_residue.a was compiled in-cargo (c_from_compile_db + native-gendib +
# asm_archive). sysfb/arm64input self-filter to arm64 via Cell.arches. Used by
# --drivers — the COMPLETE driver set is now cmake-off.
CMAKE_OFF_DRIVERS = (
    "win32k",
    "hal",
    "kdcom",
    "bootvid",
    "beep",
    "null",
    "nmidebug",
    "pci",
    "pciide",
    "classpnp",
    "condrv",
    "hidparse",
    "mouclass",
    "kbdclass",
    "i8042prt",
    "swenum",
    "usbd",
    "processr",
    "framebuf",
    "netstack",
    "sysfb",
    "arm64input",
)

# Boot loaders, cmake-off via WS1 DEEP. uefildr (efi) builds on all 3 arches;
# rosload (native) self-filters to i386+amd64 via Cell.arches. Used by --loaders.
CMAKE_OFF_LOADERS = ("uefildr", "rosload")


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def pinned_nightly() -> str:
    """The pinned nightly toolchain name, single-sourced from `.rust-nightly`.

    Nothing pinned the toolchain until 2026-07-12, so a bare `nightly` floated to
    whatever rustup last installed — and upstream has since removed the c_variadic
    API our variadic defs use, so a `rustup update` would break the build. Bump the
    pin in `.rust-nightly` (see rust-toolchain.toml for the stable half).
    """
    return (repo_root() / ".rust-nightly").read_text().strip()


def log(msg: str) -> None:
    print(f"[ws5] {msg}", flush=True)


# `--trace`: the orchestrator's own spawned commands, in order. The in-cargo
# (reactos-link engine) commands are captured separately via VIBEOS_LINK_TRACE
# (one file per cell) and stitched in by main().
_TRACE_CMDS: list[str] = []


def _trace(cmd: list[str], cwd: Path | None = None) -> None:
    prefix = f"cd {cwd} && " if cwd else ""
    _TRACE_CMDS.append(prefix + " ".join(shlex.quote(str(c)) for c in cmd))


def run(cmd: list[str], cwd: Path | None = None, dry: bool = False) -> None:
    shown = " ".join(str(c) for c in cmd)
    if dry:
        log(f"DRY  {shown}")
        return
    log(f"exec {shown}")
    _trace(cmd, cwd)
    subprocess.run(cmd, cwd=cwd, check=True)


def host_cc() -> str:
    """The host C compiler for building host tools (matches cmake's host build)."""
    return os.environ.get("CC", "cc")


def cross_strip(root: Path) -> str:
    """The CROSS strip for the efi KIND — the llvm-mingw `llvm-strip` (== cmake's
    CMAKE_STRIP). A host `strip` mis-handles the cross PE/EFI; the toolchain copy
    is what the loaders' cmake POST_BUILD used. Falls back to PATH llvm-strip."""
    tc = root / ".build-tools/llvm-mingw/bin/llvm-strip"
    if tc.is_file():
        return str(tc)
    for cand in (os.environ.get("STRIP"), "llvm-strip", "strip"):
        if cand and shutil.which(cand):
            return cand
    return "strip"


def build_pefixup(root: Path, arch: str, out_dir: Path, dry: bool = False) -> Path:
    """Self-host native-pefixup (sdk/tools/pefixup.c) for the target arch.

    TARGET_<host> satisfies compat.h's CONTEXT gate (pefixup is a host tool);
    _TARGET_PE64 selects PE32+ for the 64-bit targets (cmake's per-arch
    target_compile_definitions). Single .c, like spec2def/native-hpp.
    """
    exe = out_dir / f"native-pefixup-{arch}"
    cmd = [
        host_cc(),
        "-O2",
        "-DTARGET_amd64",  # host arch (x86_64) — compat.h CONTEXT/RUNTIME_FUNCTION
        "-I",
        str(root / "sdk/include/host"),
    ]
    if ARCH_PE64[arch]:
        cmd.append("-D_TARGET_PE64")
    src = root / "sdk/tools/pefixup.c"
    cmd += ["-o", str(exe), str(src)]
    # Rebuilt when its source or command changes: a kept binary outlived a new
    # post-link option (--dllname) and failed every cell that used it.
    compat = root / "dll/win32/dbghelp/compat.h"  # its one local include
    digest = hashlib.sha256(
        src.read_bytes() + compat.read_bytes() + "\0".join(cmd).encode()
    ).hexdigest()
    stamp = exe.with_name(exe.name + ".stamp")
    if exe.exists() and not dry and stamp.exists() and stamp.read_text().strip() == digest:
        return exe
    out_dir.mkdir(parents=True, exist_ok=True)
    run(cmd, dry=dry)
    if not dry:
        stamp.write_text(digest + "\n")
    return exe


def build_std(cell: Cell, arch: str) -> bool:
    """Only i386's kernel kinds still build core themselves (on the softfloat json), and
    only those with an i686-kernel.json: a generated driver cell has none and builds on
    the builtin triple like its body (gcc.cmake:add_cargo_module)."""
    if arch != "i386" or cell.kind in USER_KINDS:
        return False
    return (repo_root() / "sdk/lib/rust" / cell.crate / "i686-kernel.json").exists()


def cell_stem(cell: Cell, arch: str) -> str:
    """The --target stem: cargo's target/<stem>/ subdir."""
    return I386_USER_STEM if arch == "i386" and not build_std(cell, arch) else ARCH_STEM[arch]


def target_json(root: Path, cell: Cell, arch: str) -> Path:
    """A build-std cell's custom target spec (i386 kernel kinds only)."""
    return root / "sdk/lib/rust" / cell.crate / f"{ARCH_STEM[arch]}.json"


def cargo_target_dir(root: Path, cell: Cell, arch: str) -> Path:
    """The CARGO_TARGET_DIR a cell builds in.

    The i386 kernel json comes in two variants (345bab65b7a): kernel-mode cells add
    `-mmx,-sse,+soft-float` to keep x87 out of the kernel, and the user-mode cells and
    win32k stay hard-float for the f32-in-ST0 ABI at their C calls. Both are named
    i686-kernel.json, so in one target dir they would share `target/i686-kernel/`, and
    cargo does not fingerprint a target json's contents: the second variant would link
    against the first one's `core`. Each variant gets a directory of its own, keyed by
    the json's hash."""
    ws = root / "sdk/lib/rust"
    if not build_std(cell, arch):
        return ws / "target"
    digest = hashlib.sha256(target_json(root, cell, arch).read_bytes()).hexdigest()[:12]
    return ws / "target" / f"orchestrate-{arch}-{digest}"


def built_exe(root: Path, cell: Cell, arch: str) -> Path:
    """Where cargo drops a cell's linked .exe."""
    stem = cell_stem(cell, arch)
    bin_name = cell.crate.removesuffix("-cargo")
    return cargo_target_dir(root, cell, arch) / stem / "release" / f"{bin_name}.exe"


def cargo_build_many(
    root: Path,
    cells: list[Cell],
    arch: str,
    dry: bool = False,
    trace_dir: Path | None = None,
    force_off: bool = False,
) -> None:
    """ONE cargo invocation PER CELL, never one for many.

    A single `cargo build -p a -p b ...` UNIFIES FEATURES across every package it
    builds: the kernel cells' `panic_common` feature that calls `KeBugCheckEx` leaked
    into csrss/smss/csrsrv and they failed to link (undefined `KeBugCheckEx`) — each
    links fine alone. The cmake build never saw it because `add_cargo_module` runs
    one cargo per cell. Cells still share a target dir, so the common rlibs are built
    once per feature set and reused."""
    for c in cells:
        _cargo_build_group(
            root, [c], arch, cargo_target_dir(root, c, arch), dry, trace_dir, force_off
        )


def _cargo_build_group(
    root: Path,
    cells: list[Cell],
    arch: str,
    target_dir: Path,
    dry: bool,
    trace_dir: Path | None,
    force_off: bool,
) -> None:
    """One cargo invocation for cells that share a target spec and target dir."""
    ws = root / "sdk/lib/rust"
    stem = cell_stem(cells[0], arch)
    # Mirror gcc.cmake:add_cargo_module. i386's kernel kinds are the only build-std
    # cells left (they need the `x86-softfloat` kernel json, which has no builtin
    # triple); everything else links the BUILTIN triple on pinned stable, with the
    # `-C` flags that restore what the retired json carried. Keep in sync with
    # gcc.cmake + sdk/lib/rust/cargo-cell-args.sh.
    env = dict(os.environ)
    if build_std(cells[0], arch):
        # Every cell in a group has the same json bytes (cargo_target_dir's key).
        target = str(target_json(root, cells[0], arch))
        std_args = [
            "-Z",
            "build-std=core,alloc,compiler_builtins",
            "-Z",
            "build-std-features=compiler-builtins-mem",
        ]
        env["RUSTUP_TOOLCHAIN"] = pinned_nightly()
    else:
        target = stem
        std_args = []
        # link-self-contained is amd64-only: gnullvm rejects the flag outright.
        rustflags = "-C force-frame-pointers=yes"
        if arch in ("amd64", "i386"):
            rustflags += " -C link-self-contained=no"
        if arch == "i386":
            rustflags += " -C target-feature=-sse,-sse2"
        env["RUSTFLAGS"] = rustflags

    env["CARGO_TARGET_DIR"] = str(target_dir)
    env_cmd = [
        "cargo",
        "build",
        "--release",
        "--target",
        target,
        *std_args,
        "--manifest-path",
        str(ws / "Cargo.toml"),
    ]
    for c in cells:
        env_cmd += ["-p", c.crate]  # each cell package builds its single [[bin]]
    if dry:
        pin = env.get("RUSTUP_TOOLCHAIN", "<stable: rust-toolchain.toml>")
        log(
            f"DRY  RUSTUP_TOOLCHAIN={pin} CARGO_TARGET_DIR={target_dir} "
            f"{' '.join(env_cmd)}  (cwd={ws})"
        )
        return
    if trace_dir is not None:
        # Engine appends each in-cargo command to <trace_dir>/<pkg>.trace.
        env["VIBEOS_LINK_TRACE"] = str(trace_dir)
    if force_off:
        # Engine takes the cmake-OFF paths even though build-<arch>/ is present.
        env["VIBEOS_FORCE_CMAKE_OFF"] = "1"
    _trace(env_cmd, ws)
    log(f"exec cargo build — {', '.join(c.crate for c in cells)}")
    subprocess.run(env_cmd, cwd=ws, env=env, check=True)


def post_link(
    root: Path, cell: Cell, arch: str, out: Path, pefixup: Path, dry: bool = False
) -> None:
    """Apply the KIND post-link — the per-KIND POST_BUILD add_cargo_module emits."""
    is_i386 = arch == "i386"
    fix_fwd = [
        sys.executable,
        str(root / "sdk/tools/fix_lld_forwarding.py"),
        str(out),
    ]
    if cell.kind == "kernelexe":
        run([str(pefixup), "--kernel", str(out)], dry=dry)
        if is_i386:
            run(fix_fwd, dry=dry)
    elif cell.kind == "kerneldll":
        run([str(pefixup), "--kerneldll", str(out)], dry=dry)
        if is_i386:
            run(fix_fwd, dry=dry)
    elif cell.kind == "kernelmodedriver":
        run([str(pefixup), "--kernelmodedriver", str(out)], dry=dry)
        if is_i386:
            run(fix_fwd, dry=dry)
    elif cell.kind in ("win32dll", "win32exe"):
        # No pefixup (not a kernel module type); only the i386 forwarder fix.
        if is_i386:
            run(fix_fwd, dry=dry)
    elif cell.kind == "efi":
        strip = cross_strip(root)
        run([strip, "--remove-section=.rossym", str(out)], dry=dry)
        run([strip, "--strip-all", str(out)], dry=dry)
    elif cell.kind == "native":
        pass  # rosload's POST_BUILD was a no-op
    else:
        raise SystemExit(f"unknown KIND {cell.kind!r} for cell {cell.target}")


def stage_build(
    root: Path, cells: list[Cell], arch: str, out_dir: Path, pefixup: Path, force_off: bool
) -> dict[str, Path]:
    """Build every cell (one fan-out invocation) + copy out + post-link into
    out_dir. force_off=True passes VIBEOS_FORCE_CMAKE_OFF so the engine takes the
    cmake-OFF paths. Returns {cell.target: staged path}."""
    cargo_build_many(root, cells, arch, force_off=force_off)
    out_dir.mkdir(parents=True, exist_ok=True)
    staged: dict[str, Path] = {}
    for cell in cells:
        dest = out_dir / cell.output
        shutil.copy2(built_exe(root, cell, arch), dest)
        post_link(root, cell, arch, dest, pefixup)
        staged[cell.target] = dest
    return staged


def diff_offsets(a: Path, b: Path) -> tuple[int, int]:
    """(number of differing bytes, max differing file offset) between two files.
    A length mismatch counts every trailing byte as differing."""
    da, db = a.read_bytes(), b.read_bytes()
    n, last = 0, -1
    for i in range(min(len(da), len(db))):
        if da[i] != db[i]:
            n += 1
            last = i
    extra = abs(len(da) - len(db))
    if extra:
        n += extra
        last = max(len(da), len(db)) - 1
    return n, last


def verify_cmake_off(root: Path, cells: list[Cell], arch: str, out_dir: Path, pefixup: Path) -> int:
    """Byte-identity gate: build every cell twice from the SAME source — once the
    normal replay (cmake-present) path, once the forced cmake-OFF path — and
    compare. They must differ only in the PE header (the TimeDateStamp, < 0x400);
    any later-offset diff means the cmake-off path diverged from cmake. Comparing
    the two engine builds (not a possibly-stale build-<arch>/ artifact) avoids
    false drift from baseline staleness. exit 1 on any DRIFT."""
    log(f"verify: pass 1/2 — replay (cmake-present) build of {len(cells)} cells")
    rep = stage_build(root, cells, arch, out_dir / ".verify-replay", pefixup, force_off=False)
    log(f"verify: pass 2/2 — forced cmake-OFF build of {len(cells)} cells")
    off = stage_build(root, cells, arch, out_dir / ".verify-off", pefixup, force_off=True)
    HEADER = 0x400  # PE-header region: TimeDateStamp/checksum live here
    fails = []
    for cell in cells:
        n, last = diff_offsets(rep[cell.target], off[cell.target])
        ok = last < HEADER
        where = "identical" if n == 0 else f"{n} diff bytes, max off 0x{last:x}"
        log(f"  {'OK   ' if ok else 'DRIFT'} {cell.output:22} {where}")
        if not ok:
            fails.append(cell.target)
    if fails:
        log(f"verify: {len(fails)}/{len(cells)} DRIFT — cmake-off ≠ cmake: {', '.join(fails)}")
        return 1
    log(f"verify: all {len(cells)} cells cmake-off byte-identical to cmake (PE-timestamp only)")
    return 0


# ---- every cell, from the manifest (tools/cell_manifest.py) ---------------


def configure_gen(root: Path, arch: str, gen: Path, seed: Path | None = None) -> Path:
    """The configure step a build with no cmake still needs: `version.h` and `buildno.h`,
    which `sdk/include/reactos/version.cmake` writes at CONFIGURE time (the UTC date, `git
    describe`), into `gen/sdk/include/reactos/` for the engine (VIBEOS_GEN_DIR). Like a
    configure, it runs once: a later build keeps them until `--reconfigure`. `seed` copies
    a configured build's own instead (a comparison against it needs the same inputs).
    The host-tools project configures its own pair, naming the host compiler, under
    `gen/host-tools/bin` (mkisofs prints it in `-version`)."""
    import datetime
    import re

    out = gen / "sdk/include/reactos"
    host_out = gen / "host-tools/bin/sdk/include/reactos"
    pairs = ((out, "sdk/include/reactos"), (host_out, "host-tools/bin/sdk/include/reactos"))
    if all((d / h).exists() for d, _ in pairs for h in ("buildno.h", "version.h")):
        return gen
    if seed is not None:
        for d, rel in pairs:
            d.mkdir(parents=True, exist_ok=True)
            for h in ("version.h", "buildno.h"):
                shutil.copy2(seed / rel / h, d / h)
        return gen
    cm = (root / "sdk/include/reactos/version.cmake").read_text()
    var = dict(re.findall(r'^set\((\w+) "([^"]*)"\)', cm, re.M))
    winarch = "x86" if arch == "i386" else arch
    tag = var["KERNEL_VERSION_BUILD_TYPE"]
    kv = ".".join(
        var[k]
        for k in ("KERNEL_VERSION_MAJOR", "KERNEL_VERSION_MINOR", "KERNEL_VERSION_PATCH_LEVEL")
    )
    dll_major = int(var["KERNEL_VERSION_MAJOR"]) + 42
    dll = f"{dll_major}.{var['KERNEL_VERSION_MINOR']}.{var['KERNEL_VERSION_PATCH_LEVEL']}"

    def git(*a: str) -> str:
        r = subprocess.run(["git", "-C", str(root), *a], capture_output=True, text=True)
        return r.stdout.strip() if r.returncode == 0 else ""

    clang = root / ".build-tools/llvm-mingw/bin/clang"
    ver = re.search(
        r"clang version (\S+)",
        subprocess.run(
            [str(clang), "--version"], capture_output=True, text=True, check=True
        ).stdout,
    )
    # The host project's compiler is the host cc, named as cmake identifies it.
    def cc(*a: str) -> str:
        return subprocess.run(["cc", *a], capture_output=True, text=True, check=True).stdout

    host_id = "Clang" if "clang" in cc("--version") else "GNU"
    host_cc = cc("-dumpfullversion" if host_id == "GNU" else "-dumpversion").strip()
    values = {
        **var,
        "KERNEL_VERSION_BUILD": datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d"),
        "KERNEL_VERSION": f"{kv}-{winarch}" + (f"-{tag}" if tag else ""),
        "REACTOS_DLL_VERSION_MAJOR": str(dll_major),
        "DLL_VERSION_STR": dll + (f"-{tag}" if tag else ""),
        "COMMIT_HASH": git("rev-parse", "HEAD") or "unknown-hash",
        "REVISION": git("describe", "--abbrev=7", "--long", "--always") or "unknown-revision",
        "CMAKE_C_COMPILER_ID": "Clang",
        "CMAKE_C_COMPILER_VERSION": ver.group(1) if ver else "",
    }
    host = {**values, "CMAKE_C_COMPILER_ID": host_id, "CMAKE_C_COMPILER_VERSION": host_cc}
    for (d, _), vals in zip(pairs, (values, host)):
        d.mkdir(parents=True, exist_ok=True)
        for h in ("version.h", "buildno.h"):
            tmpl = (root / f"sdk/include/reactos/{h}.cmake").read_text()
            (d / h).write_text(re.sub(r"@(\w+)@", lambda m: vals[m.group(1)], tmpl))
    return gen


def artifact_store_env(root: Path, arch: str, report: Path):
    """The immutable body store, as `vibeos-cli build` gives it to cmake's cargo commands:
    a supervisor owns the lease, and every cell's body comes from the store (or is built
    into it once) instead of each cell compiling its own privately. A context manager
    yielding the env to add."""
    import contextlib

    sys.path.insert(0, str(root))
    from tests.qemu.cli import hostjobs
    from tools.artifact_executor import PIN
    from tools.artifact_runtime import record_outer_result, supervise

    store = Path(
        os.environ.get("VIBEOS_ARTIFACT_STORE_ROOT") or Path.home() / "vibeos-artifact-store"
    )

    @contextlib.contextmanager
    def ctx():
        hostjobs.ensure(root)
        toolchain = Path.home() / ".rustup" / "toolchains" / PIN
        real = Path(os.environ.get("VIBEOS_ORCHESTRATE_REPO") or root)
        with supervise(real, store, toolchain, report, clean=False, verify=False, arch=arch) as sup:
            result = {"rc": 1}
            yield (
                {
                    "VIBEOS_ARTIFACT_STORE_MODE": "ON",
                    "VIBEOS_ARTIFACT_STORE_ROOT": str(store),
                    "VIBEOS_ARTIFACT_VERIFY_NONCE": "",
                    "VIBEOS_ARTIFACT_SUPERVISOR": str(sup),
                    "VIBEOS_ARTIFACT_PYTHON": sys.executable,
                    "VIBEOS_ARTIFACT_TOOLCHAIN_ID": json.loads(sup.read_text())["toolchain_key"],
                },
                result,
            )
            record_outer_result(sup, result["rc"])

    return ctx()


def configure_sdk(root: Path, arch: str, gen: Path) -> None:
    """The generated SDK and the generated inputs cells read (tools/sdk_recipe.py), into
    `gen`, once per recipe: the stamp holds the recipe's hash, so a changed recipe
    regenerates and an unchanged one is a no-op."""
    sys.path.insert(0, str(root / "tools"))
    import sdk_recipe

    digest = hashlib.sha256(
        b"".join(sdk_recipe.recipe_path(arch, h, root).read_bytes() for h in (True, False))
    ).hexdigest()
    stamp = gen / ".sdk-recipe"
    if stamp.exists() and stamp.read_text().strip() == digest:
        return
    log(f"configure: generating the SDK into {gen}")
    sdk_recipe.run(arch, gen, root)
    stamp.write_text(digest + "\n")


def manifest_build(
    root: Path,
    arch: str,
    out_dir: Path,
    pefixup: Path,
    crates: list[str] | None,
    force_off: bool,
    gen: Path,
    store_env: dict[str, str] | None = None,
    jobs: int = 4,
) -> int:
    """Build every cell `sdk/lib/rust/cells/<arch>.jsonl` lists (or the named crates), one
    cargo per cell as add_cargo_module runs it, into `out_dir/<build-relative output>`,
    and apply each output's post-link."""
    sys.path.insert(0, str(root / "tools"))
    import cell_manifest

    import media_manifest

    # A full build stamps out_dir with the source it was made from, for
    # media_manifest.py --assemble --cells (and the smoke that boots what it masters).
    stamp = out_dir / media_manifest.CELLS_STAMP
    stamp.unlink(missing_ok=True)
    source = None if crates else media_manifest.source_hash(root)
    cells = cell_manifest.current(arch, root)
    if crates:
        unknown = set(crates) - {c["crate"] for c in cells}
        if unknown:
            raise SystemExit(f"not in the {arch} cell manifest: {', '.join(sorted(unknown))}")
        cells = [c for c in cells if c["crate"] in crates]
    ws = root / "sdk/lib/rust"

    def one(n: int, c: dict) -> str | None:
        target = c["target"]
        stem = Path(target).stem if target.endswith(".json") else target
        cmd = [
            "cargo",
            "build",
            "--release",
            "--target",
            str(root / target) if target.endswith(".json") else target,
        ]
        if c["build_std"]:
            cmd += [
                "-Z",
                "build-std=core,alloc,compiler_builtins",
                "-Z",
                "build-std-features=compiler-builtins-mem",
            ]
        cmd += ["-p", c["crate"]]
        cmd += ["--bins"] if c["bins"] == "all" else [x for b in c["bins"] for x in ("--bin", b)]
        cmd += ["--manifest-path", str(ws / "Cargo.toml")]
        env = {k: v for k, v in os.environ.items() if k not in ("RUSTFLAGS", "RUSTUP_TOOLCHAIN")}
        env.update(c["env"])
        tdir = ws / "target" / c["target_dir"]
        env["CARGO_TARGET_DIR"] = str(tdir)
        env["CARGO_BUILD_JOBS"] = "1"  # as add_cargo_module: parallelism is across cells
        env["VIBEOS_GEN_DIR"] = str(gen)
        env.update(store_env or {})
        if force_off:
            env["VIBEOS_FORCE_CMAKE_OFF"] = "1"
        log_path = out_dir / "logs" / f"{c['crate']}.log"
        log_path.parent.mkdir(parents=True, exist_ok=True)
        with open(log_path, "w") as f:
            r = subprocess.run(cmd, cwd=ws, env=env, stdout=f, stderr=subprocess.STDOUT)
        if r.returncode:
            log(f"[{n}/{len(cells)}] FAIL {c['crate']}  ({log_path})")
            return c["crate"]
        for o in c["outputs"]:
            dest = out_dir / o["out"]
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(tdir / stem / "release" / f"{o['bin']}.exe", dest)
            for step in o["post"]:
                if step.startswith("pefixup "):
                    run([str(pefixup), step.split()[1], str(dest)])
                elif step == "fix_lld_forwarding":
                    run([sys.executable, str(root / "sdk/tools/fix_lld_forwarding.py"), str(dest)])
                elif step.startswith("strip "):
                    run([cross_strip(root), *step.split()[1:], str(dest)])
                else:
                    raise SystemExit(f"{c['crate']}: unknown post-link step {step!r}")
        log(f"[{n}/{len(cells)}] ok   {c['crate']}")
        return None

    from concurrent.futures import ThreadPoolExecutor

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        failed = [x for x in pool.map(lambda nc: one(*nc), enumerate(cells, 1)) if x]
    if failed:
        log(f"{len(failed)}/{len(cells)} cells FAILED: {' '.join(failed)}")
        return 1
    log(f"all {len(cells)} cells built into {out_dir}")
    if source is not None:
        if media_manifest.source_hash(root) != source:
            log("the source tree changed during the build: not stamping it")
            return 1
        stamp.write_text(json.dumps({"arch": arch, "source": source}) + "\n")
    return 0


def topo(targets: list[str]) -> list[Cell]:
    """Dependency-ordered cell list (cargo-cell edges only)."""
    order: list[Cell] = []
    seen: set[str] = set()

    def visit(name: str, stack: tuple[str, ...]) -> None:
        if name in seen:
            return
        if name in stack:
            raise SystemExit(f"dependency cycle: {' -> '.join((*stack, name))}")
        cell = CELLS.get(name)
        if cell is None:
            raise SystemExit(f"unknown cell {name!r}; known: {', '.join(sorted(CELLS))}")
        for dep in cell.deps:
            visit(dep, (*stack, name))
        seen.add(name)
        order.append(cell)

    for t in targets:
        visit(t, ())
    return order


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", choices=sorted(ARCH_STEM), required=True)
    ap.add_argument("targets", nargs="*", help="cells to build (default: ntoskrnl)")
    ap.add_argument("--out-dir", help="output dir (default: cmake-off-out/<arch>)")
    ap.add_argument("--list", action="store_true", help="list known cells and exit")
    ap.add_argument(
        "--spine",
        action="store_true",
        help="build the full cmake-off kernel+spine set (kernel EXEs + the proven spine)",
    )
    ap.add_argument(
        "--drivers",
        action="store_true",
        help="build the complete cmake-off driver set (25/25, incl. win32k)",
    )
    ap.add_argument(
        "--loaders",
        action="store_true",
        help="build the boot loaders (uefildr efi ×3, rosload native i386+amd64)",
    )
    ap.add_argument(
        "--all",
        action="store_true",
        help="everything: kernel EXEs + spine + drivers + loaders",
    )
    ap.add_argument(
        "--manifest",
        action="store_true",
        help="build every cell in sdk/lib/rust/cells/<arch>.jsonl (or the named *-cargo "
        "crates) into <out-dir>/<build-relative output>, with no cmake configure or ninja",
    )
    ap.add_argument(
        "--cmake-off",
        action="store_true",
        help="with --manifest: take the engine's cmake-off paths (VIBEOS_FORCE_CMAKE_OFF)",
    )
    ap.add_argument(
        "--seed-from-build",
        action="store_true",
        help="with --manifest: configure version.h/buildno.h from build-<arch>'s own",
    )
    ap.add_argument(
        "--jobs",
        type=int,
        default=4,
        help="with --manifest: cells linked at once (cmake's vibeos_cargo_links pool is 4)",
    )
    ap.add_argument(
        "--no-artifact-store",
        action="store_true",
        help="with --manifest: build every cell's body privately, not through the store",
    )
    ap.add_argument(
        "--reconfigure",
        action="store_true",
        help="with --manifest: regenerate version.h/buildno.h (a configure runs once)",
    )
    ap.add_argument("--dry-run", action="store_true", help="print commands only")
    ap.add_argument(
        "--verify",
        action="store_true",
        help="byte-identity gate: build each cell replay vs forced-cmake-off and "
        "diff (must match bar the PE timestamp) — catches cmake-off regressions",
    )
    ap.add_argument(
        "--trace",
        nargs="?",
        const="",
        default=None,
        metavar="FILE",
        help="record every command run (orchestrator + in-cargo engine) to FILE "
        "(default <out-dir>/build-trace.txt) — the ninja-`-v`-style flat command list",
    )
    args = ap.parse_args(argv)

    # Every build in this tree runs at the canonical /tmp/vibeos mount, and the build
    # graph the replay paths read (build.ninja's widl commands, compile_commands.json)
    # names that path. Run outside it, and the replay of services' RPC stub failed
    # `cd /tmp/vibeos/base/system/services`. Re-enter through canon-run, as every
    # vibeos-cli build does, unless already inside it or only listing.
    canon = os.environ.get("VIBEOS_CANON", "/tmp/vibeos")
    if not args.list and not args.dry_run and str(repo_root()) != canon:
        # The canonical copy of this script, so the re-run's repo_root() IS the mount.
        script = f"{canon}/tools/{Path(__file__).name}"
        # The artifact store's supervisor must be told the REAL checkout: its worker checks
        # that the checkout is hidden inside the isolated compile, where the canonical
        # path is the (read-only) source mount and so always visible (vibeos-cli build
        # passes the worktree path for the same reason).
        os.environ["VIBEOS_ORCHESTRATE_REPO"] = str(repo_root())
        cmd = ["bash", str(repo_root() / "tools/canon-run.sh"), sys.executable, script]
        os.chdir(repo_root())
        os.execvp("bash", cmd + list(sys.argv[1:] if argv is None else argv))

    if args.list:
        for c in CELLS.values():
            deps = f"  deps={','.join(c.deps)}" if c.deps else ""
            print(f"{c.target:12} {c.crate:18} {c.output:16} {c.kind}{deps}")
        return 0

    root = repo_root()
    if args.manifest:
        out_dir = Path(args.out_dir) if args.out_dir else root / "cmake-off-out" / args.arch
        pefixup = build_pefixup(root, args.arch, out_dir)
        # With no cmake build here, the generated SDK goes where a build tree lives,
        # build-<arch>: a generated header's path is part of the image (wdm.h's inline
        # ASSERTs name it in __FILE__), so this is what makes the result byte-identical to
        # cmake's. A tree that has a cmake build keeps it untouched and uses <out>/gen.
        build = root / f"build-{args.arch}"
        gen = out_dir / "gen" if (build / "build.ninja").exists() else build
        if args.reconfigure and gen.exists():
            if gen != build:
                shutil.rmtree(gen)
            else:
                for f in (
                    "sdk/include/reactos/version.h",
                    "sdk/include/reactos/buildno.h",
                    ".sdk-recipe",
                ):
                    (gen / f).unlink(missing_ok=True)
        seed = root / f"build-{args.arch}" if args.seed_from_build else None
        configure_gen(root, args.arch, gen, seed)
        if args.cmake_off:
            configure_sdk(root, args.arch, gen)
        if args.no_artifact_store:
            return manifest_build(
                root,
                args.arch,
                out_dir,
                pefixup,
                args.targets or None,
                args.cmake_off,
                gen,
                None,
                args.jobs,
            )
        with artifact_store_env(root, args.arch, out_dir / "artifact-supervisor") as (env, result):
            result["rc"] = manifest_build(
                root,
                args.arch,
                out_dir,
                pefixup,
                args.targets or None,
                args.cmake_off,
                gen,
                env,
                args.jobs,
            )
        return result["rc"]

    targets: list[str] = list(args.targets)
    if args.spine or args.all:
        targets = ["ntoskrnl", "ntkrnlmp", *CMAKE_OFF_SPINE, *targets]
    if args.drivers or args.all:
        targets += list(CMAKE_OFF_DRIVERS)
    if args.loaders or args.all:
        targets += list(CMAKE_OFF_LOADERS)
    if not targets:
        targets = ["ntoskrnl"]
    out_dir = Path(args.out_dir) if args.out_dir else root / "cmake-off-out" / args.arch
    # Drop cells this arch doesn't build (e.g. arm64-only video/input on x86).
    cells = [c for c in topo(targets) if args.arch in c.arches]
    skipped = [c.target for c in topo(targets) if args.arch not in c.arches]
    if skipped:
        log(f"skipping (not on {args.arch}): {', '.join(skipped)}")
    log(f"arch={args.arch} cells={' -> '.join(c.target for c in cells)}")

    # `--trace`: clear a per-cell trace dir the engine appends to (VIBEOS_LINK_TRACE).
    trace_dir: Path | None = None
    if args.trace is not None and not args.dry_run:
        trace_dir = Path(out_dir) / ".trace"
        if trace_dir.exists():
            shutil.rmtree(trace_dir)
        trace_dir.mkdir(parents=True)

    pefixup = build_pefixup(root, args.arch, out_dir, dry=args.dry_run)

    if args.verify:
        return verify_cmake_off(root, cells, args.arch, out_dir, pefixup)

    # Phase 1: ONE fan-out cargo invocation builds every cell in parallel.
    cargo_build_many(root, cells, args.arch, dry=args.dry_run, trace_dir=trace_dir)

    # Phase 2: copy out + per-KIND post-link (cheap, sequential).
    for cell in cells:
        built = built_exe(root, cell, args.arch)
        dest = out_dir / cell.output
        if args.dry_run:
            log(f"DRY  copy {built} -> {dest}; post-link KIND {cell.kind}")
        else:
            out_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(built, dest)
        post_link(root, cell, args.arch, dest, pefixup, dry=args.dry_run)
        if not args.dry_run:
            sz = dest.stat().st_size if dest.exists() else 0
            log(f"OK   {cell.output}  ({sz} bytes, KIND {cell.kind})")

    if trace_dir is not None:
        trace_out = Path(args.trace) if args.trace else out_dir / "build-trace.txt"
        lines = [
            f"# cmake-off build trace — arch={args.arch}, cells: "
            f"{' '.join(c.target for c in cells)}",
            "# (no cmake configure, no ninja; every command this build ran)",
            "",
            "# === orchestrator-spawned commands ===",
            *_TRACE_CMDS,
            "",
            "# === in-cargo (reactos-link engine) commands, per cell ===",
        ]
        for cell in cells:
            f = trace_dir / f"{cell.crate}.trace"
            body = f.read_text().rstrip("\n").splitlines() if f.exists() else []
            lines.append(f"## {cell.target} ({cell.crate}) — {len(body)} commands")
            lines.extend(body)
        trace_out.write_text("\n".join(lines) + "\n")
        n_engine = sum(1 for line in lines if line and not line.startswith("#"))
        log(f"trace: {n_engine} commands -> {trace_out}")

    log("done — cmake-off build complete (no cmake configure, no ninja)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
