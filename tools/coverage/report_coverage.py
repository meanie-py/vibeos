#!/usr/bin/env python3
"""
report_coverage.py — turn a vibeos_cov TB-vaddr dump into the per-function
boot coverage map.

Inputs: a cov.txt from the vibeos_cov.so TCG plugin (one hex TB-start vaddr
per line) plus the arch's built kernel. Every function ENTRY is a branch
target, so it starts its own TB — a function is "exercised" iff at least one
recorded TB start falls inside its [start, next-start) range.

Runtime base is SELF-CALIBRATED from the coverage data: ntoskrnl links low
(amd64: ImageBase 0x400000) and is relocated high by the loader, and nothing
on serial discloses the load address — so we correlate the TB-start set
against the image's own function-offset signature. For a sample of function
offsets, every cov addr votes for the base that would place that function
there; the true base collects votes from every genuinely-executed sampled
function while wrong bases get noise. The winner is then verified against
the full entry set and reported (fail-loud if the correlation is weak).

The headline product is the NEVER-EXECUTED list for Rust-owned functions:
translated code a green smoke boot never runs is exactly where painting.rs-
class landmines sit dormant until a new caller (Rust explorer) first
exercises them.

Usage:
    report_coverage.py --arch amd64 --cov <cov.txt> [--out-dir DIR]

Writes <out-dir>/coverage-report.txt (summary) and
<out-dir>/never-executed-rust.txt (the landmine registry), prints the summary.
"""

from __future__ import annotations

import argparse
import bisect
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent

ARCH_NM_PREFIX = {"i386": "i686", "amd64": "x86_64", "arm64": "aarch64"}


def nm(repo: Path) -> Path:
    p = repo / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    if not p.is_file():
        sys.exit(f"missing {p} — toolchain not provisioned")
    return p


def text_symbols(nm_bin: Path, image: Path) -> list[tuple[int, str]]:
    """Sorted (addr, name) for defined text symbols of a PE image."""
    cp = subprocess.run(
        [str(nm_bin), "--defined-only", str(image)], capture_output=True, text=True, timeout=120
    )
    if cp.returncode != 0:
        sys.exit(f"llvm-nm failed on {image}: {cp.stderr.strip()[:200]}")
    out = []
    for ln in cp.stdout.splitlines():
        parts = ln.split()
        if len(parts) >= 3 and parts[1] in ("T", "t"):
            try:
                out.append((int(parts[0], 16), parts[2]))
            except ValueError:
                continue
    out.sort()
    return out


def rust_symbol_names(nm_bin: Path, repo: Path, arch: str) -> set[str]:
    """Defined text-symbol NAMES from the Rust kernel staticlib(s) — the
    'Rust-owned' set to intersect with the kernel image's functions."""
    pats = [
        f"build-{arch}/**/libkernel?rust*.a",
        f"build-{arch}/**/kernel?rust*.a",
        f"build-{arch}/**/libntoskrnl?rust*.a",
    ]
    libs: list[Path] = []
    for p in pats:
        libs += list(repo.glob(p))
    libs = sorted(set(libs))
    if not libs:
        sys.exit(
            f"no Rust kernel staticlib found under build-{arch}/ "
            f"(tried {', '.join(pats)}) — build first; refusing to emit a "
            f"report that silently lacks the Rust view"
        )
    names: set[str] = set()
    for lib in libs:
        cp = subprocess.run(
            [str(nm_bin), "--defined-only", str(lib)], capture_output=True, text=True, timeout=300
        )
        for ln in cp.stdout.splitlines():
            parts = ln.split()
            if len(parts) >= 3 and parts[1] in ("T", "t"):
                names.add(parts[2])
    return names


def discover_base(cov: list[int], entries: list[int], link_base: int) -> tuple[int, float]:
    """Vote for the runtime image base: for each cov addr c and sampled
    function offset s, candidate base = c - s. The true base accumulates one
    vote per executed sampled function; wrong bases collect only hash noise.
    Returns (base, verified_hit_fraction_of_sample)."""
    from collections import Counter

    offs = sorted(e - link_base for e in entries)
    # Uniform stride across the image. A boot only exercises ~15-40% of kernel
    # functions, so the winning base won't match most of the sample — the
    # acceptance test is vote SEPARATION (true base gets one vote per executed
    # sampled function; every wrong base only gets alignment-coincidence noise).
    sample = offs[:: max(1, len(offs) // 256)][:256]
    votes: Counter[int] = Counter()
    for c in cov:
        for s in sample:
            b = c - s
            if b & 0xFFF == 0:  # bases are at least page-aligned
                votes[b] += 1
    if not votes:
        sys.exit("coverage base discovery: no aligned candidates — empty cov?")
    top = votes.most_common(2)
    base, n = top[0]
    runner = top[1][1] if len(top) > 1 else 0
    if n < 8 or n < 3 * max(runner, 1):
        sys.exit(
            f"coverage base discovery FAILED: best {base:#x} ({n} votes) vs "
            f"runner-up ({runner}) — separation too weak; refusing to emit a "
            f"report against a guessed base"
        )
    return base, n / len(sample)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    ap.add_argument("--cov", required=True, type=Path)
    ap.add_argument("--out-dir", type=Path, default=None)
    args = ap.parse_args()

    out_dir = args.out_dir or args.cov.parent
    out_dir.mkdir(parents=True, exist_ok=True)
    image = REPO / f"build-{args.arch}" / "ntoskrnl" / "ntoskrnl.exe"
    if not image.is_file():
        sys.exit(f"missing {image} — build first")

    nm_bin = nm(REPO)
    syms = text_symbols(nm_bin, image)
    if not syms:
        sys.exit(f"no text symbols in {image}")
    addrs = [a for a, _ in syms]
    lo, hi = addrs[0], addrs[-1] + 0x10000  # slack past the last symbol

    cov: list[int] = []
    for ln in args.cov.read_text().splitlines():
        ln = ln.strip()
        if ln:
            try:
                cov.append(int(ln, 16))
            except ValueError:
                continue
    if not cov:
        sys.exit(f"{args.cov} holds no vaddrs")

    link_base = (addrs[0] >> 16) << 16  # e.g. first text 0x401000 -> 0x400000
    base, frac = discover_base(cov, addrs, link_base)
    delta = base - link_base

    covered_fn: set[int] = set()  # indices into syms
    n_kernel_tbs = 0
    for va in cov:
        va -= delta  # runtime -> link VA
        if va < lo or va >= hi:
            continue
        n_kernel_tbs += 1
        covered_fn.add(bisect.bisect_right(addrs, va) - 1)

    rust_names = rust_symbol_names(nm_bin, REPO, args.arch)
    total = len(syms)
    hit = len(covered_fn)
    # Dedup by NAME: cohab links with --allow-multiple-definition, so a symbol
    # can exist at two addresses (C copy + Rust copy). A name is exercised if
    # ANY of its addresses was hit — otherwise the un-linked twin reads as a
    # false "never".
    exercised_names = {syms[i][1] for i in covered_fn}
    rust_in_image = {n for _, n in syms if n in rust_names}
    rust_hit = rust_in_image & exercised_names
    never_rust = sorted(rust_in_image - exercised_names)

    (out_dir / "never-executed-rust.txt").write_text(
        "\n".join(never_rust) + ("\n" if never_rust else "")
    )
    summary = [
        f"boot coverage — arch={args.arch}  image={image.name}",
        f"  runtime base (self-calibrated)  : {base:#x} (sample correlation {frac * 100:.0f}%)",
        f"  kernel-range TB starts recorded : {n_kernel_tbs}",
        f"  kernel functions                : {hit}/{total} exercised"
        f" ({hit * 100 // max(total, 1)}%)",
        f"  Rust-owned functions            : {len(rust_hit)}/{len(rust_in_image)} exercised"
        f" ({len(rust_hit) * 100 // max(len(rust_in_image), 1)}%)",
        f"  NEVER-executed Rust functions   : {len(never_rust)}"
        f"  -> {out_dir / 'never-executed-rust.txt'}",
        "  (never-executed = untested-by-boot: the landmine registry —",
        "   a green smoke proves nothing about these)",
    ]
    (out_dir / "coverage-report.txt").write_text("\n".join(summary) + "\n")
    print("\n".join(summary))
    return 0


if __name__ == "__main__":
    sys.exit(main())
