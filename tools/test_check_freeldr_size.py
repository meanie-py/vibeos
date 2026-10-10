"""check_freeldr_size: the BIOS loader's real-mode size budget, read from the built PE."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import check_freeldr_size as m


def _pe(size_of_image: int) -> bytes:
    pe = bytearray(0x200)
    pe[:2] = b"MZ"
    struct.pack_into("<I", pe, 0x3C, 0x80)
    pe[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<I", pe, 0x80 + 24 + 56, size_of_image)
    return bytes(pe)


def _build(tmp_path: Path, size: int) -> Path:
    pe = tmp_path / m.PE
    pe.parent.mkdir(parents=True)
    pe.write_bytes(_pe(size))
    return tmp_path


def test_self_test_passes():
    assert m.self_test() == 0


def test_the_limit_is_read_from_the_real_header():
    assert m.limit_from_header(m.HEADER.read_text()) == 0x77000


def test_the_image_that_hung_every_bios_boot_fails(tmp_path):
    assert m.check(_build(tmp_path, 0x77000 + 1)) == 1


def test_an_image_at_the_limit_passes(tmp_path):
    assert m.check(_build(tmp_path, 0x77000)) == 0


def test_an_arch_without_a_bios_loader_skips(tmp_path, capsys):
    assert m.check(tmp_path) == 0
    assert "SKIP" in capsys.readouterr().out
