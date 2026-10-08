"""The flexible-array panic reader: length discrimination and corpus honesty."""

from __future__ import annotations

import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_flex_array_panic as s


class _Proc:
    def __init__(self, stdout: str) -> None:
        self.stdout, self.stderr, self.returncode = stdout, "", 0


def _run(monkeypatch, listing: str):
    """(symbol, length) pairs — the context column is exercised separately below."""
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return [(sym, ln) for sym, ln, _ in s.scan_object(Path("fake.a"))]


def _run_explain(monkeypatch, listing: str):
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return s.scan_object(Path("fake.a"), explain=True)


# The REAL shape. `panic_bounds_check(index, len, &Location)` under cdecl emits
# `push loc; push len; push index`, so the length is the MIDDLE push. The original
# fixture here had only two pushes and encoded the same wrong assumption as the code it
# was testing -- which is precisely why keying on the last push (the INDEX) survived
# review: the test could not tell the two apart.
FLEX = """
00001000 <_Victim@8>:
    1000: 68 70 05 00 00               \tpushl\t$0x570                  # imm = 0x570
\t\t\t00001001:  IMAGE_REL_I386_DIR32\t.rdata
    1005: 6a 01                        \tpushl\t$0x1
    1007: 6a 01                        \tpushl\t$0x1
    1009: e8 00 00 00 00               \tcalll\t0x100e
\t\t\t0000100a:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE
"""

# A real fixed array: len 2, so index 1 is in range and index 2 is an ordinary
# off-the-end, not a bindgen flexible array.
REAL = FLEX.replace("pushl\t$0x1\n    1007", "pushl\t$0x2\n    1007").replace(
    "_Victim@8", "_Innocent@8"
)

# The length in a REGISTER: a runtime-length slice. A [T; 1] cannot have a dynamic
# length, so this fails the grading protocol's "is the declared length a LITERAL 1"
# test by construction and must not be reported.
DYNAMIC_LEN = FLEX.replace(
    "    1005: 6a 01                        \tpushl\t$0x1\n",
    "    1005: 50                           \tpushl\t%eax\n",
).replace("_Victim@8", "_DynamicLen@8")


def test_a_length_one_check_is_reported(monkeypatch):
    assert _run(monkeypatch, FLEX) == [("_Victim@8", 1)]


def test_a_real_fixed_array_is_left_alone(monkeypatch):
    """SubKeyLists[Type] on [T; 2] is a correct check, not a flexible array."""
    assert _run(monkeypatch, REAL) == []


def test_a_runtime_length_slice_is_not_a_flexible_array(monkeypatch):
    """Measured over the i386 corpus, keying on the INDEX reported five of these --
    sending graders to rtl_strings functions that index a plain slice and declare no
    array at all."""
    assert _run(monkeypatch, DYNAMIC_LEN) == []


def test_an_unrelated_instruction_between_does_not_carry_the_length(monkeypatch):
    """An instruction that is not part of the call setup clears the pending arguments,
    so a length from some earlier, unrelated computation cannot be attributed here."""
    listing = FLEX.replace(
        "    1009: e8 00 00 00 00               \tcalll\t0x100e\n",
        "    1008: 90                           \tnop\n"
        "    1009: e8 00 00 00 00               \tcalll\t0x100e\n",
    )
    assert _run(monkeypatch, listing) == []


def test_the_length_does_not_leak_across_symbols(monkeypatch):
    listing = (
        "00001000 <_Pusher@4>:\n    1000: 6a 01                        \tpushl\t$0x1\n"
        "00002000 <_Other@4>:\n"
        "    2000: e8 00 00 00 00               \tcalll\t0x2005\n"
        "\t\t\t00002001:  IMAGE_REL_I386_REL32\t__ZN4core9panicking18panic_bounds_checkE\n"
    )
    assert _run(monkeypatch, listing) == []


# amd64 and arm64, transcribed from real atapi-legacy-rust disassembly with the length
# changed to 1. Win64 passes (index, len, &Location) in rcx/rdx/r8 and the AAPCS in
# x0/x1/x2; the index is left in its register by the hot path, so only the length and
# the Location are set in the panic block itself.
X64_FLEX = """
0000000000001000 <Victim>:
    1000: 4c 8d 05 58 00 00 00         \tleaq\t0x58(%rip), %r8         # 0x105f <Victim+0x5f>
\t\t0000000000001003:  IMAGE_REL_AMD64_REL32\t.rdata
    1007: ba 01 00 00 00               \tmovl\t$0x1, %edx
    100c: e8 00 00 00 00               \tcallq\t0x1011 <Victim+0x11>
\t\t000000000000100d:  IMAGE_REL_AMD64_REL32\t_ZN4core9panicking18panic_bounds_checkE
"""

A64_FLEX = """
0000000000000000 <Victim>:
     5f0: 90000002     \tadrp\tx2, 0x0 <Victim>
\t\t00000000000005f0:  IMAGE_REL_ARM64_PAGEBASE_REL21\tanon.aa.6
     5f4: 91000042     \tadd\tx2, x2, #0x0
\t\t00000000000005f4:  IMAGE_REL_ARM64_PAGEOFFSET_12A\tanon.aa.6
     5f8: 52800021     \tmov\tw1, #0x1                // =1
     5fc: 94000000     \tbl\t0x5fc <Victim+0x5fc>
\t\t00000000000005fc:  IMAGE_REL_ARM64_BRANCH26\t_ZN4core9panicking18panic_bounds_checkE
"""


def _run_arch(monkeypatch, listing: str, arch: str):
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(listing))
    return [(sym, ln) for sym, ln, _ in s.scan_object(Path("fake.a"), arch=arch)]


def test_a_length_one_check_is_reported_on_amd64(monkeypatch):
    assert _run_arch(monkeypatch, X64_FLEX, "amd64") == [("Victim", 1)]


def test_a_length_one_check_is_reported_on_arm64(monkeypatch):
    assert _run_arch(monkeypatch, A64_FLEX, "arm64") == [("Victim", 1)]


def test_a_real_fixed_array_is_left_alone_on_64_bit(monkeypatch):
    """atapi-legacy-rust's real shape: `movl $0x5, %edx` / `mov w1, #0x5`."""
    x64 = X64_FLEX.replace("movl\t$0x1, %edx", "movl\t$0x5, %edx")
    a64 = A64_FLEX.replace(
        "mov\tw1, #0x1                // =1", "mov\tw1, #0x5                // =5"
    )
    assert _run_arch(monkeypatch, x64, "amd64") == []
    assert _run_arch(monkeypatch, a64, "arm64") == []


def test_a_register_length_is_a_slice_on_64_bit(monkeypatch):
    """A length copied from another register is a runtime length -- never a [T; 1]."""
    x64 = X64_FLEX.replace("movl\t$0x1, %edx", "movl\t%eax, %edx")
    a64 = A64_FLEX.replace("mov\tw1, #0x1                // =1", "mov\tw1, w19")
    assert _run_arch(monkeypatch, x64, "amd64") == []
    assert _run_arch(monkeypatch, a64, "arm64") == []


def test_a_later_write_to_the_length_register_wins(monkeypatch):
    """The LAST constant in the slot is the argument; an earlier one was overwritten."""
    x64 = X64_FLEX.replace(
        "    1007: ba 01 00 00 00               \tmovl\t$0x1, %edx\n",
        "    1007: ba 01 00 00 00               \tmovl\t$0x1, %edx\n"
        "    1008: 89 c2                        \tmovl\t%eax, %edx\n",
    )
    assert _run_arch(monkeypatch, x64, "amd64") == []


def test_an_unrelated_instruction_ends_the_setup_on_64_bit(monkeypatch):
    """Same discipline as i386: only the call's own argument setup may intervene, so a
    constant written for some earlier computation is never attributed to this call."""
    x64 = X64_FLEX.replace(
        "    100c: e8",
        "    100b: 48 85 f6                     \ttestq\t%rsi, %rsi\n    100c: e8",
    )
    a64 = A64_FLEX.replace(
        "     5fc: 94000000",
        "     5fa: eb01001f     \tcmp\tx0, x1\n     5fc: 94000000",
    )
    assert _run_arch(monkeypatch, x64, "amd64") == []
    assert _run_arch(monkeypatch, a64, "arm64") == []


def test_another_call_clobbers_the_tracked_arguments(monkeypatch):
    """rdx/x1 are caller-saved: a call to anything else between the write and the
    panic means the value the panic sees is not the one we read."""
    x64 = X64_FLEX.replace(
        "    100c: e8",
        "    100a: e8 00 00 00 00               \tcallq\t0x100f <Victim+0xf>\n"
        "\t\t000000000000100b:  IMAGE_REL_AMD64_REL32\tsome_helper\n    100c: e8",
    )
    assert _run_arch(monkeypatch, x64, "amd64") == []


def test_a_constant_index_into_a_one_element_array_is_reported(monkeypatch):
    """Loop peeling: LLVM proves the SECOND iteration of a walk over a [T; 1] is out of
    bounds and emits panic(index 1, len 1) unconditionally. diskpart's Extents walk
    shipped this on all three arches; it is the strongest form of the finding."""
    x64 = X64_FLEX.replace(
        "    1007: ba 01 00 00 00",
        "    1006: b9 01 00 00 00               \tmovl\t$0x1, %ecx\n    1007: ba 01 00 00 00",
    )
    assert _run_arch(monkeypatch, x64, "amd64") == [("Victim", 1)]


def test_symbols_normalize_to_one_name_across_arches():
    """The allowlist is written once per site, so the same function must have the same
    key on every arch and across rebuilds that move a hash."""
    i386 = "__ZN17atapi_legacy_rust5entry29AtapiFindNativeModeController17hb6081ba0f0486feeE@24"
    x64 = "_ZN17atapi_legacy_rust5entry29AtapiFindNativeModeController17h203528268eed9fccE"
    want = "_ZN17atapi_legacy_rust5entry29AtapiFindNativeModeControllerE"
    assert s.normalize_symbol(i386, "i386") == want
    assert s.normalize_symbol(x64, "amd64") == want
    assert s.normalize_symbol(x64 + ".llvm.2798096731955180627", "arm64") == want
    assert s.normalize_symbol("_SHTestTokenPrivilegeW@8", "i386") == "SHTestTokenPrivilegeW"
    # Only i386 decorates C names: a 64-bit leading underscore is part of the name.
    assert s.normalize_symbol("_Private", "amd64") == "_Private"


def test_allowlist_scope_limits_an_entry_to_its_arches(tmp_path, monkeypatch):
    """An entry graded on the arches where the site EXISTS must not read as stale on an
    arch that never builds it -- and an unscoped entry applies everywhere."""
    allow = tmp_path / "allow.txt"
    allow.write_text(
        "everywhere-rust: Everywhere   # rationale\n"
        "@amd64,arm64 wide-rust: _ZN4wide1fE\n"
        "@i386 narrow-rust: _ZN6narrow1gE\n"
    )
    monkeypatch.setattr(s, "ALLOWLIST", allow)
    assert s.graded_ok("i386") == {
        ("everywhere-rust", "Everywhere"),
        ("narrow-rust", "_ZN6narrow1gE"),
    }
    assert s.graded_ok("arm64") == {("everywhere-rust", "Everywhere"), ("wide-rust", "_ZN4wide1fE")}


def test_a_normalized_entry_is_not_normalized_twice(tmp_path, monkeypatch):
    """The file holds normalized names. Re-applying i386's `_`-strip to one turned
    `_ZN17atapi_legacy_rust…E` into `ZN17…` and made the entry stale on i386 alone."""
    allow = tmp_path / "allow.txt"
    allow.write_text("atapi-legacy-rust: _ZN17atapi_legacy_rust5entry1fE\n")
    monkeypatch.setattr(s, "ALLOWLIST", allow)
    for arch in s.ARCHES:
        assert s.graded_ok(arch) == {("atapi-legacy-rust", "_ZN17atapi_legacy_rust5entry1fE")}


def test_v0_mangling_drops_the_crate_disambiguator():
    """The i386 cargo cell mangles v0, whose crate root carries a disambiguator."""
    v0 = "_RNvNtCs6vQPevndiDL_7ex_rust7sysinfo26QSI_SystemNumaProcessorMap.llvm.42"
    assert (
        s.normalize_symbol("_" + v0, "i386")
        == "_RNvNtC7ex_rust7sysinfo26QSI_SystemNumaProcessorMap"
    )


def test_a_linked_image_call_names_its_callee_on_the_call_line(monkeypatch):
    """A linked PE has no relocations: `callq 0x… <…panic_bounds_check…>` decides, and
    any other resolved call clobbers the setup."""
    linked = """
0000000140001000 <Victim>:
   140001430: 4c 8d 05 f1 db 02 00        \tleaq\t0x2dbf1(%rip), %r8      # 0x14002f028
   140001437: ba 01 00 00 00              \tmovl\t$0x1, %edx
   14000143c: e8 26 86 01 00     \tcallq\t0x140019a67 <_ZN4core9panicking18panic_bounds_checkE>
"""
    assert _run_arch(monkeypatch, linked, "amd64") == [("Victim", 1)]
    other = linked.replace(
        "   14000143c:",
        "   14000143a: e8 26 86 01 00              \tcallq\t0x140019000 <helper>\n   14000143c:",
    )
    assert _run_arch(monkeypatch, other, "amd64") == []


def test_a_wildcard_entry_grades_the_function_in_every_image():
    graded = {("*", "_ZN7network4pure11format_ipv6E")}
    assert s.is_graded(graded, "ntdll-cargo", "_ZN7network4pure11format_ipv6E")
    assert not s.is_graded(graded, "ntdll-cargo", "_ZN7network4pure5otherE")


def test_cargo_images_are_declared_by_the_build_graph_for_one_arch(tmp_path):
    """The boot path is cargo-linked PEs. Their corpus comes from build.ninja -- the
    target dir also keeps images earlier configurations produced -- and only this arch's."""
    (tmp_path / "build.ninja").write_text(
        "build /tmp/vibeos/sdk/lib/rust/target/smss-cargo-amd64-cargo-link/"
        "x86_64-pc-windows-gnu/release/smss.exe: CUSTOM_COMMAND\n"
        "build sdk/lib/rust/target/smss-cargo-i386-cargo-link/i686-kernel/release/smss.exe: X\n"
        "  # deps are not images: sdk/lib/rust/target/smss-cargo-amd64-cargo-link/"
        "x86_64-pc-windows-gnu/release/deps/libfoo.rlib\n"
    )
    got = s.declared_cargo_images(tmp_path, "amd64")
    assert [(c, p.name) for c, p in got] == [("smss-cargo", "smss.exe")]
    assert str(got[0][1]).startswith(str(s.REPO / "sdk/lib/rust/target/smss-cargo-amd64"))


def test_a_linked_location_resolves_by_address():
    """No relocations in a linked image: the &Location and its file pointer are both
    resolved addresses into the one .rdata."""
    import struct

    base = 0x14002F000
    name = b"win32k-rust/src/freetype.rs"
    rdata = bytearray(0x100)
    rdata[0x80 : 0x80 + len(name)] = name
    struct.pack_into("<QQII", rdata, 0x20, base + 0x80, len(name), 4694, 49)
    got = s.resolve_va((base, bytes(rdata)), base + 0x20, ptr64=True)
    assert got == "win32k-rust/src/freetype.rs:4694:49"
    assert "outside" in s.resolve_va((base, bytes(rdata)), base + 0x1000, ptr64=True)


def test_i386_fastcall_decoration_is_stripped():
    assert s.normalize_symbol("@ftGetFontUnicodeRanges@8", "i386") == "ftGetFontUnicodeRanges"


def test_the_shipped_allowlist_parses_on_every_arch():
    """A malformed scope would silently grade nothing on that arch."""
    for arch in s.ARCHES:
        assert s.graded_ok(arch), f"no allowlist entries apply on {arch}"


def test_arch_is_required():
    """No default arch, like the rest of vibeos-cli: a reader that defaulted to i386 is
    how the 64-bit corpus went unread."""
    try:
        s.main([])
    except SystemExit as e:
        assert e.code == 2  # argparse's usage error
    else:
        raise AssertionError("main() accepted a missing --arch")


def test_the_corpus_comes_from_the_workspace_and_is_not_empty():
    corpus = s.declared_corpus()
    assert len(corpus) > 100
    assert "cmlib-rust" in corpus


def test_a_crate_target_dir_is_not_mistaken_for_the_artifact(tmp_path):
    real = tmp_path / "sdk" / "lib" / "cmlib" / "libcmlib_rust.a"
    inter = tmp_path / "sdk" / "lib" / "cmlib" / "cmlib-rust-target" / "libcmlib_rust.a"
    for p in (real, inter):
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(b"")
    assert s.find_artifact(tmp_path, "cmlib-rust") == real


def test_the_crate_target_dir_is_PRUNED_not_filtered_afterwards(tmp_path, monkeypatch):
    """Selecting the right artifact is not the same as never entering the wrong tree.

    The old spelling was a recursive `rglob(want)` rooted at the build dir, with an
    `if "-target/" not in str(p)` filter after it — which passes the test above while
    still walking every file under the one
    directory a concurrent build is rewriting — and pathlib's recursive glob lets a
    FileNotFoundError out of the generator, so the scanner dies with a traceback that
    names no source defect. Assert the descent, not just the answer.
    """
    real = tmp_path / "sdk" / "lib" / "cmlib" / "libcmlib_rust.a"
    target = tmp_path / "sdk" / "lib" / "cmlib" / "cmlib-rust-target" / "deep" / "deeper"
    real.parent.mkdir(parents=True, exist_ok=True)
    real.write_bytes(b"")
    target.mkdir(parents=True)
    (target / "libcmlib_rust.a").write_bytes(b"")

    seen: list[str] = []
    real_walk = os.walk

    def spy(top, *a, **kw):
        for dirpath, dirnames, filenames in real_walk(top, *a, **kw):
            seen.append(dirpath)
            yield dirpath, dirnames, filenames

    monkeypatch.setattr(s.os, "walk", spy)
    assert s.find_artifact(tmp_path, "cmlib-rust") == real
    # The spy must have run: an rglob implementation calls os.walk zero times, and
    # "no -target in an empty list" would then be true for the wrong reason.
    assert seen, "os.walk was never called — this assertion would pass vacuously"
    assert not any("-target" in p for p in seen), f"descended into a pruned tree: {seen}"


def test_context_is_collected_only_when_asked(monkeypatch):
    """The corpus-wide default run must not carry windows it will never print."""
    assert _run_explain(monkeypatch, FLEX)[0][2]
    monkeypatch.setattr(s.subprocess, "run", lambda *a, **k: _Proc(FLEX))
    assert s.scan_object(Path("fake.a"))[0][2] == []


def test_context_names_what_the_check_was_about(monkeypatch):
    """WHY --explain exists: the flagged symbol is where the check was INLINED TO, so
    its own source may declare no array at all. The relocation in the window is the
    only place the real array is named."""
    listing = FLEX.replace(
        "    1005: 6a 01                        \tpushl\t$0x1\n",
        "    1004: 8d 3d 00 00 00 00            \tleal\t0x0, %edi\n"
        "\t\t\t00001005:  IMAGE_REL_I386_DIR32\t_ReparseBuffer_PathBuffer\n"
        "    1005: 6a 01                        \tpushl\t$0x1\n",
    )
    ctx = _run_explain(monkeypatch, listing)[0][2]
    assert any("_ReparseBuffer_PathBuffer" in line for line in ctx)


def test_the_window_does_not_span_symbols(monkeypatch):
    """A window that leaked across a symbol boundary would name another function's
    array and send the grader to the wrong file."""
    listing = (
        "00000900 <_Neighbour@4>:\n"
        "\t\t\t00000901:  IMAGE_REL_I386_DIR32\t_NOT_THIS_ONE\n" + FLEX.lstrip("\n")
    )
    ctx = _run_explain(monkeypatch, listing)[0][2]
    assert not any("_NOT_THIS_ONE" in line for line in ctx)


def test_a_large_immediate_with_objdumps_trailing_comment_is_read(monkeypatch):
    """objdump renders the 5-byte push as `pushl $0x570  # imm = 0x570`. Anchoring the
    length regex on end-of-line skipped every large immediate -- and the &Location
    argument is always one, so five sites reported no source location at all."""
    assert _run(monkeypatch, FLEX) == [("_Victim@8", 1)]
