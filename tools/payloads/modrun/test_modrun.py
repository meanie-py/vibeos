"""modrun's grader: the guest only reports, so the host's verdict is the whole test."""

import importlib.util
import sys
from pathlib import Path

import pytest

_PATH = Path(__file__).resolve().parent / "modrun.py"
_SPEC = importlib.util.spec_from_file_location("modrun_under_test", _PATH)
assert _SPEC is not None and _SPEC.loader is not None
mr = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = mr
_SPEC.loader.exec_module(mr)


@pytest.fixture
def suites(tmp_path, monkeypatch):
    (tmp_path / "m.yaml").write_text(
        "module: m\nvariants: {c: 'C:\\c.exe', rust: 'C:\\r.exe'}\n"
        "normalize: [['pid=\\d+', 'pid=N']]\n"
        "cases:\n"
        "  - {id: same, stdin: [list], expect: [Disk 0]}\n"
        "  - {id: differs}\n"
        "  - {id: crash, arches: [amd64]}\n"
        "  - {id: known, xfail: 'bug 7'}\n"
        "  - id: better\n"
        "    diverge: [{why: 'rust finds it', c: '^find ', rust: '^find '}]\n"
    )
    monkeypatch.setattr(mr, "SUITES", tmp_path)
    return tmp_path


def _serial(*lines):
    """Lines in the test's shorthand -- `[MODRUN:case:variant] text` and
    `[MODRUN] exit case variant rc=.. ms=..` -- as modrun.c writes them: numbered,
    checksummed, every record twice, the exit carrying the record count."""
    out, seq = [], {}
    for ln in lines:
        rec = mr.re.match(r"\[MODRUN:([^:\]]+):([^\]]+)\] ?(.*)", ln)
        ex = mr.re.match(r"\[MODRUN\] exit (\S+) (\S+) (.*)", ln)
        if rec:
            key = (rec.group(1), rec.group(2))
            n = seq.get(key, 0)
            text = rec.group(3)
            line = f"[MODRUN:{key[0]}:{key[1]}:{n}:{mr.checksum(text, n):04x}] {text}"
            out += [line, line]
            seq[key] = n + 1
        elif ex:
            rest = ex.group(3).split(" ", 2)
            out.append(
                f"[MODRUN] exit {ex.group(1)} {ex.group(2)} {rest[0]} {rest[1]} "
                f"records={seq.get((ex.group(1), ex.group(2)), 0)} {' '.join(rest[2:])}".rstrip()
            )
        else:
            out.append(ln)
    return "\n".join(out) + "\n"


def test_case_file_and_arch_filter(suites):
    text = mr.case_file("i386")
    assert (
        "case m.same\nvariant c C:\\c.exe\nvariant rust C:\\r.exe\nstdin list\ntimeout 60\nend"
        in text
    )
    assert "m.crash" not in text
    assert "m.crash" in mr.case_file("amd64")


def test_grading_diffs_against_the_first_variant(suites):
    serial = _serial(
        "[MODRUN:m.same:c] Disk 0 pid=12",
        "noise from another process",
        "[MODRUN] exit m.same c rc=0x00000000 ms=5",
        "[MODRUN:m.same:rust] Disk 0 pid=99   ",
        "[MODRUN] exit m.same rust rc=0x00000000 ms=5",
        "[MODRUN:m.differs:c] a",
        "[MODRUN] exit m.differs c rc=0x00000000 ms=5",
        "[MODRUN:m.differs:rust] b",
        "[MODRUN] exit m.differs rust rc=0x00000001 ms=5",
        "[MODRUN] exit m.crash c rc=0x00000000 ms=5",
        "[MODRUN] exit m.crash rust rc=0xe0525354 ms=5",
        "[MODRUN] exit m.known c rc=0x00000000 ms=5",
        "[MODRUN:m.known:rust] x",
        "[MODRUN] exit m.known rust rc=0x00000000 ms=5",
    )
    report, failures = mr.grade(serial, "amd64-uefi")
    assert "PASS m.same (2 variant(s), 1 line(s) each)" in report
    text = "\n".join(failures)
    assert "m.differs: rust: exit 0x00000001, c exited 0x00000000" in text
    assert any("+b" in r for r in report)
    assert "m.crash: rust: crashed, exit 0xe0525354" in text
    assert any(r.startswith("XFAIL m.known") for r in report)  # differs, as expected
    assert "m.known" not in text


def test_a_missing_exit_and_an_unexpected_pass_both_fail(suites):
    serial = _serial(
        "[MODRUN:m.known:c] same",
        "[MODRUN] exit m.known c rc=0x00000000 ms=5",
        "[MODRUN:m.known:rust] same",
        "[MODRUN] exit m.known rust rc=0x00000000 ms=5",
    )
    _, failures = mr.grade(serial, "i386")
    assert any(f.startswith("m.same: c: never finished") for f in failures)
    assert any("m.known: XPASS" in f for f in failures)


def test_a_torn_copy_is_recovered_and_a_lost_record_is_transport_not_a_diff(suites):
    whole = _serial(
        "[MODRUN:m.same:c] Disk 0 pid=1",
        "[MODRUN] exit m.same c rc=0x00000000 ms=5",
        "[MODRUN:m.same:rust] Disk 0 pid=2",
        "[MODRUN] exit m.same rust rc=0x00000000 ms=5",
    ).splitlines()
    # Another writer lands inside the first copy of the rust record; its twin is whole.
    torn = list(whole)
    i = next(k for k, ln in enumerate(torn) if ":m.same:rust:0:" in ln)
    torn[i] = torn[i][:20] + "[CENSUS] wait profile" + torn[i][20:]
    report, _ = mr.grade("\n".join(torn), "i386")
    assert "PASS m.same (2 variant(s), 1 line(s) each)" in report
    # Both copies gone: the case is not graded as output.
    lost = [ln for ln in whole if ":m.same:rust:0:" not in ln]
    report, failures = mr.grade("\n".join(lost), "i386")
    assert any("TRANSPORT" in f for f in failures)
    assert not any("output differs" in r for r in report)


def test_a_declared_divergence_leaves_the_diff_and_a_stale_one_fails(suites):
    serial = _serial(
        "[MODRUN:m.better:c] find none",
        "[MODRUN:m.better:c] tail",
        "[MODRUN] exit m.better c rc=0x00000000 ms=5",
        "[MODRUN:m.better:rust] find disk0",
        "[MODRUN:m.better:rust] tail",
        "[MODRUN] exit m.better rust rc=0x00000000 ms=5",
    )
    report, failures = mr.grade(serial, "amd64-uefi")
    assert "PASS m.better (2 variant(s), 2 line(s) each, 1 declared divergence(s))" in report
    assert not any(f.startswith("m.better") for f in failures)

    # Everything else must still match...
    other = _serial(
        "[MODRUN:m.better:c] find none",
        "[MODRUN:m.better:c] tail",
        "[MODRUN] exit m.better c rc=0x00000000 ms=5",
        "[MODRUN:m.better:rust] find disk0",
        "[MODRUN:m.better:rust] TAIL",
        "[MODRUN] exit m.better rust rc=0x00000000 ms=5",
    )
    _, failures = mr.grade(other, "amd64-uefi")
    assert any(f.startswith("m.better: rust: output differs") for f in failures)
    # ...and once neither side prints the line, the entry is stale.
    stale = _serial(
        "[MODRUN:m.better:c] tail",
        "[MODRUN] exit m.better c rc=0x00000000 ms=5",
        "[MODRUN:m.better:rust] tail",
        "[MODRUN] exit m.better rust rc=0x00000000 ms=5",
    )
    _, failures = mr.grade(stale, "amd64-uefi")
    assert any("m.better: diverge 'rust finds it' matched no line" in f for f in failures)


def test_shared_data_is_staged_once_and_a_clash_fails(tmp_path, monkeypatch):
    # The CLI suites each stage the tools/payloads/modrun/data/ files they read.
    data = "{src: tools/payloads/modrun/data/a.txt, dest: bin/modrun/data/a.txt}"
    for m in ("find", "sort"):
        (tmp_path / f"{m}.yaml").write_text(
            f"module: {m}\nvariants: {{rust: x}}\nstage: [{data}]\ncases: [{{id: a}}]\n"
        )
    monkeypatch.setattr(mr, "SUITES", tmp_path)
    assert len(mr.stage_entries("amd64")) == 1
    (tmp_path / "comp.yaml").write_text(
        "module: comp\nvariants: {rust: x}\n"
        "stage: [{src: other/a.txt, dest: bin/modrun/data/a.txt}]\ncases: [{id: a}]\n"
    )
    with pytest.raises(SystemExit, match="staged from"):
        mr.stage_entries("amd64")


def test_a_record_ending_in_cr_is_not_lost():
    # more.com writes CR CR LF: the guest strips one CR and checksums "one\r".
    text = "one\r"
    rec = f"[MODRUN:m.x:c:0:{mr.checksum(text, 0):04x}] {text}"
    serial = f"{rec}\n{rec}\r\n[MODRUN] exit m.x c rc=0x0 ms=1 records=1\n"
    out, exits = mr.parse(serial)
    assert out[("m.x", "c")] == ["one\r"] and "LOST" not in exits[("m.x", "c")][2]


def test_return_minus_one_is_an_exit_code_not_a_crash(suites):
    # fc returns FCRET_INVALID (-1) on every usage error; both variants agree.
    serial = _serial(
        "[MODRUN:m.same:c] Disk 0 usage",
        "[MODRUN] exit m.same c rc=0xffffffff ms=5",
        "[MODRUN:m.same:rust] Disk 0 usage",
        "[MODRUN] exit m.same rust rc=0xffffffff ms=5",
    )
    report, failures = mr.grade(serial, "amd64-uefi")
    assert not [f for f in failures if f.startswith("m.same")], failures


def test_a_non_ascii_record_survives_the_latin1_serial(suites):
    # serial_client.py decodes LATIN-1; the guest checksums the UTF-8 bytes.
    sent = "Disk 0 thrΘe"
    wire = sent.encode("utf-8").decode("latin-1")
    serial = "\n".join(
        [
            f"[MODRUN:m.same:c:0:{mr.checksum(sent, 0):04x}] {wire}",
            "[MODRUN] exit m.same c rc=0x00000000 ms=5 records=1",
        ]
    )
    out, exits = mr.parse(serial)
    assert out[("m.same", "c")] == [sent]
    assert "LOST" not in exits[("m.same", "c")][2]
