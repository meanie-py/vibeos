"""port cppapp: a unit the regeneration no longer writes is parked, not left compiling."""

from __future__ import annotations

from rustconv import port


def test_a_dropped_units_module_is_moved_out_and_unwired(tmp_path, monkeypatch):
    repo, staging = tmp_path / "repo", tmp_path / "staging"
    src = repo / "sdk" / "lib" / "rust" / "x-rust" / "src"
    (src / "x").mkdir(parents=True)
    (src / "x" / "arr_h_h.rs").write_text("pub fn stale() {}\n")
    (src / "x" / "keep_h_h.rs").write_text("pub fn kept() {}\n")
    (src / "lib.rs").write_text("pub mod x {\n    pub mod arr_h_h;\n    pub mod keep_h_h;\n}\n")
    monkeypatch.setattr(port, "REPO", repo)
    monkeypatch.setattr(port, "STATE_ROOT", staging)
    state = {"stem": "x", "crate": "x-rust"}

    port._park_dropped_unit(state, str(staging / "x" / "cppapp" / "arr_h.hpp"))

    assert not (src / "x" / "arr_h_h.rs").exists()
    (parked,) = (staging / "x").glob("orphans-*/arr_h_h.rs")
    assert parked.read_text() == "pub fn stale() {}\n"
    assert (src / "lib.rs").read_text() == "pub mod x {\n    pub mod keep_h_h;\n}\n"


def test_own_reexports_are_the_names_the_prelude_imports_from_the_crate(tmp_path):
    # cicero's island-enum alias `POINTING_TIMER_ID` beside the cpp_classes re-export: E0255.
    p = tmp_path / "prelude.rs"
    p.write_text(
        "pub use crate::cicero::cpp_classes::*;\n"
        "pub use crate::cicero::cpp_classes::{CUIFWindow, POINTING_TIMER_ID};\n"
        "pub use crate::cicero::cicbase_h_h::cicMemAlloc;\n"
        "pub use win32macros_rust::{LOWORD};\n"
        "pub const USER_TIMER_ID: u32 = 1; // island enum\n"
    )
    assert port._own_reexports(p) == {"CUIFWindow", "POINTING_TIMER_ID", "cicMemAlloc"}


def test_read_only_stages_leave_the_crate_alone():
    from argparse import Namespace

    assert port._writes_nothing(Namespace(port_stage="check"))
    assert port._writes_nothing(Namespace(port_stage="bank-import", dry_run=False))
    assert port._writes_nothing(Namespace(port_stage="translate", dry_run=True))
    assert port._writes_nothing(Namespace(port_stage="hand", apply=False))
    assert not port._writes_nothing(Namespace(port_stage="hand", apply=True))
    assert not port._writes_nothing(Namespace(port_stage="replay"))
