"""Target.islands is the generic form of every per-subsystem bindgen hook.

The engine used to carry five hard-coded branches — the shared kernel bindings plus one
each for hal / win32k / winsrv / crate-local — whose merge policies were identical. They
collapsed into one loop over `Target.islands`, which is also how a NON-ReactOS project
declares its FFI bindings without the engine growing a branch for it.
"""

from __future__ import annotations

from pathlib import Path

from rustconv.config import TARGETS, Island, Target


def _target(**kw) -> Target:
    base = dict(
        name="t",
        source_dir=Path("/src"),
        output_crate=Path("/crate"),
        bindgen_files={},
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=Path("/p.md"),
        src_subdir=None,
        cmake_deps_file=None,
    )
    base.update(kw)
    return Target(**base)


def test_base_bindings_are_not_authoritative():
    """Hand-written substrate must outrank the shared kernel island."""
    t = _target(bindgen_files={"amd64": Path("/ddk.rs")})
    assert [(i.module, i.authoritative) for i in t.islands] == [("ddk", False)]


def test_subsystem_islands_are_authoritative_and_ordered():
    """A module's OWN island outranks the shared one, and order is significance order."""
    t = _target(
        bindgen_files={"amd64": Path("/ddk.rs")},
        win32k_bindgen_files={"amd64": Path("/w32k.rs")},
    )
    assert [(i.module, i.authoritative) for i in t.islands] == [
        ("ddk", False),
        ("win32k", True),
    ]


def test_empty_hooks_contribute_nothing():
    """The 60-entry registry passes {} for hooks it does not use — no phantom islands."""
    t = _target(bindgen_files={"amd64": Path("/ddk.rs")}, winsrv_bindgen_files={})
    assert [i.module for i in t.islands] == ["ddk"]


def test_crate_module_names_the_island():
    """crate_bindgen_files lands under crate_module, not a hardcoded name."""
    t = _target(
        bindgen_files={},
        crate_bindgen_files={"amd64": Path("/types.rs")},
        crate_module="shellcom",
    )
    assert [i.module for i in t.islands] == ["ddk", "shellcom"]


def test_a_project_may_declare_islands_directly():
    """The qTox path: set islands explicitly and the legacy hooks are never consulted.

    An explicit declaration wins outright — no merging with the derived list, so a
    non-ReactOS project never inherits a 'ddk' island it has no concept of.
    """
    t = _target(
        bindgen_files={"amd64": Path("/ignored.rs")},
        win32k_bindgen_files={"amd64": Path("/also-ignored.rs")},
        islands=(Island("qt", {"amd64": Path("/qt.rs")}),),
    )
    assert [(i.module, i.authoritative) for i in t.islands] == [("qt", True)]


def test_live_registry_shapes():
    """The real targets derive the islands the old hard-coded branches produced."""
    assert [i.module for i in TARGETS["ntoskrnl"].islands] == ["ddk"]
    assert [i.module for i in TARGETS["hal"].islands] == ["ddk", "hal_ddk"]
    assert [i.module for i in TARGETS["win32k"].islands] == ["ddk", "win32k"]
    assert [i.module for i in TARGETS["winsrv"].islands] == ["ddk", "winsrv"]
    # a generic driver/DLL island lands under crate_module
    assert [i.module for i in TARGETS["ws2_32"].islands] == ["ddk", "types"]
    assert [i.module for i in TARGETS["shell32"].islands] == ["ddk", "shellcom"]


def test_engine_no_longer_names_subsystems():
    """catalog.py must not reach for a per-subsystem field ever again."""
    src = Path(__file__).resolve().parents[1] / "rustconv" / "catalog.py"
    text = src.read_text()
    for banned in (
        "hal_bindgen_files",
        "win32k_bindgen_files",
        "winsrv_bindgen_files",
        "crate_bindgen_files",
    ):
        assert banned not in text, f"catalog.py still reaches for {banned}"
