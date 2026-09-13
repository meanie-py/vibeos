from types import SimpleNamespace

import pytest

from rustconv import __main__ as cli
from rustconv import response_cache
from rustconv.dead_regions import never_compiled_lines
from rustconv.scanner import scan_file


@pytest.mark.parametrize("command", [cli.cmd_submit, cli.cmd_batch])
def test_legacy_paid_entries_require_explicit_opt_out(command):
    with pytest.raises(ValueError, match="unsafe-unprotected"):
        command(SimpleNamespace())


def test_changed_seal_namespace_does_not_rebuy(tmp_path, monkeypatch):
    monkeypatch.setattr(response_cache, "RESPONSES_DIR", tmp_path)
    old = tmp_path / "sysaudio__pin"
    old.mkdir()
    (old / "GetConnectRequest.rs").write_text("fn GetConnectRequest() {}")
    with pytest.raises(ValueError, match="previously purchased"):
        response_cache.require_no_namespace_rebuy("sysaudio__pin", "sysaudio__pin__factseal_v1")
    response_cache.require_no_namespace_rebuy("sysaudio__pin", "sysaudio__pin")
    assert (old / "GetConnectRequest.rs").read_text() == "fn GetConnectRequest() {}"


def test_changed_body_and_reject_are_not_fresh_purchases(tmp_path, monkeypatch):
    monkeypatch.setattr(response_cache, "RESPONSES_DIR", tmp_path)
    cache = response_cache.ResponseCache("pilot")
    cache.put("Kept", "old C body", "fn Kept() {}")
    assert cache.get("Kept", "new C body") is None
    with pytest.raises(ValueError, match="already has purchased"):
        cache.refuse_repurchase("Kept")
    cache.put_rejected("Rejected", "fn Rejected() {}", "audit failed")
    with pytest.raises(ValueError, match="already has purchased"):
        cache.refuse_repurchase("Rejected")
    cache.refuse_repurchase("New")


def test_dead_function_positions_use_the_scanner_source(tmp_path):
    # Resolving an in-body #if removes lines. Comparing transformed line numbers
    # with the original file used to drop the following LIVE function (SysAudio).
    path = tmp_path / "pin.c"
    path.write_text(
        "void First(void) {\n#if 0\n"
        + "x();\n" * 20
        + "#endif\n}\nvoid GetConnectRequest(void) {}\n"
    )
    scan = scan_file(path)
    dead = never_compiled_lines(scan.raw_source)
    names = {f.name for f in scan.functions if f.line_start not in dead}
    assert "GetConnectRequest" in names


def test_portclass_api_annotation_is_not_a_return_type(tmp_path):
    path = tmp_path / "resource.cpp"
    path.write_text("PORTCLASSAPI\nNTSTATUS\nNTAPI\nPcNewResourceList(void) { return 0; }")
    function = scan_file(path).functions[0]
    assert function.name == "PcNewResourceList"
    assert function.return_type == "NTSTATUS"
    assert function.calling_convention == "NTAPI"


def test_import_macro_cannot_rename_the_next_function(tmp_path):
    path = tmp_path / "mathf.c"
    path.write_text("__ASM_GLOBAL_IMPORT(sinf)\nfloat powf(float x) { return x; }")
    functions = scan_file(path).functions
    assert [(f.name, f.return_type) for f in functions] == [("powf", "float")]
