"""Shipping native strip/cache refusal controls, with a real producer subprocess."""

import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

import pytest

from tools import strip_cd_media as strip
from tools.build_infra_compat import NativeError

ROOT = Path(__file__).resolve().parents[1]


def image(magic=0x20B, long_name=False):
    data = bytearray(1024)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 128)
    data[128:132] = b"PE\0\0"
    struct.pack_into("<HHIIIHH", data, 132, 0x8664, 1, 0, 0, 0, 240, 0x2022)
    struct.pack_into("<H", data, 152, magic)
    data[392:400] = b".debug\0\0"
    if long_name:
        struct.pack_into("<I", data, 140, 900)
        struct.pack_into("<I", data, 900, 16)
        data[904:916] = b".debug_info\0\0"
        data[392:400] = b"/4\0\0\0\0\0\0"
    return data


def oracle_checksum(data, offset):
    data = bytearray(data)
    data[offset : offset + 4] = bytes(4)
    original_length = len(data)
    if len(data) % 2:
        data.append(0)
    total = sum(struct.unpack(f"<{len(data) // 2}H", data))
    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)
    return total + original_length


@pytest.fixture
def producer(tmp_path):
    tool = tmp_path / "objcopy"
    count = tmp_path / "calls"
    tool.write_text(
        f"#!{sys.executable}\n"
        "import sys\nfrom pathlib import Path\n"
        f"count=Path({str(count)!r})\n"
        "count.write_text(count.read_text()+'x' if count.exists() else 'x')\n"
        "data=Path(sys.argv[-2]).read_bytes().replace(b'.debug\\0\\0', b'.text\\0\\0\\0')\n"
        "Path(sys.argv[-1]).write_bytes(data)\n"
    )
    tool.chmod(0o755)
    cache = tmp_path / "cache"
    cache.mkdir()
    src = tmp_path / "original.sys"
    src.write_bytes(image())
    return src, cache, tool, count


@pytest.mark.parametrize("magic,long_name", [(0x10B, False), (0x20B, False), (0x20B, True)])
def test_pe_headers_and_coff_long_section_names(magic, long_name):
    offset, names = strip.parse_pe(image(magic, long_name))
    assert offset == 216
    assert names == ([".debug_info"] if long_name else [".debug"])
    assert strip.has_debug_sections(names)
    assert strip.parse_pe(b"not PE") is None
    with pytest.raises(NativeError, match="truncated"):
        strip.parse_pe(bytes(image())[:400])


@pytest.mark.parametrize("extra", [0, 1, 2, 3, 127])
def test_checksum_uses_actual_image_length_and_ignores_existing_field(extra):
    data = image() + bytes(range(extra))
    struct.pack_into("<I", data, 216, 0xDEADBEEF)
    expected = oracle_checksum(data, 216)
    assert strip.pe_checksum(data, 216) == expected
    assert data[216:220] == bytes(4)
    with pytest.raises(NativeError, match="outside"):
        strip.pe_checksum(data, len(data))


def test_twin_names_remain_fex_compatible_and_cache_checks_actual_bytes(producer):
    src, cache, tool, count = producer
    manifest = {}
    original = src.read_bytes()
    twin = strip.twin_for(src, cache, str(tool), manifest)
    assert twin == cache / (hashlib.sha1(str(src).encode()).hexdigest()[:12] + "-original.sys")
    assert src.read_bytes() == original
    assert twin is not None
    assert not strip.has_debug_sections(strip.parse_pe(twin.read_bytes())[1])
    assert struct.unpack_from("<I", twin.read_bytes(), 216)[0] == oracle_checksum(
        twin.read_bytes(), 216
    )
    assert strip.twin_for(src, cache, str(tool), manifest) == twin
    assert count.read_text() == "x"
    # Same size and mtime no longer hide a changed source.
    timestamp = src.stat().st_mtime_ns
    changed = bytearray(src.read_bytes())
    changed[-1] = 1
    src.write_bytes(changed)
    os.utime(src, ns=(timestamp, timestamp))
    strip.twin_for(src, cache, str(tool), manifest)
    assert count.read_text() == "xx" and twin.read_bytes()[-1] == 1
    twin.write_bytes(b"corrupt")
    strip.twin_for(src, cache, str(tool), manifest)
    assert count.read_text() == "xxx" and twin.read_bytes()[:2] == b"MZ"
    tool.write_text(tool.read_text() + "# new tool policy\n")
    strip.twin_for(src, cache, str(tool), manifest)
    assert count.read_text() == "xxxx"


@pytest.mark.parametrize(
    "body",
    [
        "raise SystemExit(7)",
        "pass",
        "Path(sys.argv[-1]).write_bytes(b'bad')",
        "Path(sys.argv[-1]).write_bytes(Path(sys.argv[-2]).read_bytes())",
    ],
)
def test_bad_producer_cannot_replace_completed_twin(producer, body):
    src, cache, tool, _count = producer
    dst = cache / "existing.sys"
    dst.write_bytes(b"previous complete output")
    tool.write_text(f"#!{sys.executable}\nimport sys\nfrom pathlib import Path\n{body}\n")
    with pytest.raises(NativeError):
        strip.strip_one(src, dst, str(tool))
    assert dst.read_bytes() == b"previous complete output"
    with pytest.raises(NativeError, match="aliases"):
        strip.strip_one(src, src, str(tool))


def test_list_mode_delegates_payload_checks_and_publishes_manifest_atomically(
    producer, monkeypatch
):
    src, cache, tool, _count = producer
    build = cache.parent / "build-amd64" / "boot"
    build.mkdir(parents=True)
    source = build / "livecd.Debug.lst"
    source.write_text(f"reactos/original.sys={src}\nreactos/empty={cache}\n")
    output = build / "livecd.stripped.lst"
    monkeypatch.setenv("VIBEOS_PAYLOAD", "")
    command = [
        str(ROOT / "tools/build-infra"),
        "strip-cd-media",
        "--lst",
        str(source),
        "--out",
        str(output),
        "--cache",
        str(cache),
        "--objcopy",
        str(tool),
    ]
    first = subprocess.run(command, capture_output=True, text=True)
    assert first.returncode == 0, first.stderr
    assert str(src) not in output.read_text()
    manifest = json.loads((cache / strip.MANIFEST).read_text())
    assert manifest[str(src)]["size"] == src.stat().st_size
    previous = output.read_bytes()
    monkeypatch.setenv("VIBEOS_PAYLOAD", "no-such-payload-for-refusal-control")
    failed = subprocess.run(command, capture_output=True, text=True)
    assert failed.returncode != 0 and "unknown payload" in failed.stderr
    assert output.read_bytes() == previous
    (cache / strip.MANIFEST).write_text("corrupt manifest")
    failed = subprocess.run(command, capture_output=True, text=True)
    assert failed.returncode != 0 and output.read_bytes() == previous


@pytest.mark.parametrize("changed", ["source", "tool"])
def test_source_or_tool_mutation_cannot_publish_over_previous_twin(producer, changed):
    src, cache, tool, _count = producer
    manifest = {}
    twin = strip.twin_for(src, cache, str(tool), manifest)
    assert twin is not None
    previous = twin.read_bytes()
    action = (
        "source.write_bytes(source.read_bytes()[:-1]+b'X')"
        if changed == "source"
        else f"tool=Path({str(tool)!r}); tool.write_text(tool.read_text()+'# edited\\n')"
    )
    tool.write_text(
        f"#!{sys.executable}\nimport sys\nfrom pathlib import Path\n"
        "source=Path(sys.argv[-2])\n"
        "data=source.read_bytes().replace(b'.debug\\0\\0', b'.text\\0\\0\\0')\n"
        + action
        + "\nPath(sys.argv[-1]).write_bytes(data)\n"
    )
    with pytest.raises(NativeError, match="changed"):
        strip.twin_for(src, cache, str(tool), manifest)
    assert twin.read_bytes() == previous


@pytest.mark.parametrize("machine,magic", [(0x14C, 0x10B), (0x8664, 0x20B), (0xAA64, 0x20B)])
def test_actual_llvm_objcopy_strips_all_arch_images_and_recomputes_checksum(
    tmp_path, machine, magic
):
    import shutil

    objcopy = shutil.which("llvm-objcopy") or shutil.which("llvm-objcopy-18")
    assert objcopy is not None, "native media integration needs the build's llvm-objcopy"
    data = image(magic)
    struct.pack_into("<H", data, 132, machine)
    struct.pack_into("<I", data, 184, 4096)  # SectionAlignment
    struct.pack_into("<I", data, 188, 512)  # FileAlignment
    struct.pack_into("<I", data, 208, 8192)  # SizeOfImage
    struct.pack_into("<I", data, 212, 512)  # SizeOfHeaders
    struct.pack_into("<IIII", data, 400, 8, 4096, 512, 512)
    struct.pack_into("<I", data, 428, 0x42000040)  # readable, discardable debug data
    src = tmp_path / "debug.sys"
    src.write_bytes(data)
    cache = tmp_path / "cache"
    cache.mkdir()
    twin = strip.twin_for(src, cache, objcopy, {})
    assert twin is not None
    stripped = twin.read_bytes()
    offset, sections = strip.parse_pe(stripped)
    assert not strip.has_debug_sections(sections)
    assert len(stripped) < len(data)
    assert struct.unpack_from("<I", stripped, offset)[0] == oracle_checksum(stripped, offset)
    assert src.read_bytes() == data


def test_media_payload_cli_uses_stamp_and_requires_an_arch_only_when_selected(
    tmp_path, monkeypatch
):
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "media_payload_control", ROOT / "tools/payloads/compile.py"
    )
    assert spec is not None and spec.loader is not None
    payloads = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(payloads)
    monkeypatch.delenv("VIBEOS_PAYLOAD", raising=False)
    monkeypatch.delenv("VIBEOS_DOOM", raising=False)
    observed = []
    monkeypatch.setattr(
        payloads, "manifest", lambda names, arch: observed.append((names, arch)) or {}
    )
    build = tmp_path / "build-i386"
    build.mkdir()
    (build / "payload-selection.json").write_text('{"payloads":["from-stamp"]}')
    assert payloads.main(["media-manifest", "--output", str(build / "boot/output.lst")]) == 0
    assert observed == [(["from-stamp"], "i386")]
    monkeypatch.setenv("VIBEOS_PAYLOAD", "")
    assert payloads.main(["media-manifest", "--output", str(tmp_path / "output.lst")]) == 0
    monkeypatch.setenv("VIBEOS_PAYLOAD", "selected")
    with pytest.raises(SystemExit):
        payloads.main(["media-manifest", "--output", str(tmp_path / "output.lst")])
