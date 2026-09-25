"""Tests for `vibeos-cli sym` — user-mode address -> module!symbol+offset.

The invariant under test is NOT "it resolves addresses". It is "it never names
a symbol it cannot prove owns the address". A symbolicator that guesses is
worse than none: a wrong function name sends the hunt somewhere else entirely,
confidently, and nothing downstream catches it.
"""

from __future__ import annotations

from pathlib import Path

from tests.qemu.cli import symbolize


def _index(tmp_path: Path, modules: list[str], symbols: list[str]) -> Path:
    repo = tmp_path
    (repo / "qemu").mkdir(parents=True, exist_ok=True)
    (repo / "qemu" / "modules-amd64.tsv").write_text("\n".join(modules) + "\n")
    (repo / "qemu" / "symbols-amd64-user.tsv").write_text("\n".join(symbols) + "\n")
    return repo


def _mod(base: int, end: int, name: str, flag: str = "unique") -> str:
    return f"{base:016x}\t{end:016x}\t{name}\tbuild-amd64/{name}\t{flag}"


def test_resolves_to_module_symbol_and_offset(tmp_path):
    repo = _index(
        tmp_path,
        [_mod(0x7FFB7000000, 0x7FFB72DD000, "ntdll.dll")],
        ["ntdll.dll\t000007ffb70078d0\tvDbgPrintExWithPrefixInternal\tdebug.rs:93"],
    )
    out = symbolize.resolve(repo, "amd64", 0x7FFB70078FD)
    assert "ntdll.dll!vDbgPrintExWithPrefixInternal+0x2d" in out
    assert "debug.rs:93" in out


def test_picks_the_NEAREST_preceding_symbol_not_the_first(tmp_path):
    repo = _index(
        tmp_path,
        [_mod(0x400000, 0x500000, "app.exe")],
        [
            "app.exe\t0000000000401000\tfirst\t-",
            "app.exe\t0000000000401100\tsecond\t-",
            "app.exe\t0000000000401200\tthird\t-",
        ],
    )
    assert "second+0x10" in symbolize.resolve(repo, "amd64", 0x401110)


def test_declines_when_modules_SHARE_an_image_base(tmp_path):
    # 86 drivers sit on the linker default and are relocated at load time, so no
    # address can identify them. Naming one would be a coin flip presented as a
    # fact -- this is the case a single flat address table gets wrong.
    repo = _index(
        tmp_path,
        [
            _mod(0x10000, 0x20000, "acpi.sys", "shared"),
            _mod(0x10000, 0x20000, "afd.sys", "shared"),
        ],
        ["acpi.sys\t0000000000011000\tAcpiStart\t-"],
    )
    out = symbolize.resolve(repo, "amd64", 0x11500)
    assert "ambiguous" in out and "--module" in out
    assert "AcpiStart" not in out, "must not name a symbol from a coin-flip module"


def test_module_override_resolves_a_shared_base_module(tmp_path):
    repo = _index(
        tmp_path,
        [
            _mod(0x10000, 0x20000, "acpi.sys", "shared"),
            _mod(0x10000, 0x20000, "afd.sys", "shared"),
        ],
        ["acpi.sys\t0000000000011000\tAcpiStart\t-"],
    )
    assert "acpi.sys!AcpiStart+0x500" in symbolize.resolve(
        repo, "amd64", 0x11500, want_module="acpi.sys"
    )


def test_declines_when_the_nearest_symbol_is_implausibly_far(tmp_path):
    # A PE symbol table omits statics, thunks and data regions, so SOME symbol
    # always sorts below any address. Past MAX_SANE_OFFSET that is not evidence.
    repo = _index(
        tmp_path,
        [_mod(0x400000, 0x900000, "app.exe")],
        ["app.exe\t0000000000401000\tonly_symbol\t-"],
    )
    out = symbolize.resolve(repo, "amd64", 0x401000 + symbolize.MAX_SANE_OFFSET + 1)
    assert "no symbol" in out and "outside any function" in out
    assert "only_symbol" not in out


def test_declines_when_no_module_covers_the_address(tmp_path):
    repo = _index(tmp_path, [_mod(0x400000, 0x500000, "app.exe")], ["app.exe\t400000\tx\t-"])
    assert "no user-mode module covers" in symbolize.resolve(repo, "amd64", 0xDEADBEEF)


def test_missing_index_says_how_to_build_it(tmp_path):
    (tmp_path / "qemu").mkdir(parents=True)
    assert "build-user-index.sh" in symbolize.resolve(tmp_path, "amd64", 0x1000)


def test_index_is_stale_when_the_media_is_newer(tmp_path):
    # The failure this guards is silent: a stale index does not error, it names
    # a plausible WRONG symbol, because functions move between builds. Proven
    # in practice -- a hand-symbolication from an older ntdll named
    # RtlpAllocateMemory for an address that had become RtlpCaptureStackLimits.
    repo = _index(tmp_path, [_mod(0x400000, 0x500000, "a.exe")], ["a.exe\t400000\tx\t-"])
    (repo / "build-amd64").mkdir(parents=True)
    iso = repo / "build-amd64" / "livecd.iso"
    iso.write_text("")
    idx = symbolize.symbols_path(repo, "amd64")
    import os

    os.utime(idx, (1_000_000, 1_000_000))
    os.utime(iso, (2_000_000, 2_000_000))
    assert symbolize.index_is_stale(repo, "amd64") is True
    os.utime(iso, (500_000, 500_000))
    assert symbolize.index_is_stale(repo, "amd64") is False


def test_module_override_falls_back_to_live_nm_for_an_unindexed_module(tmp_path, monkeypatch):
    # Modules on a shared image base carry no rows in the index by design -- an
    # address cannot identify them. But --module supplies that identity, so the
    # only thing missing is symbols, and one nm answers it. Before this
    # fallback, --module returned "no symbol table entry" for exactly the
    # modules it exists to serve: wow64.dll shares 0x180000000 with 7 others,
    # and the WoW64 lane's addresses all live in it.
    repo = _index(
        tmp_path,
        [_mod(0x180000000, 0x180040000, "wow64.dll", "shared")],
        ["other.dll\t0000000000401000\tx\t-"],
    )
    called: list[tuple[str, int]] = []

    def fake_live(_repo, _arch, mod, addr):
        called.append((mod.name, addr))
        return ("Wow64pUserCallbackCommon", 0x180017BE0, "wow64_callback.c:346")

    monkeypatch.setattr(symbolize, "lookup_symbol_live", fake_live)
    out = symbolize.resolve(repo, "amd64", 0x180017C18, want_module="wow64.dll")
    assert called == [("wow64.dll", 0x180017C18)]
    assert "wow64.dll!Wow64pUserCallbackCommon+0x38" in out


def test_live_nm_is_NOT_used_without_an_explicit_module(tmp_path, monkeypatch):
    # The fallback must not turn an honest "ambiguous" into a guess: without
    # --module we still do not know which of the sharing modules it is.
    repo = _index(
        tmp_path,
        [
            _mod(0x180000000, 0x180040000, "wow64.dll", "shared"),
            _mod(0x180000000, 0x180040000, "other.dll", "shared"),
        ],
        [],
    )
    monkeypatch.setattr(
        symbolize,
        "lookup_symbol_live",
        lambda *a: (_ for _ in ()).throw(AssertionError("must not nm on an ambiguous address")),
    )
    assert "ambiguous" in symbolize.resolve(repo, "amd64", 0x180017C18)
