#!/usr/bin/env python3
"""WS5 — cmake/ninja-free orchestrator for the cargo-driven PE cells.

This is the thin "driver" the cargo-drive-cmake-off roadmap (WS5, Phase 3) calls
for: it replaces the `add_cargo_module()` ninja custom-commands in
`sdk/cmake/gcc.cmake` with a plain dependency-ordered walk that

  1. self-hosts the host tools a cell's post-link needs (`native-pefixup`),
  2. runs `cargo +nightly build-std` per cell (the exact invocation cmake emits),
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
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# arch → the cargo custom-target json stem (== cargo's target/<stem>/ output dir)
ARCH_STEM = {"i386": "i686-kernel", "amd64": "x86_64-kernel", "arm64": "aarch64-kernel"}
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


# Verified cmake-off cells. KINDs mirror the add_cargo_module() calls; deps are
# the *cargo-cell* build-order edges (NOT spec/import-lib deps — those each cell
# self-generates from .spec via the self-hosted spec2def). The kernel EXEs have
# no cargo-cell deps: they synth their hal/kdcom/bootvid import libs in-cargo.
CELLS: dict[str, Cell] = {
    c.target: c
    for c in (
        Cell("ntoskrnl", "ntoskrnl-cargo", "ntoskrnl.exe", "kernelexe"),
        Cell("ntkrnlmp", "ntkrnlmp-cargo", "ntkrnlmp.exe", "kernelexe"),
    )
}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def log(msg: str) -> None:
    print(f"[ws5] {msg}", flush=True)


def run(cmd: list[str], cwd: Path | None = None, dry: bool = False) -> None:
    shown = " ".join(str(c) for c in cmd)
    if dry:
        log(f"DRY  {shown}")
        return
    log(f"exec {shown}")
    subprocess.run(cmd, cwd=cwd, check=True)


def host_cc() -> str:
    """The host C compiler for building host tools (matches cmake's host build)."""
    return os.environ.get("CC", "cc")


def host_strip() -> str:
    """A PE-capable strip for the efi KIND (llvm-strip handles cross PE)."""
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
    if exe.exists() and not dry:
        return exe
    out_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        host_cc(),
        "-O2",
        "-DTARGET_amd64",  # host arch (x86_64) — compat.h CONTEXT/RUNTIME_FUNCTION
        "-I",
        str(root / "sdk/include/host"),
    ]
    if ARCH_PE64[arch]:
        cmd.append("-D_TARGET_PE64")
    cmd += ["-o", str(exe), str(root / "sdk/tools/pefixup.c")]
    run(cmd, dry=dry)
    return exe


def cargo_build(root: Path, cell: Cell, arch: str, dry: bool = False) -> Path:
    """Run the cargo +nightly build-std drive for one cell; return its .exe."""
    ws = root / "sdk/lib/rust"
    stem = ARCH_STEM[arch]
    bin_name = cell.crate.removesuffix("-cargo")
    env_cmd = [
        "cargo",
        "build",
        "--release",
        "--target",
        str(ws / cell.crate / f"{stem}.json"),
        "-Z",
        "build-std=core,alloc,compiler_builtins",
        "-Z",
        "build-std-features=compiler-builtins-mem",
        "-p",
        cell.crate,
        "--bin",
        bin_name,
        "--manifest-path",
        str(ws / "Cargo.toml"),
    ]
    if dry:
        log(f"DRY  RUSTUP_TOOLCHAIN=nightly {' '.join(env_cmd)}  (cwd={ws})")
    else:
        env = dict(os.environ, RUSTUP_TOOLCHAIN="nightly")
        log(f"exec RUSTUP_TOOLCHAIN=nightly {' '.join(env_cmd)}")
        subprocess.run(env_cmd, cwd=ws, env=env, check=True)
    return ws / "target" / stem / "release" / f"{bin_name}.exe"


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
        strip = host_strip()
        run([strip, "--remove-section=.rossym", str(out)], dry=dry)
        run([strip, "--strip-all", str(out)], dry=dry)
    elif cell.kind == "native":
        pass  # rosload's POST_BUILD was a no-op
    else:
        raise SystemExit(f"unknown KIND {cell.kind!r} for cell {cell.target}")


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
    ap.add_argument("--dry-run", action="store_true", help="print commands only")
    args = ap.parse_args(argv)

    if args.list:
        for c in CELLS.values():
            deps = f"  deps={','.join(c.deps)}" if c.deps else ""
            print(f"{c.target:12} {c.crate:18} {c.output:16} {c.kind}{deps}")
        return 0

    root = repo_root()
    targets = args.targets or ["ntoskrnl"]
    out_dir = Path(args.out_dir) if args.out_dir else root / "cmake-off-out" / args.arch
    cells = topo(targets)
    log(f"arch={args.arch} cells={' -> '.join(c.target for c in cells)}")

    pefixup = build_pefixup(root, args.arch, out_dir, dry=args.dry_run)

    for cell in cells:
        log(f"=== {cell.target} ({cell.output}, KIND {cell.kind}) ===")
        built = cargo_build(root, cell, args.arch, dry=args.dry_run)
        dest = out_dir / cell.output
        if args.dry_run:
            log(f"DRY  copy {built} -> {dest}")
        else:
            out_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(built, dest)
            log(f"copied -> {dest}")
        post_link(root, cell, args.arch, dest, pefixup, dry=args.dry_run)
        if not args.dry_run:
            sz = dest.stat().st_size if dest.exists() else 0
            log(f"OK   {cell.output}  ({sz} bytes)")

    log("done — cmake-off build complete (no cmake configure, no ninja)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
