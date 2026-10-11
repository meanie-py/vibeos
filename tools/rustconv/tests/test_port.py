"""rustconv port: the hand-offs it automates are idempotent and land where they should."""

import json
from pathlib import Path

import pytest

from rustconv import port

SCRIPT = """case "$DRIVER" in
  alpha)
    USERMODE=1
    DRIVER_DIR="${SRC}/dll/win32/alpha"
    ;;
  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;
esac
"""

ARM = """  beta)
    USERMODE=1
    DRIVER_DIR="${SRC}/base/setup/lib"
    ;;
"""


def test_profile_arm_is_inserted_before_the_default_and_replaced_in_place():
    once = port.upsert_profile_arm(SCRIPT, ARM, "beta")
    assert once.index("  beta)") < once.index("  *) echo")
    assert once.count("  alpha)") == 1
    # idempotent: the same arm again changes nothing
    assert port.upsert_profile_arm(once, ARM, "beta") == once
    # a re-derived arm REPLACES the old one rather than adding a second
    changed = port.upsert_profile_arm(once, ARM.replace("base/setup/lib", "base/setup/x"), "beta")
    assert (
        changed.count("  beta)") == 1 and "base/setup/x" in changed and "setup/lib" not in changed
    )


def test_a_multi_island_arm_is_replaced_not_appended_again():
    multi = ARM.replace("  beta)", "  beta|beta-types_1)")
    script = port.upsert_profile_arm(SCRIPT, multi, "beta")
    script = script.replace("  *) echo", multi + "  *) echo")  # the duplicate derives left
    once = port.upsert_profile_arm(script, ARM, "beta")
    assert once.count("  beta)") == 1 and "beta|beta-types_1" not in once
    assert once.count("  alpha)") == 1


def test_a_dead_ifndef_reactos_case_arm_is_dropped(tmp_path):
    c = tmp_path / "m.c"
    c.write_text(
        "BOOL WINAPI DllMain(HINSTANCE h, DWORD reason, LPVOID r)\n{\n    switch (reason)\n    {\n"
        "#ifndef __REACTOS__\n        case DLL_WINE_PREATTACH:\n            return FALSE;\n#endif\n"
        "        case DLL_PROCESS_ATTACH:\n            hcpl = h;\n    }\n    return TRUE;\n}\n"
    )
    rs = tmp_path / "m.rs"
    rs.write_text(
        "fn DllMain(reason: DWORD) -> BOOL {\n    match reason {\n"
        "        DLL_WINE_PREATTACH => {\n            return FALSE;\n        }\n"
        "        DLL_PROCESS_ATTACH | DLL_WINE_PREATTACH => {\n            hcpl = h;\n        }\n"
        "        _ => {}\n    }\n    return TRUE;\n}\n"
    )
    assert port._drop_dead_arms(rs, c) == ["DLL_WINE_PREATTACH", "DLL_WINE_PREATTACH"]
    out = rs.read_text()
    assert "DLL_WINE_PREATTACH" not in out and "return FALSE" not in out
    assert "        DLL_PROCESS_ATTACH => {" in out
    assert port._drop_dead_arms(rs, c) == []


def test_a_translations_own_wine_debug_channel_storage_is_dropped(tmp_path):
    rs = tmp_path / "security.rs"
    rs.write_text(
        "use crate::prelude::*;\n"
        'pub static __wine_dbch___default: [u8; 8] = *b"inetcpl\\0";\n'
        "fn f() {\n    if __wine_dbch___default[0] as c_int & 8 != 0 {}\n}\n"
    )
    assert port._wine_debug_storage(rs) == ["__wine_dbch___default", "[0] -> .flags x1"]
    out = rs.read_text()
    assert "static" not in out and "(*__wine_dbch___default).flags as c_int" in out
    assert port._wine_debug_storage(rs) == []


def test_script_without_a_default_arm_is_refused():
    with pytest.raises(port.PortError):
        port.upsert_profile_arm("case x in\nesac\n", ARM, "beta")


def test_run_cost_reads_the_last_running_total():
    log = "  [1/3] A... OK  [running $0.02]\n  [2/3] B... OK  [running $0.06]\n  [3/3] C... CACHED ($0)\n"
    assert port._run_cost(log) == pytest.approx(0.06)
    assert port._run_cost("nothing paid") == 0.0


def test_workspace_member_is_added_once(tmp_path, monkeypatch):
    manifest = tmp_path / "sdk" / "lib" / "rust" / "Cargo.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('[workspace]\nmembers = [\n    "a-rust",\n]\n')
    monkeypatch.setattr(port, "REPO", tmp_path)
    assert port.add_workspace_member("b-rust")
    assert not port.add_workspace_member("b-rust")
    text = manifest.read_text()
    assert text.count('"b-rust"') == 1 and text.index('"a-rust"') < text.index('"b-rust"')


def test_to_buy_sums_the_resume_cache_lines():
    log = (
        "  Resume cache: 0 banked ($0 replay), 19 to buy   [x]\n"
        "  Resume cache: 3 banked ($0 replay), 2 to buy   [y]\n"
    )
    assert port._to_buy(log) == 21
    assert port._to_buy("Resumed 8 unit(s) from cache") == 0


def test_a_failed_glob_import_is_reported_as_masking():
    def e0432(line: int, text: str) -> dict:
        span = {
            "is_primary": True,
            "file_name": "a.rs",
            "line_start": line,
            "text": [{"text": text}],
        }
        return {"level": "error", "code": {"code": "E0432"}, "spans": [span]}

    diags = [e0432(1, "use x::*;"), e0432(2, "use crate::y::Z;")]
    assert port._masking_glob(diags) == ["a.rs:1"]


def test_tokens_reads_both_log_formats():
    log = "  [3] A OK (6118/16287)  [running $0.07]\n  [4/9] B... OK (6736 in / 1312 out)\n"
    assert port._tokens(log) == (2, 12854, 17599)


def test_type_defined_in_finds_typedef_shapes():
    from rustconv.port import _type_defined_in

    hdr = (
        "typedef struct _MOUNTDEV_NAME {\n  USHORT NameLength;\n  WCHAR Name[1];\n"
        "} MOUNTDEV_NAME, *PMOUNTDEV_NAME;\ntypedef ULONG FOO_ID;\n"
        "struct BAR { int x; };\nVOID Use(MOUNTDEV_NAME *p);\n"
    )
    assert _type_defined_in(hdr, "MOUNTDEV_NAME") and _type_defined_in(hdr, "PMOUNTDEV_NAME")
    assert _type_defined_in(hdr, "FOO_ID") and _type_defined_in(hdr, "BAR")
    assert not _type_defined_in("VOID Use(MOUNTDEV_NAME *p);", "MOUNTDEV_NAME")


def test_a_banked_hand_function_survives_a_replay(tmp_path, monkeypatch):
    monkeypatch.setattr(port, "REPO", tmp_path)
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path / "state")
    state = {"stem": "m", "crate": "m-rust", "stages": []}
    (tmp_path / "state" / "m").mkdir(parents=True)
    (tmp_path / "state" / "m" / "state.json").write_text(json.dumps(state))
    mods = tmp_path / "sdk" / "lib" / "rust" / "m-rust" / "src" / "m"
    mods.mkdir(parents=True)
    bought = 'fn A() {\n    x();\n}\n\npub unsafe extern "C" fn V(a: u32, args: ...) {}\n'
    hand = 'pub unsafe extern "C" fn V_body(a: u32, ap: *mut c_void) {\n    y();\n}'
    (mods / "f.rs").write_text(
        bought.replace('pub unsafe extern "C" fn V(a: u32, args: ...) {}', hand)
    )
    port.stage_hand("m", ["f.rs:V_body=V"])
    (mods / "f.rs").write_text(bought)  # a replay rebuilt the module from the bank
    assert port.apply_hand("m") == 1
    assert "fn V_body(a: u32, ap: *mut c_void)" in (mods / "f.rs").read_text()
    assert "args: ..." not in (mods / "f.rs").read_text()


def test_a_c_variadic_hand_port_is_banked_as_one_piece(tmp_path, monkeypatch):
    # The extern decl, the va_entry! mod and the comments above each are one hand item: a
    # replay must put all three back over the one bought `args: ...` function.
    monkeypatch.setattr(port, "REPO", tmp_path)
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path / "state")
    state = {"stem": "m", "crate": "m-rust", "stages": []}
    (tmp_path / "state" / "m").mkdir(parents=True)
    (tmp_path / "state" / "m" / "state.json").write_text(json.dumps(state))
    mods = tmp_path / "sdk" / "lib" / "rust" / "m-rust" / "src" / "m"
    mods.mkdir(parents=True)
    bought = 'fn A() {}\n\n#[no_mangle]\npub unsafe extern "C" fn P(fmt: PCSTR, args: ...) {}\n'
    hand = (
        'extern "C" {\n    /// declared so callers pass varargs\n'
        "    pub fn P(fmt: PCSTR, ...);\n}\n\n"
        "// defined through the naked thunk\nmod P_export {\n    use super::*;\n"
        "    varargs_rust::va_entry!(P => |a| -> () {\n        x(a.get(0));\n    });\n}"
    )
    (mods / "f.rs").write_text(
        bought.replace('#[no_mangle]\npub unsafe extern "C" fn P(fmt: PCSTR, args: ...) {}', hand)
    )
    port.stage_hand("m", ["f.rs:P"])
    banked = json.loads((tmp_path / "state" / "m" / "hand" / "f.P.json").read_text())
    assert banked["text"] == hand
    (mods / "f.rs").write_text(bought)
    assert port.apply_hand("m") == 1
    assert (mods / "f.rs").read_text() == "fn A() {}\n\n" + hand + "\n"


def test_hand_span_finds_an_import_renamed_in_rust_only():
    text = (
        "fn A() {}\n\n"
        '#[cfg(target_arch = "x86")]\nextern "C" {\n    #[link_name = "Tag"]\n'
        "    static Tag_dll: u8;\n}\n"
        '#[cfg(target_arch = "x86_64")]\nextern "C" {\n    #[link_name = "__imp_Tag"]\n'
        "    static Tag_imp: *const u8;\n}\n"
    )
    lo, hi = port.hand_span(text, "Tag", where="t")
    assert text[lo:hi] == text[len("fn A() {}\n\n") :].rstrip("\n")


def test_hand_span_refuses_pieces_with_other_code_between():
    text = 'extern "C" {\n    fn P(...);\n}\nfn Other() {}\nmod P_export {\n}\n'
    with pytest.raises(port.PortError, match="other code between"):
        port.hand_span(text, "P", where="t")


def test_a_defined_callback_gets_the_c_calling_convention(tmp_path, monkeypatch):
    from rustconv import sdk_facts

    monkeypatch.setattr(port, "REPO", tmp_path)
    mods = tmp_path / "sdk" / "lib" / "rust" / "m-rust" / "src" / "m"
    mods.mkdir(parents=True)
    (mods / "a.rs").write_text(
        'unsafe extern "C" fn QueryRoutine(a: u32) -> i32 { 0 }\n'
        '#[no_mangle]\npub unsafe extern "C" fn Helper(t: unsafe extern "C" fn(u32)) {}\n'
    )
    monkeypatch.setattr(
        sdk_facts,
        "function_abis",
        lambda repo, tu, names: {"QueryRoutine": "system", "Helper": "C"},
    )
    state = {"stem": "m", "crate": "m-rust", "tus": ["x/a.c"]}
    assert port.align_definition_abis(state) == {"a.rs": ["QueryRoutine"]}
    text = (mods / "a.rs").read_text()
    assert 'unsafe extern "system" fn QueryRoutine(' in text
    assert 'pub unsafe extern "C" fn Helper(t: unsafe extern "C" fn(u32))' in text


def test_a_replay_reassembles_every_tu_and_never_buys(tmp_path, monkeypatch):
    import subprocess

    monkeypatch.setattr(port, "REPO", tmp_path)
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path / "state")
    tus = ["m/a.c", "m/b.c"]
    for tu in tus:
        (tmp_path / tu).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / tu).write_text("int f(void) { return 0; }\n")
    (tmp_path / "state" / "m").mkdir(parents=True)
    state = {"stem": "m", "crate": "m-rust", "module_dir": "m", "tus": tus, "stages": []}
    (tmp_path / "state" / "m" / "state.json").write_text(json.dumps(state))
    runs: list[list[str]] = []

    def fake_run(cmd, **kw):
        runs.append(cmd)
        out = "parallel translation completed with rejected bodies\n"
        return subprocess.CompletedProcess(cmd, 0, stdout=out, stderr="")

    monkeypatch.setattr(port.subprocess, "run", fake_run)
    port.stage_translate(
        "m", model="deepseek", cost_cap=0.0, only=[], dry_run=False, replay_only=True
    )
    assert len(runs) == 2, "every TU is reassembled even at cost cap 0"
    assert all("--replay-only" in c and "--rebuy-rejected" not in c for c in runs)


def test_a_dry_run_estimates_every_tu_at_cost_cap_zero(tmp_path, monkeypatch):
    import subprocess

    monkeypatch.setattr(port, "REPO", tmp_path)
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path / "state")
    monkeypatch.setattr(port, "_arch_only_tus", lambda state, tus: {})
    tus = ["m/a.c", "m/b.c"]
    for tu in tus:
        (tmp_path / tu).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / tu).write_text("int f(void) { return 0; }\n")
    (tmp_path / "state" / "m").mkdir(parents=True)
    state = {"stem": "m", "crate": "m-rust", "module_dir": "m", "tus": tus, "stages": []}
    (tmp_path / "state" / "m" / "state.json").write_text(json.dumps(state))
    runs: list[list[str]] = []

    def fake_run(cmd, **kw):
        runs.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, stdout="", stderr="")

    monkeypatch.setattr(port.subprocess, "run", fake_run)
    port.stage_translate("m", model="deepseek", cost_cap=0.0, only=[], dry_run=True)
    assert len(runs) == 2 and all("--dry-run" in c for c in runs)


def test_base_header_case_constants_are_recorded_not_blocking():
    from types import SimpleNamespace

    from rustconv import profile_derive as pd

    dp = SimpleNamespace(
        errors=[
            pd.Finding("error", "SWITCH_CONST_IN_BASE_HEADER", "`STD_INPUT_HANDLE` is owned"),
            pd.Finding("error", "SWITCH_CONST_IN_BASE_HEADER", "`STATUS_X` is owned"),
        ]
    )
    assert port.switch_facts(dp) == ["STATUS_X", "STD_INPUT_HANDLE"]  # type: ignore[arg-type]


def test_a_second_port_run_on_one_module_is_refused(tmp_path, monkeypatch):
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path)
    first = port._hold("m")
    with pytest.raises(port.PortError, match="another `rustconv port` run holds m"):
        port._hold("m")
    first.close()
    port._hold("m").close()


def test_the_models_type_twin_gives_way_to_c2rusts(tmp_path):
    mods = tmp_path / "m"
    mods.mkdir()
    (mods / "registry.rs").write_text(
        "use crate::prelude::*;\n#[repr(C)]\npub struct HIVE_LIST_ENTRY {\n    pub a: u32,\n}\n"
        "pub type PHIVE_LIST_ENTRY = *mut HIVE_LIST_ENTRY;\nfn f() {}\n"
    )
    dropped = port.drop_model_type_twins(mods, {"HIVE_LIST_ENTRY", "_HIVE_LIST_ENTRY"})
    assert dropped == {"HIVE_LIST_ENTRY": ["registry.rs"]}
    text = (mods / "registry.rs").read_text()
    assert "struct HIVE_LIST_ENTRY" not in text and "pub type PHIVE_LIST_ENTRY" in text
    assert "fn f() {}" in text


def test_item_spans_find_types_statics_and_macros():
    text = (
        'pub type POPEN = Option<unsafe extern "system" fn(a: u32)>;\n'
        "#[macro_export]\nmacro_rules! DoesFileExist {\n    ($a:expr) => { f($a, TRUE) };\n}\n"
        "#[repr(C)]\npub struct S {\n    pub a: u32,\n}\n"
        "static mut T: [u32; 2] = [1, 2];\nfn g() {}\n"
    )
    for name, starts, ends in (
        ("POPEN", "pub type POPEN", "fn(a: u32)>;"),
        ("DoesFileExist", "#[macro_export]", "TRUE) };\n}"),
        ("S", "#[repr(C)]", "pub a: u32,\n}"),
        ("T", "static mut T", "[1, 2];"),
        ("g", "fn g", "fn g() {}"),
    ):
        ((lo, hi),) = port.item_spans(text, name)
        assert text[lo:hi].startswith(starts) and text[lo:hi].endswith(ends), name


def test_a_macro_name_inside_a_string_is_not_a_call():
    from rustconv.crate_integrate import fix_macro_calls

    src = (
        "fn f() {\n    if SpInfFindFirstLine(a, b) == 0 {\n"
        '        DPRINT1(b"SpInfFindFirstLine(\\"x\\") failed\\n\\0");\n'
        "    } // SpInfFindFirstLine(old)\n}\n"
    )
    out, fixed = fix_macro_calls(src, {"SpInfFindFirstLine", "DPRINT1"}, set())
    assert "if SpInfFindFirstLine!(a, b)" in out and "DPRINT1!(" in out
    assert 'b"SpInfFindFirstLine(\\"x\\")' in out and "// SpInfFindFirstLine(old)" in out


def test_seeded_switch_facts_already_in_the_prelude_pass(tmp_path, monkeypatch):
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("")
    monkeypatch.setattr(port, "_resolve_facts", lambda state, p, names: names)
    monkeypatch.setattr(
        port.prelude_resolve, "existing_facts", lambda p: {"STD_INPUT_HANDLE": ("DWORD", 1, "x")}
    )
    port._seed_switch_facts({"switch_facts": ["STD_INPUT_HANDLE"]}, prelude)
    with pytest.raises(port.PortError):
        port._seed_switch_facts({"switch_facts": ["STD_INPUT_HANDLE", "MISSING_X"]}, prelude)


def test_an_initguid_tu_is_lowered_from_its_header(tmp_path, monkeypatch):
    monkeypatch.setattr(port, "REPO", tmp_path)
    inc = tmp_path / "sdk" / "include" / "psdk"
    inc.mkdir(parents=True)
    (inc / "diskguid.h").write_text(
        "DEFINE_GUID(PARTITION_SYSTEM_GUID, 0xC12A7328, 0xF81F, 0x11D2, 0xBA, 0x4B, 0x00, "
        "0xA0, 0xC9, 0x3E, 0xC9, 0x3B);    // EFI\n"
    )
    tu = tmp_path / "guid.c"
    tu.write_text("/* x */\n#define INITGUID\n#include <guiddef.h>\n#include <diskguid.h>\n")
    out = port.initguid_module(tu)
    assert out is not None and "Data1: 0xC12A7328" in out and "0xC9, 0x3B]" in out
    tu.write_text("#define INITGUID\n#include <diskguid.h>\nint x;\n")
    assert port.initguid_module(tu) is None, "real code goes to translate"


def test_split_top_keeps_fn_pointer_types_whole():
    got = port._split_top(
        'a: u32, cb: Option<unsafe extern "C" fn(x: i32, y: u8) -> u32>, z: *mut u16'
    )
    assert got == [
        "a: u32",
        'cb: Option<unsafe extern "C" fn(x: i32, y: u8) -> u32>',
        "z: *mut u16",
    ]


def test_flip_comments_each_tu_and_links_the_archive(tmp_path, monkeypatch):
    monkeypatch.setattr(port, "REPO", tmp_path)
    (tmp_path / "m").mkdir()
    state = {
        "stem": "m",
        "crate": "m-rust",
        "target": "m",
        "module_dir": "m",
        "tus": ["m/a.c", "m/sub/b.c"],
    }
    cml = "list(APPEND SOURCE\n    a.c\n    sub/b.c\n    m.h)\n\nadd_executable(m ${SOURCE} m.rc)\nset_module_type(m win32cui)\n"
    out = port.flip_cmake(cml, state)
    assert "    # a.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/a.rs)" in out
    assert "    # sub/b.c  # replaced by Rust (sdk/lib/rust/m-rust/src/m/b.rs)" in out
    assert "    rust_cohab_stub.c\n    m.h)" in out
    assert 'rust_staticlib_variant(m_rust m-rust "" TRUE)\nadd_executable(m' in out
    assert "m.rc)\nadd_dependencies(m rust_m_rust)\ntarget_link_libraries(m ${m_rust_LIB})\n" in out
    assert port.flip_cmake(out, state) == out, "idempotent"
    import pytest as _pytest

    with _pytest.raises(port.PortError):
        port.flip_cmake(cml.replace("    a.c\n", ""), state)


def test_an_anonymous_type_is_matched_by_shape_in_its_own_tu(tmp_path, monkeypatch):
    # via.c's bridge-table struct is C2RustUnnamed_154/_162/_163 on i386/amd64/arm64, and
    # amd64's _163 is a different struct: the number identifies nothing, the fields do.
    from rustconv import c2rust_anchor

    def item(n, fields):
        body = "".join(f"    pub {f}: {ty},\n" for f, ty in fields)
        return f"#[derive(Copy, Clone)]\n#[repr(C)]\npub struct {n} {{\n{body}}}\n"

    bridge = [("DeviceID", "USHORT"), ("MinimumRevisionID", "UCHAR"), ("Flags", "UCHAR")]
    other = [("DeviceID", "USHORT"), ("Flags", "USHORT")]
    texts = {
        "i386": item("C2RustUnnamed_154", bridge) + item("C2RustUnnamed_155", other),
        "amd64": item("C2RustUnnamed_162", bridge) + item("C2RustUnnamed_163", other),
        "arm64": item("C2RustUnnamed_163", bridge) + item("C2RustUnnamed_164", other),
    }
    monkeypatch.setattr(c2rust_anchor, "_native_text", lambda f, arch: texts[arch])
    code = (
        "static mut L: [C2RustUnnamed_163; 1] = [C2RustUnnamed_163 {\n"
        "    DeviceID: 1 as USHORT, MinimumRevisionID: f(0, 1), Flags: 3 as UCHAR,\n}];\n"
    )
    fields = port._literal_fields(code, "C2RustUnnamed_163")
    assert fields == {"DeviceID", "MinimumRevisionID", "Flags"}
    agreed, differs = c2rust_anchor.anonymous_types(
        tmp_path / "via.c", {"C2RustUnnamed_163": fields}
    )
    assert differs == {}
    assert "pub struct C2RustUnnamed_163 {" in agreed["C2RustUnnamed_163"]
    assert "MinimumRevisionID: UCHAR" in agreed["C2RustUnnamed_163"]
    # No struct of that shape on one arch: refused, never guessed.
    texts["arm64"] = item("C2RustUnnamed_164", other)
    agreed, differs = c2rust_anchor.anonymous_types(
        tmp_path / "via.c", {"C2RustUnnamed_163": fields}
    )
    assert agreed == {} and "C2RustUnnamed_163" in differs


def test_an_invented_const_for_a_function_like_macro_is_dropped(tmp_path):
    from rustconv.local_macros import CMacro

    mods = tmp_path / "m"
    mods.mkdir()
    (mods / "intel.rs").write_text(
        "pub const PIIX_REG_UDMATIM: u8 = 0x4A;\npub const KEEP: u8 = 1;\n"
        "fn f() { PciRead8(c, PIIX_REG_UDMATIM(Channel)); }\n"
    )
    defs = {
        "PIIX_REG_UDMATIM": CMacro(
            "PIIX_REG_UDMATIM", ("Channel",), "(0x4A + (Channel))", "intel.c"
        ),
        "KEEP": CMacro("KEEP", None, "1", "intel.c"),
    }
    assert port._drop_invented_macro_consts(mods, defs) == {"PIIX_REG_UDMATIM"}
    text = (mods / "intel.rs").read_text()
    assert "const PIIX_REG_UDMATIM" not in text and "const KEEP" in text


def test_definition_signatures_read_a_static_function_from_its_own_definition(monkeypatch):
    # PataEnableInterrupts is `static`: no header declares it, so only its definition can
    # give the fn-pointer slot its exact type (`Enable: BOOLEAN`, not the model's `i32`).
    from rustconv import c2rust_anchor

    text = (
        'unsafe extern "C" fn PataEnableInterrupts(\n    mut ChannelContext: PVOID,\n'
        "    mut Enable: BOOLEAN,\n) {\n}\n"
        'pub unsafe extern "C" fn PataStartIo(mut c: PVOID, mut r: PATA_DEVICE_REQUEST)'
        " -> BOOLEAN {\n    0\n}\n"
    )
    monkeypatch.setattr(c2rust_anchor, "_native_text", lambda f, arch: text)
    sigs = c2rust_anchor.definition_signatures(
        Path("pata.c"), {"PataEnableInterrupts", "PataStartIo"}
    )
    assert sigs["PataEnableInterrupts"] == (
        "fn PataEnableInterrupts(ChannelContext: PVOID, Enable: BOOLEAN);"
    )
    assert sigs["PataStartIo"] == "fn PataStartIo(c: PVOID, r: PATA_DEVICE_REQUEST) -> BOOLEAN;"


def test_a_constant_used_only_as_a_match_pattern_is_found(tmp_path):
    # `BusQueryDeviceID =>` and atapi's `IOCTL_SCSI_RESCAN_BUS =>` were bindings that matched
    # everything: a name used only as a pattern is never "unresolved", so resolve never saw it.
    src = tmp_path / "src"
    mods = src / "m"
    mods.mkdir(parents=True)
    (src / "prelude.rs").write_text("pub const IOCTL_KNOWN: u32 = 1;\n")
    (mods / "a.rs").write_text(
        "fn f(c: u32) {\n    match c {\n        IOCTL_KNOWN => {}\n"
        "        IOCTL_A | IOCTL_B => {}\n        BusQueryDeviceID => {}\n"
        "        _ => {}\n    }\n}\n"
    )
    found = port._pattern_idents(mods)
    assert found == {"IOCTL_KNOWN", "IOCTL_A", "IOCTL_B", "BusQueryDeviceID"}
    assert port._undefined_pattern_constants(src, mods, None) == {"IOCTL_A", "IOCTL_B"}


def test_an_arch_comparison_never_counts_a_fallback_transpile(monkeypatch):
    # notepad's main.c fails c2rust on i386 and anchored from build-amd64; extern_signatures
    # then compared amd64 with itself and declared `wcslen -> c_ulonglong` for i386.
    from rustconv import c2rust_anchor

    sig = "    pub fn wcslen(_Str: *const wchar_t) -> ::core::ffi::c_ulonglong;\n"
    narrow = "    pub fn wcsncpy(_Dest: *mut wchar_t, _Source: *const wchar_t, _Count: size_t) -> *mut wchar_t;\n"
    texts = {
        "i386": None,
        "amd64": f'extern "C" {{\n{sig}{narrow}}}\n',
        "arm64": f'extern "C" {{\n{sig}{narrow}}}\n',
    }
    monkeypatch.setattr(c2rust_anchor, "_native_text", lambda f, arch: texts[arch])
    got = c2rust_anchor.extern_signatures(Path("main.c"), {"wcslen", "wcsncpy"})
    assert "wcslen" not in got  # a 64-bit width with i386 unheard is not agreement
    assert "wcsncpy" in got


def test_a_definition_spelled_with_an_sdk_alias_exports_the_real_name(tmp_path, monkeypatch):
    # notepad defines `_tWinMain`; tchar.h makes it `wWinMain`, which the CRT startup calls.
    from rustconv import macro_authority

    rs = tmp_path / "main.rs"
    rs.write_text(
        '#[no_mangle]\npub unsafe extern "system" fn _tWinMain(h: HINSTANCE) -> i32 {\n    0\n}\n'
        '#[no_mangle]\npub unsafe extern "C" fn Helper() {}\n'
    )
    monkeypatch.setattr(
        macro_authority, "identifier_aliases", lambda _r, _c: {"_tWinMain": "wWinMain"}
    )
    assert port._aw_export_names(rs, tmp_path / "main.c") == ["_tWinMain -> wWinMain"]
    out = rs.read_text()
    assert '#[export_name = "wWinMain"]\npub unsafe extern "system" fn _tWinMain(' in out
    assert '#[no_mangle]\npub unsafe extern "C" fn Helper()' in out


def test_crt_data_is_read_through_its_accessor(tmp_path):
    # clipbrd/mplay32: `__argc`/`__wargv` are dllimport data, reachable only via the IAT slot.
    rs = tmp_path / "main.rs"
    rs.write_text(
        "fn f() {\n    let n = __argc;\n    let a = *__wargv.offset(1);\n    __p___argc();\n}\n"
    )
    assert port._crt_data_globals(rs) == ["__argc x1", "__wargv x1"]
    out = rs.read_text()
    assert "let n = (*__p___argc());" in out and "*(*__p___wargv()).offset(1)" in out
    assert "    __p___argc();" in out  # an existing accessor call is untouched
    local = tmp_path / "local.rs"
    local.write_text("fn f(__argc: i32) -> i32 { __argc }\n")
    assert port._crt_data_globals(local) == []


def test_trace_on_names_the_channel_static(tmp_path):
    # TRACE_ON(ch) pastes `&__wine_dbch_##ch`, which macro_rules cannot.
    rs = tmp_path / "x.rs"
    rs.write_text("fn f() { if TRACE_ON(extrac32) != 0 {} if WINE_WARN_ON(foo) != 0 {} }\n")
    port._wine_debug_storage(rs)
    out = rs.read_text()
    assert "TRACE_ON!(__wine_dbch_extrac32)" in out and "WARN_ON!(__wine_dbch_foo)" in out


def test_stdio_streams_are_the_iob_entries(tmp_path):
    rs = tmp_path / "f.rs"
    rs.write_text("fn f() { fprintf(stderr, m); fgetws(b, 9, stdin); }\n")
    assert port._crt_data_globals(rs) == ["stdin x1", "stderr x1"]
    assert "fprintf(__iob_func().add(2), m)" in rs.read_text()
    assert "fgetws(b, 9, __iob_func().add(0))" in rs.read_text()


def test_an_all_zero_array_field_takes_its_length_from_the_type(tmp_path):
    # mplay32: c2rust froze amd64's 6-byte TBBUTTON.bReserved; i386's is 2.
    rs = tmp_path / "t.rs"
    rs.write_text(
        "static B: [T; 1] = [T { a: 1, bReserved: [0 as ::core::ffi::c_int as BYTE, 0, 0, 0, 0, 0], c: [1, 0] }];\n"
    )
    assert port._zero_array_literals(rs) == ["bReserved"]
    out = rs.read_text()
    assert "bReserved: unsafe { ::core::mem::zeroed() }" in out and "c: [1, 0]" in out


def test_c2rust_prelude_field_renames_are_undone(tmp_path):
    rs = tmp_path / "e.rs"
    rs.write_text("fn f(d: *mut D) { if (*d).Option_0 == 1 { Option_0(); } }\n")
    assert port._c2rust_prelude_fields(rs) == 1
    assert "(*d).Option == 1 { Option_0(); }" in rs.read_text()


def test_long_double_is_f64_on_every_reactos_target(tmp_path):
    # calc: c2rust's c_longdouble, an invented placeholder and an f128 crate; -mlong-double-64.
    rs = tmp_path / "r.rs"
    rs.write_text(
        "fn f() { a = (x & y) as core::ffi::c_longdouble; b = z as core::ffi::c_longdouble_placeholder;\n"
        "  c = ::f128::f128::new(!n.i).to_f64().unwrap(); }\n"
    )
    assert port._long_double_f64(rs) == 3
    out = rs.read_text()
    assert "(x & y) as f64" in out and "z as f64" in out and "((!n.i) as f64)" in out


def test_c2rust_anonymous_members_take_bindgen_names(tmp_path):
    island = (
        "pub struct _NID {\n    pub cbSize: u32,\n    pub __bindgen_anon_1: _NID__bindgen_ty_1,\n}\n"
        "pub union _NID__bindgen_ty_1 {\n    pub uTimeout: u32,\n    pub uVersion: u32,\n}\n"
    )
    rs = tmp_path / "m.rs"
    rs.write_text(
        "static mut nid: NID = _NID { cbSize: 0, c2rust_unnamed: C2RustUnnamed { uTimeout: 0 }, };\n"
        "fn f() { let a = si.c2rust_unnamed.c2rust_unnamed.wProcessorArchitecture; }\n"
    )
    port._c2rust_unnamed(rs, island)
    out = rs.read_text()
    assert "__bindgen_anon_1: _NID__bindgen_ty_1 { uTimeout: 0 }" in out
    assert "let a = si.wProcessorArchitecture;" in out


def test_an_anonymous_type_no_arch_finds_is_not_agreed(tmp_path, monkeypatch):
    # mmc: every arch found 0 matches, the miss was recorded as the string "0 candidates",
    # the set held one string -- "agreed" -- and it was written into mscfile.rs.
    from rustconv import c2rust_anchor

    monkeypatch.setattr(
        c2rust_anchor, "_native_text", lambda f, arch: "pub struct Other { pub a: i32 }\n"
    )
    agreed, differs = c2rust_anchor.anonymous_types(
        tmp_path / "x.c", {"C2RustUnnamed_7": frozenset({"x", "y"})}
    )
    assert agreed == {} and "C2RustUnnamed_7" in differs


def test_a_rust_str_passed_to_c_gets_its_nul(tmp_path):
    # magnify: "1".as_ptr() to CB_ADDSTRING read on past the literal.
    rs = tmp_path / "m.rs"
    rs.write_text(
        'fn f() { g("1".as_ptr()); MessageBox(h, TEXT!("Help").as_ptr() as LPCSTR); '
        'k(b"ok\\0".as_ptr()); j("done\\0".as_ptr()); }\n'
    )
    assert port._c_str_literals(rs) == ["1", "Help"]
    out = rs.read_text()
    assert 'g(b"1\\0".as_ptr())' in out and 'MessageBox(h, b"Help\\0".as_ptr() as LPCSTR)' in out
    assert 'k(b"ok\\0".as_ptr())' in out and 'j("done\\0".as_ptr())' in out


def test_a_derive_mode_override_is_remembered_for_later_stages(tmp_path, monkeypatch):
    # winegdi links into gdi32 from another CMakeLists, so derive cannot infer its mode;
    # the --usermode given once must reach every later re-derive without being repeated.
    monkeypatch.setattr(port, "REPO", tmp_path)
    monkeypatch.setattr(port, "STATE_ROOT", tmp_path / "state")
    mod = tmp_path / "win32ss" / "gdi" / "gdi32" / "wine"
    mod.mkdir(parents=True)
    for stem, target, extra in (
        ("winegdi", "winegdi", {"mode_override": True}),
        ("other", "other", {"usermode": False}),
    ):
        (tmp_path / "state" / stem).mkdir(parents=True)
        state = {"stem": stem, "module_dir": "win32ss/gdi/gdi32/wine", "target": target}
        (tmp_path / "state" / stem / "state.json").write_text(json.dumps({**state, **extra}))
    assert port.recorded_mode(mod, "winegdi") is True
    assert port.recorded_mode(mod, "other") is None  # an inferred mode is not an override
    assert port.recorded_mode(mod, None) is True  # the CLI omits --target for a lone target


def test_token_pasting_macros_and_their_callers_are_hand_ports():
    from types import SimpleNamespace as M

    macros = {
        "CS_StrA": M(body="((LPCSTR)(pCS) + (pCS)->dw##Name##Offset)"),
        "CS_StrW": M(body="((LPCWSTR)CS_StrA(pCS, Name))"),
        "CS_Wrap": M(body="CS_StrW(p, n)"),
        "ALIGN_DWORD": M(body="(((len) + 3) & ~3)"),
        # cmd's console.h: GNU's comma swallow, not pasting.
        "ConOutPrintf": M(body="ConPrintf(StdOut, (szStr), ##__VA_ARGS__)"),
    }
    kept, pasting = port._without_token_pasting(macros)
    assert list(kept) == ["ALIGN_DWORD", "ConOutPrintf"]
    assert pasting == ["CS_StrA", "CS_StrW", "CS_Wrap"]


def test_each_source_list_gets_its_own_stub():
    lines = [
        "list(APPEND VISTA_SOURCE",
        "     # wine/win.c  # replaced by Rust (x.rs)",
        "     )",
        "list(APPEND SOURCE",
        "    # dpi.c  # replaced by Rust (y.rs)",
        "    rust_cohab_stub.c",
        "    )",
    ]
    assert not port._list_has_stub(lines, 1)
    assert port._list_has_stub(lines, 4)


def test_flip_links_with_the_targets_own_link_signature():
    state = {
        "stem": "winegdi",
        "crate": "winegdi-rust",
        "target": "winegdi",
        "module_dir": "win32ss/gdi/gdi32/wine",
        "tus": [],
    }
    cml = "add_library(winegdi STATIC ${SOURCE})\ntarget_link_libraries(winegdi PRIVATE pseh)\n"
    out = port.flip_cmake(cml, state)
    assert "target_link_libraries(winegdi PRIVATE ${winegdi_rust_LIB})" in out


def test_a_copy_naming_its_source_crates_private_type_is_refused(tmp_path, monkeypatch):
    # regedit got ucrt-rust's `qsort(.., comp: Compare)`; `Compare` is ucrt's own alias.
    rust = tmp_path / "sdk" / "lib" / "rust"
    (rust / "ucrt-rust" / "src").mkdir(parents=True)
    (rust / "ucrt-rust" / "src" / "q.rs").write_text("type Compare = Option<fn()>;\n")
    monkeypatch.setattr(port, "REPO", tmp_path)
    from rustconv.prelude_resolve import Item

    priv = Item("fn", "pub fn qsort(b: *mut c_void, n: usize, comp: Compare);", None, "ucrt-rust/src/q.rs")
    ok = Item("fn", "pub fn qsort(b: *mut c_void, n: usize, comp: PFN);", None, "ucrt-rust/src/q.rs")
    assert port._private_types(priv, {"PFN"}) == {"Compare"}
    assert port._private_types(ok, {"PFN"}) == set()
    rel = Item("fn", "pub fn qsort(b: *mut c_void, cmp: super::super::CntCmpFn);", None, "cnt-rust/src/g.rs")
    assert port._private_types(rel, set()) == {"super::super::CntCmpFn"}


def test_a_negative_fact_of_an_unsigned_type_is_its_bit_pattern():
    # calc: clang evaluated _UI64_MAX to -1; `pub const _UI64_MAX: u64 = -1;` is E0600.
    from rustconv.prelude_resolve import existing_facts, render_facts

    out = render_facts({"_UI64_MAX": ("u64", -1, "fun_ieee.c")})
    assert "pub const _UI64_MAX: u64 = 0xFFFFFFFFFFFFFFFF; // clang, fun_ieee.c" in out


def test_a_const_rewriting_an_sdk_macro_from_memory_is_dropped(tmp_path, monkeypatch):
    # regedit's regproc.rs: `HKEY_CLASSES_ROOT = 0x80000000usize as HKEY` zero-extends what
    # winreg.h sign-extends. The module's own #define (MAX_X) is not the header's.
    from rustconv import sdk_macros

    rs, c = tmp_path / "regproc.rs", tmp_path / "regproc.c"
    rs.write_text(
        "pub const HKEY_CLASSES_ROOT: HKEY = 0x80000000usize as HKEY;\n"
        "pub const MAX_X: u32 = 4;\nconst LOCAL: u32 = 1;\nfn f() {}\n"
    )
    c.write_text("#define MAX_X 4\n")
    monkeypatch.setattr(
        sdk_macros,
        "object_expansions",
        lambda repo, src, names: {n: "x" for n in names if n.startswith("HKEY_") or n == "MAX_X"},
    )
    assert port._drop_sdk_macro_shadows(rs, c) == ["HKEY_CLASSES_ROOT"]
    out = rs.read_text()
    assert "HKEY_CLASSES_ROOT" not in out and "MAX_X" in out and "LOCAL" in out


def test_reactos_gates_are_settled_to_the_live_arm(tmp_path):
    # regedit's regproc.rs: both arms, gated on things Rust never sets -> Wine's arm compiled.
    rs = tmp_path / "regproc.rs"
    rs.write_text(
        "fn a() {\n    if cfg!(feature = \"reactos\") {\n        live1();\n    } else {\n        wine1(x, 1);\n    }\n}\n"
        "fn b() {\n    #[cfg(feature = \"reactos\")]\n    live2();\n    #[cfg(not(feature = \"reactos\"))]\n    wine2();\n}\n"
        "fn c() {\n    const REACTOS: bool = false;\n    if REACTOS {\n        live3();\n    } else {\n        wine3();\n    }\n}\n"
        "fn d() {\n    #[cfg(not(reactos))]\n    {\n        wine4();\n    }\n    if true { keep(); }\n}\n"
    )
    assert port._reactos_gates(rs) > 0
    out = rs.read_text()
    for live in ("live1()", "live2()", "live3()", "keep()"):
        assert live in out
    assert "wine1" not in out and "wine3" not in out
    assert "#[cfg(any())]\n    wine2();" in out and "#[cfg(any())]\n    {\n        wine4();" in out
    assert "feature = \"reactos\"" not in out


def test_file_and_line_are_rusts_own(tmp_path):
    # regedit: `pub const __LINE__: i32 = 7` came from the clang facts probe.
    rs = tmp_path / "s.rs"
    rs.write_text('fn f() { DbgPrint(b"%s:%d\\0".as_ptr(), __FILE__, __LINE__); } // __FILE__\n')
    assert port._builtin_location_macros(rs) == 3
    out = rs.read_text()
    assert 'DbgPrint(b"%s:%d\\0".as_ptr(), concat!(file!(), "\\0").as_ptr(), (line!() as ::core::ffi::c_int))' in out
    assert "__LINE__" in port._COMPILER_BUILTIN_MACROS


def test_the_iob_array_is_reached_through_its_accessor(tmp_path):
    # regedit: `(&raw mut _iob as *mut FILE).offset(1)` -> undefined `_iob` at link.
    rs = tmp_path / "r.rs"
    rs.write_text("fn f() { file = (&raw mut _iob as *mut FILE).offset(1) as *mut FILE; }\n")
    assert port._crt_data_globals(rs) == ["_iob x1"]
    assert "(__iob_func() as *mut FILE).offset(1)" in rs.read_text()


def test_a_static_the_c_only_declares_extern_is_not_a_second_definition(tmp_path):
    # regedit's framewnd.c: `extern WCHAR Suggestions[256];` -- childwnd.c defines it.
    rs, c = tmp_path / "framewnd.rs", tmp_path / "framewnd.c"
    rs.write_text(
        "pub static mut Suggestions: [WCHAR; 256] = [0; 256];\n"
        "static mut hMenuFrame: HMENU = core::ptr::null_mut();\nfn f() {}\n"
    )
    c.write_text("extern WCHAR Suggestions[256];\nHMENU hMenuFrame;\n")
    assert port._extern_only_statics(rs, c) == ["Suggestions"]
    assert "Suggestions" not in rs.read_text() and "hMenuFrame" in rs.read_text()


def test_a_stale_import_of_a_variadic_entry_is_dropped(tmp_path):
    # cmd: dir.rs had `use crate::cmd::console::ConOutPrintfPaging;` (E0603 once console.rs's
    # extern block moved to the prelude); an ordinary sibling import stays.
    rs = tmp_path / "dir.rs"
    rs.write_text(
        "use crate::cmd::console::ConOutPrintfPaging;\n"
        "use crate::cmd::console::ConOutResPaging;\n"
        "use crate::prelude::*;\n"
    )
    port._drop_stale_variadic_imports([rs], {"ConOutPrintfPaging", "ErrorMessage"})
    assert rs.read_text() == "use crate::cmd::console::ConOutResPaging;\nuse crate::prelude::*;\n"


def test_a_derived_arm_keeps_the_old_arms_hand_section():
    old = (
        "case \"$DRIVER\" in\n"
        "  cmd)\n    BLOCKLIST_FUNCS=0\n"
        "    # >>> hand\n    EXTRA_BL_ITEMS=( _CON_STREAM )\n    # <<< hand\n"
        "    ;;\n"
        '  *) echo "unknown driver"\n'
    )
    new_arm = "  cmd)\n    BLOCKLIST_FUNCS=1\n    ;;\n"
    out = port.upsert_profile_arm(old, new_arm, "cmd")
    assert "BLOCKLIST_FUNCS=1\n    # >>> hand\n    EXTRA_BL_ITEMS=( _CON_STREAM )\n" in out
    assert "BLOCKLIST_FUNCS=0" not in out and out.count("# >>> hand") == 1
    assert port.upsert_profile_arm(out, new_arm, "cmd") == out  # stable on re-derive


def test_gen_arm_write_is_locked_and_atomic(tmp_path, monkeypatch):
    # Three integration agents re-derived concurrently; a running bash read a half-rewritten
    # gen_driver_types.sh. The writer re-reads under a lock and swaps the inode.
    gen = tmp_path / "gen_driver_types.sh"
    gen.write_text('case "$1" in\n  a)\n    X=1\n    ;;\n  *) echo "unknown driver $1"; exit 1 ;;\nesac\n')
    monkeypatch.setattr(port, "GEN", gen)
    ino = gen.stat().st_ino
    port._write_gen_arm("  b)\n    Y=2\n    ;;\n", "b")
    text = gen.read_text()
    assert "  a)\n    X=1" in text and "  b)\n    Y=2" in text
    assert gen.stat().st_ino != ino  # replaced, not rewritten in place
    port._write_gen_arm("  b)\n    Y=2\n    ;;\n", "b")  # idempotent
    assert gen.read_text() == text


def test_conutils_streams_are_values_not_places(tmp_path):
    rs = tmp_path / "n.rs"
    rs.write_text(
        "fn f() { ConPuts((addr_of_mut!(StdOut)).cast(), s); ConPrintf(&raw mut StdErr, f); "
        "ConPuts(StdOut, s); let csStdOut_p = addr_of_mut!(csStdOut); }\n"
    )
    assert port._con_stream_values(rs) == 2
    out = rs.read_text()
    assert "ConPuts(StdOut, s); ConPrintf(StdErr, f); ConPuts(StdOut, s);" in out
    assert "addr_of_mut!(csStdOut)" in out


def test_an_ansi_t_literal_gets_its_nul_too(tmp_path):
    # arp (ANSI): `_T!("%s").as_ptr()` -- printf read on through rodata.
    rs = tmp_path / "arp.rs"
    rs.write_text('fn f() { printf(_T!("%s").as_ptr(), s); sprintf(b, _T!(" %02X").as_ptr(), c); }\n')
    assert port._c_str_literals(rs) == ["%s", " %02X"]
    assert 'printf(b"%s\\0".as_ptr(), s); sprintf(b, b" %02X\\0".as_ptr(), c);' in rs.read_text()


def test_a_const_array_whose_pointer_escapes_becomes_static(tmp_path):
    # attrib: `pszFileName = (ALL_FILES_PATTERN).as_mut_ptr();` -- a dangling temporary.
    rs = tmp_path / "attrib.rs"
    rs.write_text(
        "pub const ALL_FILES_PATTERN: [u16; 4] = [42, 46, 42, 0];\n"
        "const RO: [u16; 2] = [1, 0];\nconst SIZES: [u32; 2] = [1, 2];\n"
        "fn f() { p = (ALL_FILES_PATTERN).as_mut_ptr(); q = RO.as_ptr(); n = SIZES[1]; }\n"
    )
    assert port._pointed_const_arrays(rs) == ["ALL_FILES_PATTERN", "RO"]
    out = rs.read_text()
    assert "pub static mut ALL_FILES_PATTERN: [u16; 4]" in out and "static RO: [u16; 2]" in out
    assert "const SIZES: [u32; 2]" in out


def test_an_extern_iob_declaration_goes_with_its_uses(tmp_path):
    # finger's net.rs: `extern { static mut _iob: FILE; }` + addr_of_mut!(_iob).offset(2).
    rs = tmp_path / "net.rs"
    rs.write_text(
        'extern "C" {\n    static mut _iob: FILE;\n}\n'
        "fn f() { fprintf((core::ptr::addr_of_mut!(_iob)).offset(2) as *mut FILE, s); }\n"
    )
    assert port._crt_data_globals(rs) == ["_iob x1"]
    out = rs.read_text()
    assert "static mut _iob" not in out and "fprintf((__iob_func()).offset(2) as *mut FILE, s)" in out
