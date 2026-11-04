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


def test_module_override_takes_the_bugcheck_offset_form(tmp_path):
    # A bugcheck prints `ndis.sys+0x12D10` for a relocated driver. With the module named,
    # an address below its link-time base is that offset (the arm64 0x1E in ProSend had to
    # be symbolicated by adding the base by hand).
    repo = _index(
        tmp_path,
        [_mod(0x7FFB00000000, 0x7FFB00040000, "ndis.sys")],
        ["ndis.sys\t00007ffb00012a84\tProSend\tprotocol.c:485"],
    )
    assert "ndis.sys!ProSend+0x28c" in symbolize.resolve(
        repo, "amd64", 0x12D10, want_module="ndis.sys"
    )


def test_the_offset_form_is_an_offset_even_above_a_low_link_base(tmp_path):
    # The HALs link at 0x10000, so the bugcheck's `hal.dll+0x25B1F` is ABOVE the base. Read
    # as an address it named a float parser at 0x25B1F; the crash was memcpy at 0x35B1F.
    repo = _index(
        tmp_path,
        [_mod(0x10000, 0xD3000, "hal.dll")],
        [
            "hal.dll\t0000000000025b00\tfrom_str\tdec2flt.rs:1",
            "hal.dll\t0000000000035b10\tmem_copy_forward\tmemops.rs:148",
        ],
    )
    out = symbolize.resolve(repo, "amd64", 0x25B1F, want_module="hal.dll", offset=True)
    assert "hal.dll!mem_copy_forward+0xf" in out


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


def test_module_override_falls_back_to_live_nm_for_an_unindexed_module(tmp_path):
    repo = _index(
        tmp_path,
        [_mod(0x180000000, 0x180040000, "wow64.dll", "shared")],
        ["other.dll\t0000000000401000\tx\t-"],
    )
    calls = _live_authority(
        repo, "wow64.dll\t180017be0\tWow64pUserCallbackCommon\twow64_callback.c:346"
    )
    out = symbolize.resolve(repo, "amd64", 0x180017C18, want_module="wow64.dll")
    assert calls.read_text().splitlines() == ["amd64", "--one", str(repo / "build-amd64/wow64.dll")]
    assert "wow64.dll!Wow64pUserCallbackCommon+0x38" in out


def test_live_nm_is_NOT_used_without_an_explicit_module(tmp_path):
    repo = _index(
        tmp_path,
        [
            _mod(0x180000000, 0x180040000, "wow64.dll", "shared"),
            _mod(0x180000000, 0x180040000, "other.dll", "shared"),
        ],
        [],
    )
    calls = _live_authority(repo, "unexpected\t180017be0\tWrong\t-")
    assert "ambiguous" in symbolize.resolve(repo, "amd64", 0x180017C18)
    assert not calls.exists(), "ambiguous addresses must never invoke the live-symbol authority"


def _copies(tmp_path: Path, contents: list[bytes]) -> Path:
    mods = []
    for i, data in enumerate(contents):
        rel = f"build-amd64/copy{i}/browseui.dll"
        (tmp_path / rel).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / rel).write_bytes(data)
        mods.append(f"{0x7FF62000000:016x}\t{0x7FF62100000:016x}\tbrowseui.dll\t{rel}\tshared")
    return _index(tmp_path, mods, [])


def test_byte_identical_copies_are_one_module(tmp_path):
    repo = _copies(tmp_path, [b"same", b"same"])
    calls = _live_authority(repo, "browseui.dll\t7ff62047050\tOnDestroy\t-")
    out = symbolize.resolve(repo, "amd64", 0x7FF620473A2)
    assert "browseui.dll!OnDestroy+0x352" in out
    assert len(calls.read_text().splitlines()) == 3, "one authority call with arch/--one/image"


def test_different_copies_on_one_base_stay_ambiguous(tmp_path):
    repo = _copies(tmp_path, [b"one", b"two"])
    assert "ambiguous" in symbolize.resolve(repo, "amd64", 0x7FF620473A2)


def _live_authority(repo, row):
    import shlex

    calls = repo / "calls"
    (repo / "qemu/build-user-index.sh").write_text(
        f"printf '%s\\n' \"$@\" >> {shlex.quote(str(calls))}\nprintf '%s\\n' {shlex.quote(row)}\n"
    )
    return calls
