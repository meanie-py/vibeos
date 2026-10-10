"""Pins apply_dead_assignments: each case is an edit that must, or must never, be made."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import apply_dead_assignments as a


def run(tmp_path: Path, src: str, marker: str, shape: str, var: str) -> tuple[str, list[str]]:
    f = tmp_path / "x.rs"
    f.write_text(src)
    line = next(i for i, ln in enumerate(src.splitlines(), 1) if marker in ln)
    row = {"path": "x.rs", "line": line, "var": var, "shape": shape}
    edits, skips = a.plan_file(f, [row])
    b = f.read_bytes()
    for s, e, rep, _ in sorted(edits, reverse=True):
        b = b[:s] + rep + b[e:]
    return b.decode(), skips


def test_dead_init_drops_only_the_initializer(tmp_path):
    src = "fn f() -> i32 {\n    let mut s: i32 = STATUS_SUCCESS;\n    s = g();\n    s\n}\n"
    out, _ = run(tmp_path, src, "let mut s", "DEAD_INIT", "s")
    assert "    let mut s: i32;\n" in out


def test_multiline_let_matches_the_value_line(tmp_path):
    """rustc points at the VALUE of `let mut p: *mut T =\\n    core::ptr::null_mut();`."""
    src = (
        "fn f() {\n    let mut p: *mut T =\n        core::ptr::null_mut::<T>();\n    p = g();\n}\n"
    )
    out, _ = run(tmp_path, src, "null_mut::<T>", "DEAD_INIT", "p")
    assert "let mut p: *mut T;\n" in out


def test_initializer_with_a_call_is_never_dropped(tmp_path):
    src = "fn f() {\n    let mut s: i32 = g();\n    s = h();\n}\n"
    out, skips = run(tmp_path, src, "let mut s", "DEAD_INIT", "s")
    assert out == src and "not pure" in skips[0]


def test_inside_a_stdmethod_macro(tmp_path):
    """ole32's COM methods live in `stdmethod! { fn .. }`: one opaque token tree."""
    src = (
        "stdmethod! {\nfn f() -> i32 {\n    let mut hr: i32 = S_OK;\n    hr = g();\n    hr\n}\n}\n"
    )
    out, _ = run(tmp_path, src, "let mut hr", "DEAD_INIT", "hr")
    assert "let mut hr: i32;\n" in out


def test_macro_rules_body_is_left_alone(tmp_path):
    src = (
        "macro_rules! m {\n    ($x:expr) => {{\n        let mut s: i32 = 0;\n"
        "        s = $x;\n    }};\n}\n"
    )
    out, skips = run(tmp_path, src, "let mut s", "DEAD_INIT", "s")
    assert out == src and skips


def test_lost_call_keeps_the_call_and_drops_its_cast(tmp_path):
    src = (
        "fn f() {\n    let mut w: i32;\n    w = Po(x, &mut y as *mut u32) as i32;\n"
        "    w = 1;\n    use_(w);\n}\n"
    )
    out, _ = run(tmp_path, src, "w = Po", "LOST_CALL", "w")
    assert "    Po(x, &mut y as *mut u32);\n" in out


def test_store_whose_address_is_taken_is_skipped(tmp_path):
    """cdfs's SEH cleanup flags: a finally block reads `*p_UnlockVcb`; rustc cannot see it."""
    src = (
        "fn f() {\n    let mut UnlockVcb: u8 = 0;\n    let p = &mut UnlockVcb as *mut u8;\n"
        "    UnlockVcb = 1;\n    fin(p);\n}\n"
    )
    out, skips = run(tmp_path, src, "UnlockVcb = 1", "DEAD_STORE", "UnlockVcb")
    assert out == src and "pointer" in skips[0]


def test_addr_of_in_any_spelling_counts():
    assert a.address_taken("let p = core::ptr::addr_of_mut!(x);", "x")
    assert a.address_taken("g(&raw mut x)", "x")
    assert a.address_taken("g(&x)", "x")
    assert not a.address_taken("g(&xy)", "x")


def test_pure_store_line_is_removed_whole(tmp_path):
    src = (
        "fn f() {\n    let mut s: i32;\n    s = g();\n    s = STATUS_SUCCESS;\n"
        "    s = h();\n    s\n}\n"
    )
    out, _ = run(tmp_path, src, "s = STATUS_SUCCESS", "DEAD_STORE", "s")
    assert "STATUS_SUCCESS" not in out and "    s = g();\n    s = h();\n" in out


def test_field_store(tmp_path):
    src = "fn f() {\n    let mut b: B = B { p: 0 };\n    b.p = core::ptr::null_mut();\n}\n"
    out, _ = run(tmp_path, src, "b.p =", "DEAD_STORE", "b")
    assert "b.p" not in out


def test_borrow_after_the_write_is_not_a_hidden_read(tmp_path):
    """scsiport's `Offset += CopyField(..)`: every `addr_of_mut!(Offset)` follows a fresh
    `Offset = 0`, so no pointer exists at the dead store; rustc counts a later borrow."""
    src = (
        "fn f() {\n    let mut o: u32 = 0;\n    o = 1;\n"
        "    o = 0;\n    g(core::ptr::addr_of_mut!(o));\n}\n"
    )
    out, skips = run(tmp_path, src, "o = 1", "DEAD_STORE", "o")
    assert "o = 1" not in out, skips


def test_borrow_in_an_enclosing_loop_counts(tmp_path):
    src = "fn f() {\n    let mut a: u8;\n    loop {\n        a = 0;\n        g(&mut a);\n    }\n}\n"
    out, skips = run(tmp_path, src, "a = 0", "DEAD_STORE", "a")
    assert out == src and "pointer" in skips[0]


def test_raw_borrowed_init_is_kept(tmp_path):
    """samsrv `let mut Length: ULONG = 0;` then `Query(.., &raw mut Length)`: an in/out size.
    `addr_of_mut!` of an uninitialized local compiles, so rustc would not catch the removal."""
    src = "fn f() {\n    let mut Length: u32 = 0;\n    q(&raw mut Length);\n    Length = 4;\n}\n"
    out, skips = run(tmp_path, src, "let mut Length", "DEAD_INIT", "Length")
    assert out == src and "raw-borrowed" in skips[0]


def test_init_overwritten_before_any_raw_borrow_is_dropped(tmp_path):
    """acpi `old_status`: written in straight-line code before its address is ever taken,
    so no borrow can see the initial value."""
    src = "fn f() {\n    let mut s: u32 = 0;\n    s = g();\n    q(&raw mut s);\n}\n"
    out, skips = run(tmp_path, src, "let mut s", "DEAD_INIT", "s")
    assert "let mut s: u32;" in out and not skips


def test_a_write_in_a_branch_does_not_dominate_the_borrow(tmp_path):
    src = (
        "fn f(c: bool) {\n    let mut s: u32 = 0;\n"
        "    if c {\n        s = g();\n    }\n    q(&raw mut s);\n}\n"
    )
    out, skips = run(tmp_path, src, "let mut s", "DEAD_INIT", "s")
    assert out == src and "raw-borrowed" in skips[0]
