"""rr diverge's comparison: what it calls a mismatch, and how it names the bytes."""

from __future__ import annotations

import hashlib
import zlib
from pathlib import Path

from tests.qemu.cli import rrsample


def _fp(icount: int, regs: str, chunks: list[bytes], store: Path) -> dict:
    hashes = []
    for c in chunks:
        h = hashlib.blake2b(c, digest_size=8).hexdigest()
        (store / f"{h}.z").write_bytes(zlib.compress(c))
        hashes.append(h)
    return {"icount": icount, "regs": regs, "mem": [{"base": 0x40000000, "hashes": hashes}]}


def test_identical_fingerprints_do_not_diverge(tmp_path: Path) -> None:
    page = bytes(rrsample.CHUNK)
    a = _fp(10, "PC=1", [page, page], tmp_path)
    assert rrsample.compare(a, dict(a), tmp_path) is None


def test_a_differing_mib_is_named_down_to_the_byte(tmp_path: Path) -> None:
    rec = bytearray(rrsample.CHUNK)
    rep = bytearray(rec)
    rep[0x123] = 0xAA
    rep[0x124] = 0xBB
    a = _fp(10, "PC=1", [bytes(rrsample.CHUNK), bytes(rec)], tmp_path)
    b = _fp(10, "PC=1", [bytes(rrsample.CHUNK), bytes(rep)], tmp_path)
    d = rrsample.compare(a, b, tmp_path)
    assert d is not None
    assert "regs" not in d
    assert d["mem_mib"] == [f"{0x40000000 + rrsample.CHUNK:#x}"]
    (chunk,) = d["mem_bytes"]
    assert chunk["bytes_differing"] == 2
    (run,) = chunk["runs"]
    assert run["addr"] == f"{0x40000000 + rrsample.CHUNK + 0x123:#x}"
    assert run["len"] == 2


def test_register_differences_are_reported_as_lines(tmp_path: Path) -> None:
    page = bytes(rrsample.CHUNK)
    d = rrsample.compare(
        _fp(1, "PC=1\nX0=2", [page], tmp_path), _fp(1, "PC=1\nX0=3", [page], tmp_path)
    )
    assert d is not None
    assert "-X0=2" in d["regs"] and "+X0=3" in d["regs"]


def test_ram_ranges_skip_the_x86_legacy_hole() -> None:
    assert rrsample.ram_ranges(["qemu-system-x86_64", "-m", "512M"]) == [
        (0, 0xA0000),
        (0x100000, (512 << 20) - 0x100000),
    ]
    assert rrsample.ram_ranges(["qemu-system-aarch64", "-m", "1G"]) == [(0x40000000, 1 << 30)]


REC = " PC=000000007c32a040 X00=000000004dc70000 X01=000000000000e171\nPSTATE=80000205 N--- EL1h"
REP = " PC=000000007fd25a80 X00=000000004dc70000 X01=000000000000e171\nPSTATE=000003c5 ---- EL1h"


def test_irq_entry_on_one_side_is_inconclusive_only_with_an_interrupt_logged() -> None:
    d = {"regs": ["-PC..", "+PC.."]}
    assert rrsample.artifact(d, False, True, REC, REP) is not None
    # no interrupt in the log at this icount: the same difference is REAL
    assert rrsample.artifact(d, False, False, REC, REP) is None


def test_irq_entry_never_excuses_a_general_register_difference() -> None:
    rep = REP.replace("X00=000000004dc70000", "X00=0000000000000001")
    assert rrsample.artifact({"regs": ["x"]}, False, True, REC, rep) is None


def test_io_at_pause_excuses_ram_only_never_registers() -> None:
    assert rrsample.artifact({"mem_mib": ["0x0"]}, True, False, REC, REC) is not None
    assert rrsample.artifact({"regs": ["x"], "mem_mib": ["0x0"]}, True, False, REC, REP) is None


def test_verify_ignores_the_harness_host_time_stamps(tmp_path: Path) -> None:
    from tests.qemu.cli import rrcmd

    a = tmp_path / "a.log"
    b = tmp_path / "b.log"
    a.write_text("# banner\n[+    0.000] Error: x[+    0.000] \n[+    0.030] Error: y\n")
    b.write_text(
        "# banner\n[+    0.000] Error: x[+    0.004] \n[+    0.034] Error: y[+    0.037] \n"
    )
    assert rrcmd._serial_divergence(rrcmd._guest_lines(a), rrcmd._guest_lines(b)) is None


def test_verify_tolerates_a_first_line_captured_late() -> None:
    from tests.qemu.cli import rrcmd

    rec = ["UEFI firmware (version x)", "next"]
    assert rrcmd._serial_divergence(rec, ["FI firmware (version x)", "next"]) is None
    assert rrcmd._serial_divergence(rec, ["FI firmware (version x)", "other"]) is not None


def test_verify_treats_a_final_line_cut_mid_print_as_a_prefix() -> None:
    from tests.qemu.cli import rrcmd

    rec = ["a", "b", "[PROC-TERM] PID=728", "more"]
    assert rrcmd._serial_divergence(rec, ["a", "b", "["]) is None
    assert rrcmd._serial_divergence(rec, ["a", "b", "[X"]) is not None


def test_file_serial_reads_everything_then_reports_the_guest_gone(tmp_path: Path) -> None:
    """The rr serial transport: a regular file, a liveness callback instead of EOF.
    Bytes written just before QEMU exits must still arrive."""
    import threading
    import time

    from tests.qemu.serial_client import SerialClient

    out = tmp_path / "serial.out"
    running = threading.Event()
    running.set()
    client = SerialClient()
    client.follow_file(str(out), alive=running.is_set)

    def guest() -> None:
        with out.open("w") as fh:
            fh.write("boot\n")
            fh.flush()
            time.sleep(0.3)
            fh.write("shell ready\nlast words")
        running.clear()

    threading.Thread(target=guest, daemon=True).start()
    client.accept(timeout=5)
    assert "shell ready" in client.read_until("shell ready", timeout=5)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline and not client._snapshot()[2]:
        time.sleep(0.05)
    buf, _, dead = client._snapshot()
    assert dead and buf.endswith("last words")
    try:
        client.send(b"x")
        raise AssertionError("send must refuse under the file transport")
    except OSError:
        pass
    client.close()
