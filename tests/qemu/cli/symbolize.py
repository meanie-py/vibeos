"""`vibeos-cli sym` — resolve an address to module!symbol+offset.

Why this exists: `qemu/build-index.sh` indexes exactly ntoskrnl.exe and
ntkrnlmp.exe, so until now every USER-MODE address had to be resolved by hand
with objdump plus a throwaway nearest-symbol script. That is the whole WoW64
lane, the winetest corpus and every stock application. Four addresses in one
session was the detection event.

The rule this module keeps: never name a symbol we cannot prove owns the
address. It declines on a shared image base, and declines on an offset large
enough to prove the nearest symbol is not the right one — the same discipline
`triage._resolve_addr` already applies to the kernel index.
"""

from __future__ import annotations

import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path

from tools import test_infra_compat as native

# Past this, "nearest preceding symbol" stops being evidence. A PE's symbol
# table omits plenty (static helpers, thunks, data-only regions), so some symbol
# always sorts below any address and will happily be reported as a match.
MAX_SANE_OFFSET = 0x20000


@dataclass(frozen=True)
class Module:
    base: int
    end: int
    name: str
    path: str
    unique: bool

    def contains(self, addr: int) -> bool:
        return self.base <= addr < self.end


def modules_path(repo: Path, arch: str) -> Path:
    return repo / "qemu" / f"modules-{arch}.tsv"


def symbols_path(repo: Path, arch: str) -> Path:
    return repo / "qemu" / f"symbols-{arch}-user.tsv"


def load_modules(repo: Path, arch: str) -> list[Module]:
    return [
        Module(**row)
        for row in native.call("verdict_symbol_load_modules", {"arch": arch}, root=repo)
    ]


def index_is_stale(repo: Path, arch: str) -> bool:
    return native.call("verdict_symbol_index_stale", {"arch": arch}, root=repo)


def build_index(repo: Path, arch: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["bash", str(repo / "qemu" / "build-user-index.sh"), arch],
        cwd=repo,
        capture_output=True,
        text=True,
    )


def lookup_symbol(repo: Path, arch: str, module: str, addr: int) -> tuple[str, int, str] | None:
    row = native.call(
        "verdict_symbol_lookup", {"arch": arch, "module": module, "addr": addr}, root=repo
    )
    return tuple(row) if row is not None else None


def lookup_symbol_live(
    repo: Path, arch: str, mod: Module, addr: int
) -> tuple[str, int, str] | None:
    row = native.call(
        "verdict_symbol_lookup_live", {"arch": arch, "module": asdict(mod), "addr": addr}, root=repo
    )
    return tuple(row) if row is not None else None


def _all_identical(repo: Path, mods: list[Module]) -> bool:
    return native.call(
        "verdict_symbol_all_identical", {"modules": [asdict(m) for m in mods]}, root=repo
    )


def resolve(
    repo: Path, arch: str, addr: int, want_module: str | None = None, offset: bool = False
) -> str:
    return native.call(
        "verdict_symbol_resolve",
        {"arch": arch, "addr": addr, "want_module": want_module, "offset": offset},
        root=repo,
    )


def run(args, repo: Path) -> int:
    arch = args.arch
    if index_is_stale(repo, arch) or args.rebuild:
        print(f"[sym] building the {arch} user-mode index (media is newer)...")
        cp = build_index(repo, arch)
        if cp.returncode != 0:
            print(cp.stderr.strip()[:2000])
            print(f"[sym] index build FAILED for {arch}")
            return 1
    rc = 0
    for raw in args.addr:
        module = args.module
        offset = "+" in raw
        if offset:  # `ndis.sys+0x12D10`, as a bugcheck prints a relocated module's PC
            module, raw = raw.rsplit("+", 1)
        try:
            addr = int(raw, 16)  # int(x, 16) already accepts a 0x prefix
        except ValueError:
            print(f"{raw} -> (not a hex address)")
            rc = 1
            continue
        print(resolve(repo, arch, addr, module, offset))
    return rc
