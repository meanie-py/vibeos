"""tools/scan_init_then_move.py: a self-linked kernel object must never change address."""

from __future__ import annotations

from pathlib import Path

from scan_init_then_move import scan


def _findings(tmp_path: Path, code: str) -> list[str]:
    f = tmp_path / "x.rs"
    f.write_text(code)
    return [what for _, _, what in scan([f])]


def test_maybeuninit_then_assume_init(tmp_path: Path) -> None:
    code = """fn f() {
    let mut E = MaybeUninit::uninit();
    KeInitializeEvent(E.as_mut_ptr(), 0, 0);
    let E = E.assume_init();
}
"""
    assert _findings(tmp_path, code) == ["KeInitializeEvent(E.as_mut_ptr()) then E.assume_init()"]


def test_constructor_returning_the_initialized_struct(tmp_path: Path) -> None:
    """The first NtCancelIoFileEx: the event's wait list named the dead local."""
    code = """unsafe fn new_request() -> Req {
    let mut req = Req { done: zeroed() };
    KeInitializeEvent(&mut req.done, 0, 0);
    req
}
"""
    assert _findings(tmp_path, code) == ["KeInitializeEvent(&mut req..) then req returned by value"]


def test_explicit_return_is_the_same_move(tmp_path: Path) -> None:
    code = """fn g() -> L {
    let mut head = L::default();
    InitializeListHead(&mut head);
    return head;
}
"""
    assert len(_findings(tmp_path, code)) == 1


def test_initialized_in_place_and_used_by_reference_is_fine(tmp_path: Path) -> None:
    code = """fn h() -> NTSTATUS {
    let mut req = new_request();
    arm(&mut req);
    KeInitializeEvent(&mut req.done, 0, 0);
    wait(&req);
    STATUS_SUCCESS
}
"""
    assert _findings(tmp_path, code) == []
