"""Controls for win10_surface — every case is a way it would line out the WRONG stubs quietly.

Nothing in this tool crashes when it is wrong; it writes a plausible block into 226 specs.
So each test pins one specific wrong answer:

  * a DATA export lined out as a code stub (a program reading the variable gets code bytes);
  * a forwarder or delay-load import missed, which shrinks the gap report without anyone
    noticing;
  * a name already declared under `-version=`/`-arch=` declared a second time;
  * the block reading its OWN previous output back as "already declared", so a re-run can
    never shrink it;
  * the writer re-flowing the hand-written part of a spec. Its first version normalized
    trailing blank lines and missing final newlines, a spurious hunk in 231 specs.
"""

from __future__ import annotations

import importlib.util
import shutil
import struct
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location("win10_surface", REPO / "tools" / "win10_surface.py")
assert _spec and _spec.loader
w = importlib.util.module_from_spec(_spec)
sys.modules["win10_surface"] = w
_spec.loader.exec_module(w)


# --- a minimal PE32+ builder -----------------------------------------------------------------

TEXT, RDATA = 0x1000, 0x2000
OPT_SIZE = 240  # PE32+: 112 fixed + 16 data directories


class Rdata:
    def __init__(self) -> None:
        self.buf = bytearray()

    def put(self, b: bytes, align: int = 8) -> int:
        while len(self.buf) % align:
            self.buf.append(0)
        rva = RDATA + len(self.buf)
        self.buf += b
        return rva

    def cstr(self, s: str) -> int:
        return self.put(s.encode() + b"\0", 2)


def build_pe(rd: Rdata, dirs: dict[int, tuple[int, int]], extra: list[tuple[bytes, bytes]] = ()):
    """Image with an executable .text, a non-executable .rdata, and optional named sections."""
    sections = [
        (b".text", TEXT, b"\xc3" * 0x100, 0x60000020),
        (b".rdata", RDATA, bytes(rd.buf), 0x40000040),
    ]
    va = 0x3000 + 0x1000 * ((len(rd.buf) + 0xFFF) // 0x1000)
    for name, body in extra:
        sections.append((name, va, body, 0x40000040))
        va += 0x1000 * ((len(body) + 0xFFF) // 0x1000 or 1)
    opt = bytearray(OPT_SIZE)
    struct.pack_into("<H", opt, 0, 0x20B)
    struct.pack_into("<Q", opt, 24, 0x180000000)
    struct.pack_into("<I", opt, 108, 16)
    for i, (rva, size) in dirs.items():
        struct.pack_into("<II", opt, 112 + 8 * i, rva, size)
    hdr = bytearray(b"MZ" + bytes(0x3A) + struct.pack("<I", 0x80) + bytes(0x80 - 0x40))
    hdr += b"PE\0\0" + struct.pack("<HHIIIHH", 0x8664, len(sections), 0, 0, 0, OPT_SIZE, 0x2022)
    hdr += opt
    for name, sva, body, chars in sections:
        hdr += name.ljust(8, b"\0") + struct.pack("<IIII", len(body), sva, len(body), sva)
        hdr += bytes(12) + struct.pack("<I", chars)
    img = bytearray(hdr.ljust(TEXT, b"\0"))
    for _, sva, body, _ in sections:
        img = img.ljust(sva, b"\0") + body
    return bytes(img)


def sample_pe() -> bytes:
    rd = Rdata()
    data_var = rd.put(b"\x2a\0\0\0")
    # Imports: KERNEL32!CreateFileW by name, ordinal 7 by ordinal.
    hint = rd.put(b"\0\0CreateFileW\0", 2)
    thunk = rd.put(struct.pack("<QQQ", hint, (1 << 63) | 7, 0))
    k32 = rd.cstr("KERNEL32.dll")
    imp = rd.put(struct.pack("<IIIII", thunk, 0, 0, k32, thunk) + bytes(20))
    # Delay import (RVA-based, attrs=1): USER32!MessageBoxW.
    dhint = rd.put(b"\0\0MessageBoxW\0", 2)
    dthunk = rd.put(struct.pack("<QQ", dhint, 0))
    u32 = rd.cstr("USER32.dll")
    dly = rd.put(struct.pack("<IIIIIIII", 1, u32, 0, dthunk, dthunk, 0, 0, 0) + bytes(32))
    # Exports, laid out contiguously so the forwarder string falls INSIDE the directory.
    names = ["CodeFn", "DataVar", "Fwd"]
    base = (len(rd.buf) + 7) & ~7
    exp_rva = RDATA + base
    funcs_rva, names_rva, ords_rva = exp_rva + 40, exp_rva + 56, exp_rva + 68
    strs = exp_rva + 76
    blob, offs = b"", []
    for s in ["t.dll", *names, "NTDLL.RtlFoo"]:
        offs.append(strs + len(blob))
        blob += s.encode() + b"\0"
    dll_name, n_code, n_data, n_fwd, fwd_target = offs
    exp = struct.pack("<IIHHIIIIIII", 0, 0, 0, 0, dll_name, 1, 4, 3, funcs_rva, names_rva, ords_rva)
    exp += struct.pack("<IIII", TEXT, data_var, fwd_target, TEXT + 0x10)  # 4th: noname
    exp += struct.pack("<III", n_code, n_data, n_fwd) + struct.pack("<HHH", 0, 1, 2) + b"\0\0"
    exp += blob
    assert rd.put(exp) == exp_rva
    return build_pe(rd, {0: (exp_rva, len(exp)), 1: (imp, 40), 13: (dly, 64)})


def test_exports_classify_code_data_forward_and_noname():
    pe = w.read_pe(sample_pe())
    assert pe.machine == "x86_64"
    assert pe.exports["CodeFn"] == w.Export("CodeFn")
    assert pe.exports["DataVar"].data, "an RVA in a non-executable section is a variable"
    assert pe.exports["Fwd"].forward == "NTDLL.RtlFoo"
    assert pe.noname == 1


def test_imports_include_ordinals_and_delay_loads():
    pe = w.read_pe(sample_pe())
    assert pe.imports["kernel32.dll"] == ["CreateFileW", "#7"]
    assert pe.imports["user32.dll"] == ["MessageBoxW"], "delay-loaded imports are needs too"


def test_truncated_image_is_a_peerror_not_a_crash():
    with pytest.raises(w.PEError):
        w.read_pe(sample_pe()[:0x200])


# --- api sets --------------------------------------------------------------------------------


def schema_pe(entries: dict[str, list[tuple[str, str]]], hashed_trim: int = 2) -> bytes:
    """An apisetschema-shaped image: a v6 namespace in a `.apiset` section."""
    count = len(entries)
    entry_off = 28
    values_off = entry_off + 24 * count
    nvalues = sum(len(v) for v in entries.values())
    strings_off = values_off + 20 * nvalues
    strings = bytearray()

    def wstr(s: str) -> tuple[int, int]:
        o = strings_off + len(strings)
        b = s.encode("utf-16-le")
        strings.extend(b)
        return o, len(b)

    ents, vals = bytearray(), bytearray()
    vi = 0
    for name, values in entries.items():
        no, nl = wstr(name)
        hashed = nl - 2 * hashed_trim  # drop the trailing `-N` minor revision
        ents += struct.pack("<IIIIII", 0, no, nl, hashed, values_off + 20 * vi, len(values))
        for importer, host in values:
            io, il = wstr(importer) if importer else (0, 0)
            ho, hl = wstr(host)
            vals += struct.pack("<IIIII", 0, io, il, ho, hl)
            vi += 1
    head = struct.pack("<IIIIIII", 6, 0, 0, count, entry_off, 0, 0)
    return build_pe(Rdata(), {}, [(b".apiset", head + ents + vals + strings)])


def test_apiset_resolves_every_minor_revision_to_the_default_host():
    schema = w.read_apiset_schema(
        schema_pe(
            {
                "api-ms-win-core-file-l1-2-0": [("", "kernelbase.dll")],
                "api-ms-win-crt-runtime-l1-1-0": [("", "ucrtbase.dll")],
            }
        )
    )
    assert w.resolve_apiset("API-MS-Win-Core-File-L1-2-1.dll", schema) == "kernelbase.dll"
    assert w.resolve_apiset("api-ms-win-crt-runtime-l1-1-0.dll", schema) == "ucrtbase.dll"
    assert w.resolve_apiset("api-ms-win-core-nonexistent-l1-1-0.dll", schema) == ""
    assert w.resolve_apiset("kernel32.dll", schema) is None


# --- planning and writing --------------------------------------------------------------------

SPEC = """\
@ stdcall Declared(long)
@ stdcall -version=0x600+ GatedVista(ptr)
@ stub -arch=i386 OnlyI386
"""


def ref(x64: dict[str, dict], x86: dict[str, dict] | None = None) -> dict:
    r = {"x64": {"exports": x64, "noname": 0}}
    if x86 is not None:
        r["x86"] = {"exports": x86, "noname": 0}
    return r


def fn(forward: str | None = None, data: bool = False) -> dict:
    return {"data": data, "forward": forward}


def test_plan_skips_every_declared_spelling_and_tags_arches(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_text(SPEC)
    x64 = {n: fn() for n in ("Declared", "GatedVista", "OnlyI386", "Both", "Only64")}
    x64["Fwd"] = fn("NTDLL.RtlFwd")
    x64["Var"] = fn(data=True)
    x86 = {n: fn() for n in ("Declared", "Both", "Only32")}
    plan = w.plan_module("t.dll", spec, ref(x64, x86))
    by_name = {ln.split("#")[0].split()[-1]: ln for ln in plan.lines}
    assert set(by_name) == {"Both", "Fwd", "Only64", "Only32"}, "declared names are never redone"
    assert by_name["Both"] == "@ stub -version=0xA00+ Both"
    assert by_name["Only64"] == "@ stub -arch=win64 -version=0xA00+ Only64"
    assert by_name["Only32"] == "@ stub -arch=win32 -version=0xA00+ Only32"
    assert by_name["Fwd"] == "@ stub -arch=win64 -version=0xA00+ Fwd # -> NTDLL.RtlFwd"
    assert plan.data == ["Var"], "a variable is listed for an extern, never stubbed as code"


def test_no_x86_reference_means_no_arch_guess(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_text(SPEC)
    plan = w.plan_module("t.sys", spec, ref({"KeNew": fn()}))
    assert plan.lines == ["@ stub -version=0xA00+ KeNew"]


def test_rerun_does_not_count_its_own_block_as_declared(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_text(SPEC)
    r = ref({"New": fn()})
    plan = w.plan_module("t.dll", spec, r)
    assert w.apply_block(spec, w.render_block(plan, "10.0.1"))
    again = w.plan_module("t.dll", spec, r)
    assert again.lines == plan.lines
    assert not w.apply_block(spec, w.render_block(again, "10.0.1")), "idempotent"


def test_a_hand_edit_inside_the_block_is_refused_not_reverted(tmp_path: Path):
    """Editing a dormant row in place reads as done in the diff, and the next regeneration
    restores the stub. Refuse instead; a real line above the block is the way in."""
    spec = tmp_path / "t.spec"
    spec.write_text(SPEC)
    r = ref({"New": fn()})
    w.apply_block(spec, w.render_block(w.plan_module("t.dll", spec, r), "10.0.1"))
    text = spec.read_text()
    stub = next(line for line in text.splitlines() if line.endswith(" New"))
    spec.write_text(text.replace(stub, "@ stdcall New() other.Elsewhere"))
    block = w.render_block(w.plan_module("t.dll", spec, r), "10.0.1")
    with pytest.raises(w.HandEditedBlock, match="New"):
        w.apply_block(spec, block)
    assert w.apply_block(spec, block, force=True)
    # The documented route: the real line moves above the block, and the stub leaves.
    spec.write_text("@ stdcall New() other.Elsewhere\n" + spec.read_text())
    w.apply_block(spec, w.render_block(w.plan_module("t.dll", spec, r), "10.0.1"))


@pytest.mark.parametrize(
    "original",
    [
        b"@ stdcall A()\n\n\n\n",  # trailing blank lines stay
        b"@ stdcall A()",  # no final newline
        b"@ stdcall A() # caf\xe9\n",  # a non-UTF-8 byte survives
    ],
)
def test_writer_preserves_the_hand_written_part_byte_for_byte(tmp_path: Path, original: bytes):
    spec = tmp_path / "t.spec"
    spec.write_bytes(original)
    block = w.render_block(w.Plan("t.dll", spec, ["@ stub -version=0xA00+ B"], [], []), "10")
    w.apply_block(spec, block)
    text = spec.read_bytes()
    assert text.startswith(original), "the hand-written part is never re-flowed"
    w.apply_block(spec, None)
    restored = spec.read_bytes()
    # Removing the block restores the original, except a file that lacked a final newline
    # keeps the one appending had to add.
    assert restored in (original, original + b"\n")


def test_writer_leaves_a_spec_with_nothing_to_add_untouched(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_bytes(b"@ stdcall A()\n\n\n")
    assert not w.apply_block(spec, None)
    assert spec.read_bytes() == b"@ stdcall A()\n\n\n"


# --- verify (needs a built spec2def) ---------------------------------------------------------

SPEC2DEF = next(
    (
        p
        for a in ("amd64", "i386", "arm64")
        if (p := REPO / f"build-{a}/host-tools/bin/spec2def").is_file()
    ),
    None,
) or (Path(s) if (s := shutil.which("spec2def")) else None)


@pytest.mark.skipif(SPEC2DEF is None, reason="no built spec2def (./vibeos-cli build --arch amd64)")
def test_verify_accepts_a_dormant_block_and_rejects_a_live_line(tmp_path: Path):
    assert SPEC2DEF is not None
    spec = tmp_path / "t.spec"
    spec.write_text(SPEC)
    plan = w.Plan("t.dll", spec, ["@ stub -version=0xA00+ Dormant"], [], [])
    w.apply_block(spec, w.render_block(plan, "10"))
    assert w.verify_spec(SPEC2DEF, "t.dll", spec, "0x502") == []
    live = w.Plan("t.dll", spec, ["@ stub Live"], [], [])
    w.apply_block(spec, w.render_block(live, "10"))
    problems = w.verify_spec(SPEC2DEF, "t.dll", spec, "0x502")
    assert problems and all("not dormant" in p for p in problems)


def test_gap_says_why_a_name_is_missing(tmp_path: Path):
    """`gated` means the work may already exist behind a newer -version; reporting it as
    plain missing would send someone to re-implement a body the tree already has."""
    spec = tmp_path / "t.spec"
    spec.write_text(
        "@ stdcall Live(long)\n"
        "@ stdcall -version=0x600+ Vista(ptr)\n"
        "@ stub OldStub\n"
        "@ stdcall -arch=i386 X86Only()\n"
    )
    plan = w.Plan("t.dll", spec, ["@ stub -version=0xA00+ Lined"], [], [])
    w.apply_block(spec, w.render_block(plan, "10"))
    got = w._classify(spec, "x86_64", 0x502)
    assert got == {
        "Live": "ok",
        "Vista": "gated",
        "OldStub": "stub",
        "X86Only": "arch",
    }, "a name only in the generated block is absent here, which gap reports as surface-only"


def test_gap_reports_names_windows_lacks_apart_from_our_gap(tmp_path: Path, monkeypatch):
    """Cheat Engine imports kernel32!AccessCheck, which Windows' kernel32 does not export
    either (a delay-load that never resolves there). Counting it as our gap sends someone
    to add an export Windows never had."""
    app = tmp_path / "App" / "a.exe"
    app.parent.mkdir()
    pe = w.PE("x86_64", imports={"k.dll": ["Have", "Missing", "NotOnWindows"]})
    monkeypatch.setattr(w, "iter_images", lambda corpus: [("App", app)])
    monkeypatch.setattr(w, "read_pe_file", lambda p: pe)
    monkeypatch.setattr(w, "module_specs", lambda root: {"k.dll": [tmp_path / "k.spec"]})
    monkeypatch.setattr(w, "export_version", lambda root: 0x502)
    monkeypatch.setattr(w, "_windows_files", lambda win, m: (tmp_path / m, None))
    monkeypatch.setattr(w, "_classify", lambda spec, arch, version: {"Have": "ok"})
    ref = {"windows": str(tmp_path), "modules": {"k.dll": _ref("Have", "Missing")}}
    res = w.gap([tmp_path], ref, {}, root=tmp_path, ros={})
    assert [(r["fn"], r["why"]) for r in res["missing_functions"]] == [("Missing", "surface-only")]
    assert res["not_on_windows"] == [{"dll": "k.dll", "fn": "NotOnWindows", "apps": ["App"]}]


@pytest.mark.parametrize(
    ("dll", "redist"),
    [
        ("msvcrt.dll", False),  # the OS CRT: matched by a bare `msvcr` prefix once
        ("msvcp_win.dll", False),
        ("msvcr120.dll", True),
        ("msvcp140.dll", True),
        ("vcruntime140_1.dll", True),
        ("d3dx9_43.dll", True),
    ],
)
def test_redistributables_never_swallow_os_modules(dll: str, redist: bool):
    assert bool(w.REDIST.match(dll)) is redist


# --- vista/kernelbase forwards and our loader's api-set table --------------------------------


def test_ros_apiset_table_reads_rows_and_skips_commented_ones():
    text = (
        '    { RTL_CONSTANT_STRING(L"api-ms-win-crt-runtime-l1-1-0"), '
        'RTL_CONSTANT_STRING(L"ucrtbase.dll"), APISET_WIN10 },\n'
        '    // { RTL_CONSTANT_STRING(L"api-ms-win-audiocore-pal-l1-1-0"), '
        'RTL_CONSTANT_STRING(L""), APISET_WIN81 },\n'
    )
    assert w.read_ros_apisets(text) == {"api-ms-win-crt-runtime-l1-1-0": "ucrtbase.dll"}


def _ref(*names: str) -> dict:
    return {"x64": {"exports": {n: {"data": False, "forward": None} for n in names}}}


def test_forwards_cover_exactly_where_the_module_row_does_not(tmp_path: Path):
    mod = tmp_path / "k.spec"
    mod.write_text(
        "@ stdcall -version=0x600+ Gated(ptr)\n"
        "@ stdcall -arch=arm64 ArmOnly(ptr) other.ArmOnly\n"
        "@ stdcall Live(long)\n"
        "@ stdcall -version=0xA00+ Newer(ptr)\n"
    )
    vista = tmp_path / "kv.spec"
    vista.write_text(
        "@ stdcall Gated(ptr)\n@ stdcall ArmOnly(ptr)\n@ stdcall Live(long)\n"
        "@ stdcall Absent(long long)\n@ stdcall Newer(ptr)\n@ stdcall NotOnWindows()\n"
    )
    lines = w.plan_forwards(
        "k.dll",
        mod,
        [("kv", vista)],
        _ref("Gated", "ArmOnly", "Live", "Absent", "Newer"),
        0x502,
    )
    assert lines == [
        "@ stdcall Absent(long long) kv.Absent",  # undeclared: unconditional, provider's sig
        "@ stdcall -arch=i386,x86_64 ArmOnly(ptr) kv.ArmOnly",
        "@ stdcall -version=0x400-0x5ff Gated(ptr) kv.Gated",  # off where the real row starts
        "@ stdcall -version=0x400-0x9ff Newer(ptr) kv.Newer",
    ], "Live is exported already, and Windows does not export NotOnWindows: neither forwards"


def test_the_first_provider_that_has_a_name_wins(tmp_path: Path):
    mod = tmp_path / "kb.spec"
    mod.write_text("@ stdcall Mine()\n")
    a, b = tmp_path / "a.spec", tmp_path / "b.spec"
    a.write_text("@ stdcall Both(ptr)\n")
    b.write_text("@ stdcall Both(ptr)\n@ stdcall OnlyB(long)\n")
    lines = w.plan_forwards("kb.dll", mod, [("a", a), ("b", b)], _ref("Both", "OnlyB"), 0x502)
    assert lines == ["@ stdcall Both(ptr) a.Both", "@ stdcall OnlyB(long) b.OnlyB"]


def test_two_generated_blocks_stay_put_when_either_is_rewritten(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_text("@ stdcall Live(long)\n")
    stubs = w.render_block(w.Plan("t.dll", spec, ["@ stub -version=0xA00+ X"], [], []), "10")
    fwds = w.render_forwards("t.dll", ["tv"], ["@ stdcall Y() tv.Y"])
    w.apply_block(spec, stubs)
    w.apply_block(spec, fwds, w.FWD_BEGIN, w.FWD_END)
    first = spec.read_bytes()
    # Re-running either tool with the same content must not move or rewrite anything:
    # the first version re-appended, swapped the blocks, and staled every build's media.
    assert not w.apply_block(spec, stubs)
    assert not w.apply_block(spec, fwds, w.FWD_BEGIN, w.FWD_END)
    assert spec.read_bytes() == first


def _pe(**exports: str | None) -> w.PE:
    pe = w.PE("x86_64")
    pe.exports = {n: w.Export(n, forward=f) for n, f in exports.items()}
    return pe


def test_forward_chains_follow_renames_and_catch_cycles_and_dead_ends():
    dlls = {
        "k.dll": _pe(Precise="kv.Precise", Loop="kv.Loop", Dead="kv.Missing"),
        "kv.dll": _pe(Precise="k.Coarse", Loop="k.Loop"),
    }
    dlls["k.dll"].exports["Coarse"] = w.Export("Coarse")
    problems = w.check_forward_chains(dlls, "k.dll")
    assert problems == [
        "k.dll!Dead: forward chain ends at missing kv.Missing",
        "k.dll!Loop: forward CYCLE through k.Loop",
    ], "Precise leaves and re-enters k under another name, which is fine"


def test_ros_apiset_host_ignores_only_the_trailing_minor_version():
    ros = {"api-ms-win-core-realtime-l1-1-0": "a.dll", "api-ms-win-core-realtime-l1-1-2": "b.dll"}
    assert w.ros_apiset_host("API-MS-Win-Core-Realtime-L1-1-0.dll", ros) == "a.dll"
    assert w.ros_apiset_host("api-ms-win-core-realtime-l1-1-1.dll", ros) == "b.dll"
    assert w.ros_apiset_host("api-ms-win-core-realtime-l1-2-0.dll", ros) is None


# --- gated ------------------------------------------------------------------------------------


def test_ungate_line_drops_only_a_plain_version_floor():
    assert (
        w.ungate_line("@ stdcall -version=0x600+ -arch=x86_64 F(ptr) G # note\n")
        == "@ stdcall -arch=x86_64 F(ptr) G # note\n"
    )
    # A range or a list is a deliberate window, not a floor: a human decides.
    assert w.ungate_line("@ stdcall -version=0x600-0x601 F(ptr)\n") is None
    assert w.ungate_line("@ stdcall -version=0x502,0x600+ F(ptr)\n") is None


def test_stdcall_bytes_is_the_i386_decoration():
    assert w.stdcall_bytes("(ptr long)") == 8
    assert w.stdcall_bytes("(int64 ptr double)") == 20
    assert w.stdcall_bytes("()") == 0


def test_apply_ungate_touches_only_rows_linked_everywhere(tmp_path: Path):
    spec = tmp_path / "t.spec"
    spec.write_text(
        "@ stdcall -version=0x600+ Linked(ptr)\n"
        "@ stdcall -version=0x600+ Partly(ptr)\n"
        "@ stdcall -version=0x600+ Excluded(ptr)\n"
    )
    rows = w.spec_rows(spec.read_text())

    def g(name: str, arches: dict[str, str], excluded: str | None = None) -> w.GatedRow:
        return w.GatedRow("t.dll", spec, name, rows[name][0], arches, excluded)

    changed, manual = w.apply_ungate(
        [
            g("Linked", {"i386": "linked", "x86_64": "linked"}),
            g("Partly", {"i386": "signature:@4!=@8", "x86_64": "linked"}),
            g("Excluded", {"i386": "linked"}, "returns E_NOTIMPL"),
        ]
    )
    assert (changed, manual) == (1, [])
    assert spec.read_text().splitlines() == [
        "@ stdcall Linked(ptr)",
        "@ stdcall -version=0x600+ Partly(ptr)",
        "@ stdcall -version=0x600+ Excluded(ptr)",
    ]


def test_a_forward_to_a_provider_that_links_back_is_private(tmp_path: Path):
    """ws2help links ws2_32's import library: a ws2_32 forward to ws2help that landed in
    that library would collide with ws2help's own definition at ws2help's link."""
    mod = tmp_path / "ws2_32.spec"
    mod.write_text("@ stdcall -version=0x600+ WahX(ptr)\n")
    prov = tmp_path / "ws2help.spec"
    prov.write_text("@ stdcall WahX(ptr)\n")
    ref = {"x64": {"exports": {"WahX": {"data": False}}}}
    lines = w.plan_forwards("ws2_32.dll", mod, [("ws2help", prov)], ref, 0x502)
    assert lines == ["@ stdcall -version=0x400-0x5ff -private WahX(ptr) ws2help.WahX"]
