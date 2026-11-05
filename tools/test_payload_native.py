"""Native staging contracts: host files, vendor encodings and shared specs."""

import importlib.util
import json
from pathlib import Path

import pytest

from tools.build_infra_compat import NativeError, call

ROOT = Path(__file__).resolve().parents[1]


def test_payload_specs_match_the_frozen_python_loader(tmp_path):
    oracles = json.loads((ROOT / "tools/vibeos-build/tests/payload/spec-oracles.json").read_text())
    for name, fixture in oracles.items():
        directory = tmp_path / "tools/payloads" / name
        directory.mkdir(parents=True)
        (directory / "payload.yaml").write_text(fixture["yaml"])
        native = call("payload_build_load_spec", {"name": name}, tmp_path)
        assert native == fixture["spec"], name


def test_host_application_globs_are_anchored_sorted_and_may_select_subdirectories(
    tmp_path, monkeypatch
):
    apps = tmp_path / "tools/payloads/hostapps"
    apps.mkdir(parents=True)
    (apps / "apps.txt").write_text("demo|Product|*.exe,lib/[ab]?.dll\n")
    product = tmp_path / "host/Product"
    (product / "lib").mkdir(parents=True)
    for name in ["z.exe", "a.exe", "lib/a1.dll", "lib/b2.dll", "lib/c1.dll", "lib/hidden.exe"]:
        (product / name).write_bytes(b"fixture")
    monkeypatch.setenv("VIBEOS_HOST_APPS_ROOT", str(tmp_path / "host"))
    entries = call(
        "payload_providers_stage_entries", {"spec": "hostapps", "arch": "amd64"}, tmp_path
    )
    assert [entry["dest"] for entry in entries] == [
        "bin/demo/a.exe",
        "bin/demo/z.exe",
        "bin/demo/a1.dll",
        "bin/demo/b2.dll",
    ]
    (apps / "apps.txt").write_text("demo|Product|missing*.exe\n")
    with pytest.raises(NativeError, match="matched nothing"):
        call("payload_providers_stage_entries", {"spec": "hostapps", "arch": "amd64"}, tmp_path)


@pytest.mark.parametrize("encoding", ["latin-1", "utf-16", "utf-16-be"])
def test_vendor_preference_edits_a_cached_copy_and_keeps_the_package(
    tmp_path, monkeypatch, encoding
):
    package = tmp_path / "package"
    package.mkdir()
    text = "[Version]\nDriverVer=01/01/2020,1.2\n[Models]\n%Device%=Install,,PCI\\VEN_1234\n"
    data = text.encode(encoding)
    if encoding == "utf-16-be":
        data = b"\xfe\xff" + data
    inf = package / "Device.INF"
    inf.write_bytes(data)
    (package / "Driver.SYS").write_bytes(b"driver")
    (package / "ndisprobe.txt").write_text("probe")
    monkeypatch.setenv("VIBEOS_VENDOR_DRIVERS", str(package))
    monkeypatch.setenv("VIBEOS_VENDOR_PREFER", "1")
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "cache"))
    monkeypatch.delenv("VIBEOS_WIFI_CONFIG", raising=False)
    entries = call("payload_providers_stage_entries", {"spec": "vendordrv", "arch": "arm64"}, ROOT)
    paths = {entry["dest"]: Path(entry["src"]) for entry in entries}
    edited = paths["inf/device.inf"]
    assert edited != inf and inf.read_bytes() == data
    result = edited.read_bytes().decode("utf-16" if encoding.startswith("utf-16") else encoding)
    assert "12/31/2099" in result
    assert "%Device%=Install,PCI\\VEN_1234" in result
    assert paths["system32/drivers/driver.sys"] == package / "Driver.SYS"
    assert paths["bin/ndisprobe.txt"] == package / "ndisprobe.txt"


def test_vendor_staging_requires_an_inf_and_no_optional_ninja_provider(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_VENDOR_DRIVERS", str(tmp_path))
    with pytest.raises(NativeError, match=r"no \.inf"):
        call("payload_providers_stage_entries", {"spec": "vendordrv", "arch": "arm64"}, ROOT)
    # This is the actual compiler API, not a stubbed provider implementation.
    spec = tmp_path / "tools/payloads/driver"
    spec.mkdir(parents=True)
    (spec / "payload.yaml").write_text(
        "name: driver\nbinaries: []\nbinaries_from: tools/payloads/vendordrv/stage.py\n"
    )
    assert (
        call("payload_build_ninja_targets", {"names": ["driver"], "arch": "arm64"}, tmp_path) == []
    )


def test_saved_invalid_utf8_serial_reaches_the_native_modrun_grader(tmp_path):
    path = ROOT / "tools/payloads/modrun/modrun.py"
    spec = importlib.util.spec_from_file_location("payload_modrun_wire", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    import sys

    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    text = "good"
    checksum = module.checksum(text, 0)
    log = (
        f"[MODRUN:m.x:c:0:{checksum:04x}] good\n".encode()
        + b"noise\xff\n[MODRUN] exit m.x c rc=0x00000000 ms=1 records=1\n"
    )
    out, _ = module.parse(log.decode("utf-8", errors="surrogateescape"))
    assert out[("m.x", "c")] == ["good"]
