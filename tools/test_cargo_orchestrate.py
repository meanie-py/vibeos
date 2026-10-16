"""cargo_orchestrate must not build one i686-kernel.json variant against another's core."""

import hashlib
import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location(
    "cargo_orchestrate", REPO / "tools/cargo_orchestrate.py"
)
assert _spec and _spec.loader
co = importlib.util.module_from_spec(_spec)
sys.modules["cargo_orchestrate"] = co
_spec.loader.exec_module(co)

I386 = [c for c in co.CELLS.values() if "i386" in c.arches and co.build_std(c, "i386")]


def _digest(cell) -> str:
    return hashlib.sha256(co.target_json(REPO, cell, "i386").read_bytes()).hexdigest()


def test_each_i386_target_dir_holds_exactly_one_json_variant():
    """Two variants in one CARGO_TARGET_DIR would share target/i686-kernel/, and cargo
    does not fingerprint a target json's contents."""
    by_dir: dict[Path, set[str]] = {}
    for cell in I386:
        by_dir.setdefault(co.cargo_target_dir(REPO, cell, "i386"), set()).add(_digest(cell))
    assert all(len(v) == 1 for v in by_dir.values()), by_dir
    assert len(by_dir) == len({_digest(c) for c in I386})


def test_the_kernel_and_user_mode_cells_really_differ():
    """The split this guards is deliberate (345bab65b7a): soft-float in the kernel,
    hard-float where C calls expect f32 in ST0 (win32k). The user-mode cells left the
    json for the builtin triple on 2026-10-14. If the json variants ever collapse to
    one, this test is what should be revisited -- not the grouping."""
    ntoskrnl, ntdll, win32k = (co.CELLS[n] for n in ("ntoskrnl", "ntdll", "win32k"))
    assert "+soft-float" in co.target_json(REPO, ntoskrnl, "i386").read_text()
    assert "+soft-float" not in co.target_json(REPO, win32k, "i386").read_text()
    assert co.cargo_target_dir(REPO, win32k, "i386") != co.cargo_target_dir(REPO, ntoskrnl, "i386")
    assert not co.build_std(ntdll, "i386")
    assert co.cell_stem(ntdll, "i386") == "i686-pc-windows-gnu"


def test_the_builtin_triple_cells_keep_the_shared_target_dir():
    for arch in ("i386", "amd64", "arm64"):
        dirs = {
            co.cargo_target_dir(REPO, c, arch)
            for c in co.CELLS.values()
            if arch in c.arches and not co.build_std(c, arch)
        }
        assert dirs == {REPO / "sdk/lib/rust/target"}
