"""tools/stub_hits.py: which stubs runs reach, what each does, ranked by runs."""

from __future__ import annotations

from pathlib import Path

from stub_hits import SpecIndex, ranked, resolved, run_logs, scan


def _run(root: Path, name: str, text: str, log: str = "serial.log") -> Path:
    d = root / name
    d.mkdir()
    (d / log).write_text(text)
    return d


def test_ranks_by_runs_before_lines(tmp_path: Path) -> None:
    _run(tmp_path, "a", "WARNING: calling stub foo.dll!Loud(0x1)\n" * 50)
    _run(tmp_path, "b", "WARNING: calling stub foo.dll!Wide()\n")
    _run(tmp_path, "c", "[+ 3.1] WARNING: calling stub foo.dll!Wide()\n")
    stubs, _, _ = scan(run_logs([tmp_path]))
    order = [(h.name, len(h.runs), h.lines) for h in ranked(stubs)]
    assert order == [("Wide", 2, 2), ("Loud", 1, 50)]


def test_each_boot_is_read_once(tmp_path: Path) -> None:
    """full.log repeats the serial output: reading both would count every hit twice. A
    smoke run keeps one serial.log per boot under boot<N>/, conformance an
    <arch>_serial.log; full.log is read only for a run with no capture beneath it."""
    line = "WARNING: calling stub foo.dll!X()\n"
    smoke = _run(tmp_path, "smoke", line, log="full.log")
    for b in ("boot0", "boot1"):
        (smoke / b).mkdir()
        (smoke / b / "serial.log").write_text(f"{b}: " + line)
    (smoke / "serial.log").write_text("boot0: " + line)  # smoke's copy of boot0's capture
    _run(tmp_path, "conf", "[+ 1.0] " + line, log="amd64-uefi_serial.log")
    _run(tmp_path, "old", "[+ 2.0] " + line, log="full.log")
    stubs, _, _ = scan(run_logs([tmp_path]))
    assert (len(stubs[("foo.dll", "X")].runs), stubs[("foo.dll", "X")].lines) == (4, 4)


def test_getprocaddress_misses_are_their_own_list(tmp_path: Path) -> None:
    _run(
        tmp_path,
        "a",
        "LDR: GetProcAddress miss KERNEL32.dll!GetSystemCpuSetInformation\n"
        "LDR: GetProcAddress miss kernel32.dll!#17\n",
    )
    stubs, misses, _ = scan(run_logs([tmp_path]))
    assert not stubs
    assert set(misses) == {("kernel32.dll", "GetSystemCpuSetInformation"), ("kernel32.dll", "#17")}


def test_kind_comes_from_the_spec(tmp_path: Path) -> None:
    spec = tmp_path / "foo.spec"
    spec.write_text(
        "@ stub Raiser\n"
        "@ stdcall -stub Zero(ptr)\n"
        "@ stdcall -stub=E_NOTIMPL Hr(ptr long)\n"
        "@ stdcall Real(ptr)\n"
    )
    index = SpecIndex({"foo.dll": [spec]})
    assert index.resolve("foo.dll", "Raiser") == ("foo.dll", "raises")
    assert index.resolve("foo.dll", "Zero") == ("foo.dll", "returns 0")
    assert index.resolve("foo.dll", "Hr") == ("foo.dll", "returns E_NOTIMPL")
    assert index.resolve("foo.dll", "Real") == ("foo.dll", "?")


def test_an_unqualified_line_is_matched_to_the_one_spec_stubbing_it(tmp_path: Path) -> None:
    """Stub lines from before the module was printed: unique names resolve, shared ones
    are reported as ambiguous rather than credited to whichever spec was read first."""
    a, b = tmp_path / "a.spec", tmp_path / "b.spec"
    a.write_text("@ stub Only\n@ stub Shared\n")
    b.write_text("@ stub Shared\n")
    index = SpecIndex({"a.dll": [a], "b.dll": [b]})
    assert index.resolve("", "Only") == ("a.dll", "raises")
    assert index.resolve("", "Shared") == ("a.dll|b.dll", "ambiguous")


def test_a_process_killed_by_a_stub_is_counted(tmp_path: Path) -> None:
    """kernel32's unhandled-exception filter names the stub; that run counts as a death
    even when the stub's own WARNING line was lost (or predates module names)."""
    _run(tmp_path, "a", "WARNING: calling stub foo.dll!Gone()\nMissing function: foo.dll!Gone\n")
    _run(tmp_path, "b", "Missing function: FOO.dll!Gone\n")
    _run(tmp_path, "c", "WARNING: calling stub foo.dll!Gone()\n")
    stubs, _, _ = scan(run_logs([tmp_path]))
    hit = stubs[("foo.dll", "Gone")]
    assert (len(hit.runs), len(hit.died), hit.lines) == (3, 2, 2)


def test_old_and_new_lines_for_one_stub_are_one_row(tmp_path: Path) -> None:
    spec = tmp_path / "foo.spec"
    spec.write_text("@ stub Gone\n")
    _run(tmp_path, "a", "WARNING: calling stub Gone()\n")
    _run(tmp_path, "b", "WARNING: calling stub foo.dll!Gone()\n")
    stubs, _, _ = scan(run_logs([tmp_path]))
    kinds: dict[tuple[str, str], str] = {}
    (hit,) = resolved(stubs, SpecIndex({"foo.dll": [spec]}), kinds)
    assert (hit.module, len(hit.runs), kinds[("foo.dll", "Gone")]) == ("foo.dll", 2, "raises")
