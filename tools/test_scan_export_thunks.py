"""scan_export_thunks: an export whose code is a thunk to the same-named import."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_export_thunks as scanner

HEADERS = """
Export Table:
 DLL name: ole32.dll
 Ordinal      RVA  Name
     105  0x3d33a  CreateErrorInfo
     125  0x11000  GetClassFile

The Data Directory
"""

DISASM = """
000007ffa803d33a <CreateErrorInfo>:
7ffa803d33a: jmpq *0x79768(%rip)  # 0x7ffa80b6aa8 <__imp_CreateErrorInfo>

000007ffa8011000 <GetClassFile>:
7ffa8011000: pushq %rsi

000007ffa8020000 <SysAllocString>:
7ffa8020000: jmpq *0x100(%rip)  # 0x7ffa80b0000 <__imp_SysAllocString>
"""


def test_exports_parsed():
    assert scanner.exports_of(HEADERS) == {"CreateErrorInfo", "GetClassFile"}


def test_only_exported_self_thunks():
    # SysAllocString is an ordinary import thunk (not exported): not a finding.
    assert scanner.self_thunks(DISASM, {"CreateErrorInfo", "GetClassFile"}) == ["CreateErrorInfo"]


def test_non_amd64_refuses_instead_of_clean(tmp_path, monkeypatch, capsys):
    d = tmp_path / "build-i386"
    d.mkdir()
    monkeypatch.setattr(sys, "argv", ["scan_export_thunks", str(d), "--check"])
    assert scanner.main() == 2
    assert "not an amd64 build" in capsys.readouterr().err
