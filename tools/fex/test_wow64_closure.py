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


def _run(tmp_path, monkeypatch, built, capsys, extra=()):
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
    monkeypatch.setattr(
        sys, "argv", ["wow64-closure", str(root), "--build-dir", str(build), *extra]
    )
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


# ---- delay imports ---------------------------------------------------------
# gen-delaylib emits ImgDelayDescr tables as plain .rdata, so `objdump -p` shows
# no delay imports and the walk lost every one of them the day delay-load was
# switched on (user32_winetest: ~64 modules -> 29, silently).


def _pe_with_delay_descr(name: bytes, attrs: int = 1, exec_section: bool = False) -> bytes:
    """A minimal PE32: one section at VA 0x1000 / file 0x200 holding an
    ImgDelayDescr at its start and the DLL name at +0x40."""
    import struct

    pe_off, opt_size = 0x80, 0xE0
    img = bytearray(0x300)
    img[0:2] = b"MZ"
    struct.pack_into("<I", img, 0x3C, pe_off)
    img[pe_off : pe_off + 4] = b"PE\0\0"
    struct.pack_into("<HH12xH", img, pe_off + 4, 0x14C, 1, opt_size)
    struct.pack_into("<I", img, pe_off + 24 + 56, 0x2000)  # SizeOfImage
    sh = pe_off + 24 + opt_size
    img[sh : sh + 8] = b".rdata\0\0"
    chars = 0x60000020 if exec_section else 0x40000040
    struct.pack_into("<IIII12xI", img, sh + 8, 0x100, 0x1000, 0x100, 0x200, chars)
    struct.pack_into("<8I", img, 0x200, attrs, 0x1040, 0x1080, 0x1090, 0x10A0, 0, 0, 0)
    img[0x240 : 0x240 + len(name) + 1] = name + b"\0"
    return bytes(img)


def test_delay_descriptor_is_found_by_shape(tmp_path):
    pe = tmp_path / "rpcrt4.dll"
    pe.write_bytes(_pe_with_delay_descr(b"WinInet.dll"))
    assert w.delay_imports_of(pe) == ["wininet.dll"]


def test_a_lookalike_that_is_not_a_descriptor_is_ignored(tmp_path):
    pe = tmp_path / "x.dll"
    pe.write_bytes(_pe_with_delay_descr(b"wininet.dll", attrs=2))  # not the RVA form
    assert w.delay_imports_of(pe) == []
    pe.write_bytes(_pe_with_delay_descr(b"wininet.dll", exec_section=True))
    assert w.delay_imports_of(pe) == []
    pe.write_bytes(b"not a PE at all")
    assert w.delay_imports_of(pe) == []


def test_delay_imports_are_walked_like_imports(tmp_path, monkeypatch, capsys):
    """The regression itself: a module reached only through a delay import
    must be staged, and a missing one must fail as loudly as any import."""
    build = tmp_path / "build-i386"
    build.mkdir()
    root = build / "app.exe"
    root.write_bytes(b"MZ")
    rpc = build / "rpcrt4.dll"
    rpc.write_bytes(_pe_with_delay_descr(b"wininet.dll"))
    net = build / "wininet.dll"
    net.write_bytes(b"MZ")
    index = {"rpcrt4.dll": rpc, "wininet.dll": net}
    monkeypatch.setattr(w, "find_objdump", lambda _p: "objdump")
    monkeypatch.setattr(w, "index_build", lambda *_a, **_k: index)
    monkeypatch.setattr(w, "string_referenced", lambda _pe, _i: set())
    monkeypatch.setattr(w, "imports_of", lambda _o, pe: ["rpcrt4.dll"] if pe == root else [])
    monkeypatch.setattr(sys, "argv", ["wow64-closure", str(root), "--build-dir", str(build)])
    assert w.main() == 0
    staged = {ln.split("\t")[0] for ln in capsys.readouterr().out.splitlines() if "\t" in ln}
    assert "wininet.dll" in staged, "a delay-imported module was dropped from the closure"

    del index["wininet.dll"]
    assert w.main() == 1
    assert "UNRESOLVED import wininet.dll" in capsys.readouterr().err


def test_library_literals_follows_a_runtime_load_inside_a_library(tmp_path, monkeypatch, capsys):
    """comctl32 LoadLibrary's msvfw32 by name; only a library-wide literal
    scan sees it. Off by default (the arm64 ramdisk has no room), on for amd64."""
    build = tmp_path / "build-i386"
    build.mkdir()
    root = build / "app.exe"
    root.write_bytes(b"MZ")
    cc = build / "comctl32.dll"
    cc.write_bytes(b"MZ")
    vfw = build / "msvfw32.dll"
    vfw.write_bytes(b"MZ")
    index = {"comctl32.dll": cc, "msvfw32.dll": vfw}
    monkeypatch.setattr(w, "find_objdump", lambda _p: "objdump")
    monkeypatch.setattr(w, "index_build", lambda *_a, **_k: index)
    monkeypatch.setattr(
        w, "string_referenced", lambda pe, _i: {"msvfw32.dll"} if pe == cc else set()
    )
    monkeypatch.setattr(w, "imports_of", lambda _o, pe: ["comctl32.dll"] if pe == root else [])

    def staged(*flags):
        monkeypatch.setattr(
            sys, "argv", ["wow64-closure", str(root), "--build-dir", str(build), *flags]
        )
        assert w.main() == 0
        return {ln.split("\t")[0] for ln in capsys.readouterr().out.splitlines() if "\t" in ln}

    assert "msvfw32.dll" not in staged()
    assert "msvfw32.dll" in staged("--library-literals")


def test_winmm_drivers_derive_from_the_hive_and_skip_commented_lines(tmp_path):
    """winmm loads every Drivers32 driver by name; an x86-64 guest's sndrec32
    was refused midimap/beepmidi/msacm32.drv until this edge was derived."""
    inf_rel, kinds = w.DYNAMIC_LOADERS["winmm.dll"]
    d = tmp_path / pathlib.Path(inf_rel).parent
    d.mkdir(parents=True)
    key = 'HKLM,"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Drivers32"'
    (tmp_path / inf_rel).write_text(
        f'{key},"midimapper",0x00000000,"midimap.dll"\n;{key},"wave",0x00000000,"sndblst.dll"\n'
    )
    assert set(w.declared_modules(tmp_path, inf_rel, kinds)) == {"midimap.dll"}
    # ...and against the shipping hive.
    real = w.declared_modules(pathlib.Path(_REPO), inf_rel, kinds)
    assert {"midimap.dll", "msacm32.drv", "beepmidi.dll"} <= set(real)
    assert "sndblst.dll" not in real


def test_emit_roots_stages_the_programs_with_their_closure(tmp_path, monkeypatch, capsys):
    """--emit-roots (payload guest_sets) prints the root itself beside its DLLs."""
    rc, cap = _run(
        tmp_path, monkeypatch, ["ws2_32.dll", "msafd.dll", "wshtcpip.dll"], capsys, ["--emit-roots"]
    )
    assert rc == 0
    staged = {ln.split("\t")[0] for ln in cap.out.splitlines() if "\t" in ln}
    assert {"guest.exe", "ws2_32.dll", "msafd.dll"} <= staged
