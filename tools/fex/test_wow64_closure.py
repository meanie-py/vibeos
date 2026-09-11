"""Unit tests for wow64-closure — the SysWOW64 staging closure.

The tool's whole contract is "never emit a PARTIAL closure", and it enforced
that for STATIC imports only. A provider loaded through the Winsock catalog was
invisible to an import-table walk, so msafd.dll went unstaged and every 32-bit
socket() failed with nothing in the log to say why. These tests pin the
DYNAMIC half of that contract, because the static half already had teeth and
the dynamic half is the one that shipped wrong.

Staging msafd then exposed a SECOND hop with the same shape: msafd loads its
WSH_* helper (wshtcpip.dll) from `HelperDllName`, and with only the first hop
staged socket() still returned WSAEINVAL while msafd itself loaded perfectly.
So the transitivity of the dynamic walk is pinned here too — it is the property
that made a one-hop fix look like a working one.
"""

import importlib.util
import os
import pathlib
import sys

import pytest

_HERE = os.path.dirname(os.path.abspath(__file__))
_REPO = os.path.dirname(os.path.dirname(_HERE))
_spec = importlib.util.spec_from_file_location(
    "wow64_closure", os.path.join(_HERE, "wow64-closure.py")
)
assert _spec and _spec.loader
w = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(w)

# Both spellings a catalog entry uses, in the layout the real INF uses: the
# packed blob is a WSAPROTOCOL_INFO whose leading field is the NUL-terminated
# path, continuation-escaped across lines.
_PACKED = (
    "25,53,79,73,74,65,6D,52,6F,6F,74,25,5C,73,79,73,\\\n"
    "74,65,6D,33,32,5C,6D,73,61,66,64,2E,64,6C,6C,00,\\\n"
    "00,00,00,00,00,00,00,00\n"
)
_FIXTURE_INF = (
    'HKLM,"...\\Catalog_Entries\\000000000001","LibraryPath",'
    '0x00000000,"%SystemRoot%\\system32\\mswsock.dll"\n'
    'HKLM,"...\\Catalog_Entries\\000000000001","PackedCatalogItem",'
    "0x00000001,\\\n" + _PACKED + 'HKLM,"...\\Tcpip\\Parameters\\WinSock","HelperDllName",'
    '0x00000000,"%SystemRoot%\\System32\\wshtcpip.dll"\n'
)

_WS2 = w.DYNAMIC_LOADERS["ws2_32.dll"][1]
_MSAFD = w.DYNAMIC_LOADERS["msafd.dll"][1]


def _inf(tmp_path, text=_FIXTURE_INF):
    d = tmp_path / "media" / "inf"
    d.mkdir(parents=True)
    (d / "nettcpip.inf").write_text(text)
    return "media/inf/nettcpip.inf"


def test_derives_providers_from_both_catalog_spellings(tmp_path):
    got = w.declared_modules(tmp_path, _inf(tmp_path), _WS2)
    assert set(got) == {"mswsock.dll", "msafd.dll"}
    # The declaration is carried, not just the name: an unresolved provider has
    # to be able to point at the line that demanded it.
    assert "PackedCatalogItem" in got["msafd.dll"]
    assert "LibraryPath" in got["mswsock.dll"]


def test_real_catalog_still_names_msafd():
    """The fixture proves the parser; this proves it against the shipping INF.

    A fixture-only test would keep passing if the real catalog were restructured
    out from under it, which is the failure that started this.
    """
    inf_rel, kinds = w.DYNAMIC_LOADERS["ws2_32.dll"]
    assert "msafd.dll" in w.declared_modules(pathlib.Path(_REPO), inf_rel, kinds)


def test_missing_inf_is_loud_not_a_silent_static_only_closure(tmp_path):
    # The dangerous outcome is "no dynamic edges found", which is
    # indistinguishable from success. It must not be reachable.
    with pytest.raises(SystemExit):
        w.declared_modules(tmp_path, "media/inf/nettcpip.inf", _WS2)


def _run(tmp_path, monkeypatch, built, capsys):
    """Drive main() over a fake build tree whose root imports ws2_32."""
    build = tmp_path / "build-i386"
    build.mkdir()
    index = {}
    for name in built:
        p = build / name
        p.write_bytes(b"MZ")
        index[name] = p
    root = build / "guest.exe"
    root.write_bytes(b"MZ")

    imports = {"guest.exe": ["ws2_32.dll"]}
    monkeypatch.setattr(w, "find_objdump", lambda _p: "objdump")
    monkeypatch.setattr(w, "index_build", lambda *_a, **_k: index)
    monkeypatch.setattr(w, "imports_of", lambda _o, pe: imports.get(pe.name, []))
    decls = {
        _WS2: {"msafd.dll": "nettcpip.inf PackedCatalogItem"},
        _MSAFD: {"wshtcpip.dll": "nettcpip.inf HelperDllName"},
    }
    monkeypatch.setattr(w, "declared_modules", lambda _p, _i, kinds: decls[kinds])
    monkeypatch.setattr(sys, "argv", ["wow64-closure", str(root), "--build-dir", str(build)])
    rc = w.main()
    return rc, capsys.readouterr()


def test_catalog_provider_is_staged_even_though_nobody_imports_it(tmp_path, monkeypatch, capsys):
    rc, cap = _run(tmp_path, monkeypatch, ["ws2_32.dll", "msafd.dll", "wshtcpip.dll"], capsys)
    assert rc == 0
    staged = {ln.split("\t")[0] for ln in cap.out.splitlines() if "\t" in ln}
    assert "msafd.dll" in staged, "the dynamic provider was dropped again"
    assert "ws2_32.dll" in staged
    # The derivation is reported, so a reader can see WHY it was staged.
    assert "dynamic provider msafd.dll" in cap.err


def test_a_second_hop_target_is_staged_too(tmp_path, monkeypatch, capsys):
    """msafd is itself a loader; admitting it must drag in its helper.

    Staging only the first hop is what made the msafd fix look complete: msafd
    loaded, DllMain returned 1, and socket() still failed with WSAEINVAL.
    """
    rc, cap = _run(tmp_path, monkeypatch, ["ws2_32.dll", "msafd.dll", "wshtcpip.dll"], capsys)
    assert rc == 0
    staged = {ln.split("\t")[0] for ln in cap.out.splitlines() if "\t" in ln}
    assert "wshtcpip.dll" in staged, "the dynamic walk stopped after one hop"
    assert "dynamic provider wshtcpip.dll" in cap.err


def test_unbuilt_second_hop_target_fails_as_loudly_as_a_missing_import(
    tmp_path, monkeypatch, capsys
):
    rc, cap = _run(tmp_path, monkeypatch, ["ws2_32.dll", "msafd.dll"], capsys)
    assert rc == 1, "a missing HELPER must fail like a missing provider"
    assert "UNRESOLVED import wshtcpip.dll" in cap.err
    assert "HelperDllName" in cap.err


def test_unbuilt_catalog_provider_fails_as_loudly_as_a_missing_import(
    tmp_path, monkeypatch, capsys
):
    rc, cap = _run(tmp_path, monkeypatch, ["ws2_32.dll"], capsys)
    assert rc == 1, "a partial DYNAMIC closure must fail like a partial static one"
    assert "UNRESOLVED import msafd.dll" in cap.err
    assert "PackedCatalogItem" in cap.err  # names the declaration that wanted it
