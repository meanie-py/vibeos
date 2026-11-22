"""rename_module_refs: every way a test exe names the module, not just `name.dll`."""

import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "rename_module_refs", Path(__file__).with_name("rename_module_refs.py")
)
assert _spec and _spec.loader
rmr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(rmr)


def test_dll_and_bare_names_in_ascii_and_utf16():
    data = (
        b"\x00CRYPT32.dll\x00"  # import table
        b"\x00crypt32\x00"  # GetModuleHandleA("crypt32") -- crypt32 store.c
        b"\x00not_crypt32\x00crypt32x\x00"  # neither is the module name
        + "crypt32.dll\x00".encode("utf-16le")
        + b"\x00\x00"
        + "crypt32\x00".encode("utf-16le")
    )
    out, counts = rmr.rename(data, "crypt32", "crypt3c")
    assert len(out) == len(data)
    assert b"\x00crypt3c.dll\x00" in out and b"\x00crypt3c\x00" in out
    assert b"not_crypt32" in out and b"crypt32x" in out
    assert "crypt3c.dll".encode("utf-16le") in out
    assert counts["ascii crypt32.dll"] == 1 and counts["ascii crypt32"] == 1
    assert counts["utf16 crypt32.dll"] == 1 and counts["utf16 crypt32"] == 1


def test_length_must_be_kept():
    try:
        rmr.rename(b"", "shlwapi", "shlwapic")
    except ValueError:
        return
    raise AssertionError("a length-changing rename must be refused")
