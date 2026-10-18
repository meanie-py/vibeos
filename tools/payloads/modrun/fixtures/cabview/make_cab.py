"""Writes test.cab, the cabview suite's cabinet: an uncompressed (typeCompress 0) MS-CAB.

No cab tool is installed on the build host, and the format with no compression is a header,
one folder, the file records and raw data blocks, so this writes it directly. The members
cover what CCabFolder shows differently: a root file, one in a subdirectory with
FILE_ATTRIBUTE_READONLY|ARCHIVE, a UTF-8 name (_A_NAME_IS_UTF), and an empty file.
Re-run with `python3 make_cab.py` from this directory; the output is byte-identical.
"""

import struct
from pathlib import Path

_A_RDONLY, _A_ARCH, _A_NAME_IS_UTF = 0x01, 0x20, 0x80


def dos_datetime(y: int, mo: int, d: int, h: int, mi: int, s: int) -> tuple[int, int]:
    return ((y - 1980) << 9) | (mo << 5) | d, (h << 11) | (mi << 5) | (s // 2)


MEMBERS = [
    # (name bytes, attribs, data, (date, time))
    (b"readme.txt", _A_ARCH, b"hello from a cabinet\r\n", dos_datetime(2024, 1, 2, 3, 4, 6)),
    (b"docs\\notes.txt", _A_RDONLY | _A_ARCH, b"notes\r\n" * 40, dos_datetime(2023, 12, 31, 23, 59, 58)),
    ("café.txt".encode(), _A_ARCH | _A_NAME_IS_UTF, b"utf-8 name\r\n", dos_datetime(2020, 6, 15, 12, 0, 0)),
    (b"empty.dat", 0, b"", dos_datetime(2021, 2, 3, 4, 5, 6)),
]


def build() -> bytes:
    header_len, folder_len = 36, 8
    files = b""
    offset = 0
    for name, attr, data, (date, time) in MEMBERS:
        files += struct.pack("<IIHHHH", len(data), offset, 0, date, time, attr) + name + b"\0"
        offset += len(data)
    payload = b"".join(m[2] for m in MEMBERS)
    coff_files = header_len + folder_len
    coff_data = coff_files + len(files)
    # One CFDATA block holds everything (< 32 KiB); csum 0 means "not checksummed".
    block = struct.pack("<IHH", 0, len(payload), len(payload)) + payload
    total = coff_data + len(block)
    header = struct.pack(
        "<4sIIIIIBBHHHHH", b"MSCF", 0, total, 0, coff_files, 0, 3, 1, 1, len(MEMBERS), 0, 0x1234, 0
    )
    folder = struct.pack("<IHH", coff_data, 1, 0)
    out = header + folder + files + block
    assert len(header) == header_len and len(out) == total
    return out


if __name__ == "__main__":
    (Path(__file__).parent / "test.cab").write_bytes(build())
