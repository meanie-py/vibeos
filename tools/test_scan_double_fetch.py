"""scan_double_fetch: the same user address read twice, and the rules that keep it
quiet on the shapes that only LOOK like a re-read.

Each SAFE case is a pattern the first run flagged wrongly: a streaming copy that
advances the source pointer, a two-stage header-then-body read whose inner `as`
cast confused the source normaliser, reads in sibling match/if-else arms, and a
read whose branch exits (break/return) before the later read is reachable.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_double_fetch as scan

H = (
    'pub unsafe extern "system" fn NtThing(P: *mut _UNICODE_STRING, Mode: KPROCESSOR_MODE)'
    " -> NTSTATUS {\n"
)


def rows(tmp_path: Path, files: dict[str, str]) -> list[str]:
    picked = []
    for rel, text in files.items():
        p = tmp_path / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        picked.append((p, rel))
    return sorted(scan.scan_tree(picked))


def test_same_pointer_captured_twice_is_flagged(tmp_path):
    # The cast on the first read must not hide that both read the same address.
    src = H + (
        "    let a = read_back(P as *const _UNICODE_STRING)?;\n    check(a);\n"
        "    let b = read_back(P)?;\n    copy(dst, b);\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == ["x-rust/src/a.rs: NtThing P"]


def test_single_capture_used_twice_is_clean(tmp_path):
    src = H + (
        "    let a = read_back(P)?;\n    check(a.Length);\n"
        "    copy(dst, a.Buffer, a.Length);\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_reads_in_sibling_match_and_if_arms_are_clean(tmp_path):
    marm = H + (
        "    match cls {\n        1 => { let a = read_back(P)?; use(a); }\n"
        "        2 => { let b = read_back(P)?; use(b); }\n        _ => {}\n    }\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": marm}) == []
    ifel = H + "    if c { let a = read_back(P)?; } else { let b = read_back(P)?; }\n    0\n}\n"
    assert rows(tmp_path, {"x-rust/src/a.rs": ifel}) == []


def test_a_branch_that_exits_before_the_second_read_is_clean(tmp_path):
    # `if c { read; return }` then a later read: the two never both run.
    early = H + (
        "    if c {\n        let a = read_back(P)?;\n        return handle(a);\n    }\n"
        "    let b = read_back(P)?;\n    use(b);\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": early}) == []


def test_a_read_after_a_conditional_block_without_exit_is_flagged(tmp_path):
    seq = H + (
        "    if c { let a = read_back(P)?; use(a); }\n"
        "    let b = read_back(P)?;\n    use(b);\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": seq}) == ["x-rust/src/a.rs: NtThing P"]


def test_streaming_copy_that_advances_the_source_is_clean(tmp_path):
    src = H + (
        "    while y < n {\n        let _ = read_back(P)?;\n        P = P.add(stride);\n    }\n"
        "    let _ = read_back(P)?;\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_a_capture_in_the_kernel_mode_arm_is_not_a_cross_ring_race(tmp_path):
    src = H + (
        "    let a = read_back(P)?;\n"
        "    if Mode != 0 { check(a); } else { let b = read_back(P)?; use(b); }\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_two_different_sources_are_clean(tmp_path):
    src = H + "    let a = read_back(P)?;\n    let b = read_back(Q)?;\n    0\n}\n"
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_different_fields_of_one_pointer_are_different_sources(tmp_path):
    # The inner `as T` cast must not collapse `(*(p as *const S)).a` and `.b`.
    src = H + (
        "    let n = capture_from_user(d1, core::ptr::addr_of!((*(P as *const S)).a), 8);\n"
        "    let m = capture_from_user(d2, core::ptr::addr_of!((*(P as *const S)).b), 8);\n"
        "    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_nolint_suppresses_a_reviewed_site(tmp_path):
    src = H + (
        "    let a = read_back(P)?;\n"
        "    let b = read_back(P)?; // nolint: double-fetch reviewed\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []
