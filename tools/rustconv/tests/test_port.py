"""rustconv port: the hand-offs it automates are idempotent and land where they should."""

import json

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
        "pub type POPEN = Option<unsafe extern \"system\" fn(a: u32)>;\n"
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
