import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import pe_equiv


def make_pe(
    exports: list[str],
    image_base: int = 0x140000000,
    code: bytes = b"\xc3",
    dll: bytes = b"t.dll",
) -> bytes:
    """A minimal PE32+ DLL: one .rdata section holding an export directory, one .text."""
    names = [n.encode() + b"\0" for n in exports]
    n = len(exports)
    rdata_rva, text_rva = 0x1000, 0x2000
    # export dir (40) | functions (4n) | name ptrs (4n) | ordinals (2n) | dll name | names
    dllname = dll + b"\0"
    off_fun, off_np = 40, 40 + 4 * n
    off_ord = off_np + 4 * n
    off_dll = off_ord + 2 * n
    off_names = off_dll + len(dllname)
    rd = bytearray(off_names + sum(len(x) for x in names))
    struct.pack_into(
        "<IIHHIIIIIII",
        rd,
        0,
        0,
        0,
        0,
        0,
        rdata_rva + off_dll,
        1,
        n,
        n,
        rdata_rva + off_fun,
        rdata_rva + off_np,
        rdata_rva + off_ord,
    )
    p = off_names
    for i, nm in enumerate(names):
        struct.pack_into("<I", rd, off_fun + 4 * i, text_rva)
        struct.pack_into("<I", rd, off_np + 4 * i, rdata_rva + p)
        struct.pack_into("<H", rd, off_ord + 2 * i, i)
        rd[p : p + len(nm)] = nm
        p += len(nm)
    rd[off_dll : off_dll + len(dllname)] = dllname
    rd += b"\0" * (-len(rd) % 0x200)
    text = code + b"\0" * (-len(code) % 0x200)

    pe = 0x40
    opt_size = 240
    hdr = bytearray(0x400)
    hdr[0:2] = b"MZ"
    struct.pack_into("<I", hdr, 0x3C, pe)
    hdr[pe : pe + 4] = b"PE\0\0"
    struct.pack_into("<HHIIIHH", hdr, pe + 4, 0x8664, 2, 0, 0, 0, opt_size, 0x2022)
    o = pe + 24
    struct.pack_into("<H", hdr, o, 0x20B)
    struct.pack_into("<Q", hdr, o + 24, image_base)
    struct.pack_into("<IIHHHHHH", hdr, o + 32, 0x1000, 0x200, 5, 1, 0, 0, 5, 1)
    struct.pack_into("<HH", hdr, o + 68, 2, 0x140)
    struct.pack_into("<QQQQ", hdr, o + 72, 0x100000, 0x1000, 0x100000, 0x1000)
    struct.pack_into("<I", hdr, o + 108, 16)
    struct.pack_into("<II", hdr, o + 112, rdata_rva, off_names)  # export directory
    sect = o + opt_size
    for i, (name, rva, raw, data, chars) in enumerate(
        [
            (b".rdata", rdata_rva, 0x400, rd, 0x40000040),
            (b".text", text_rva, 0x400 + len(rd), text, 0x60000020),
        ]
    ):
        s = sect + 40 * i
        hdr[s : s + 8] = name.ljust(8, b"\0")
        struct.pack_into("<IIII", hdr, s + 8, len(data), rva, len(data), raw)
        struct.pack_into("<I", hdr, s + 36, chars)
    return bytes(hdr) + bytes(rd) + text


def test_layout_differences_are_equivalent_contract_differences_are_not():
    a = pe_equiv.PE(make_pe(["Alpha", "Beta"]))
    assert a.exports() == ["@1 Alpha", "@2 Beta"]
    # Different code bytes (what a different link order produces): still the same module.
    b = pe_equiv.PE(make_pe(["Alpha", "Beta"], code=b"\x90\xc3"))
    assert pe_equiv.compare(a, b) == []
    # A dropped export or a moved image base is a different module.
    c = pe_equiv.PE(make_pe(["Alpha"]))
    assert "exports: - @2 Beta" in pe_equiv.compare(a, c)
    d = pe_equiv.PE(make_pe(["Alpha", "Beta"], image_base=0x7FF00000000))
    assert any(x.startswith("header: - image_base") for x in pe_equiv.compare(a, d))


def test_code_mode_holds_every_mapped_byte():
    a = pe_equiv.PE(make_pe(["Alpha", "Beta"]))
    b = pe_equiv.PE(make_pe(["Alpha", "Beta"], code=b"\x90\xc3"))
    assert pe_equiv.compare(a, a, code=True) == []
    assert any(x.startswith("loaded: - .text") for x in pe_equiv.compare(a, b, code=True))


def test_not_a_pe_is_an_error_not_a_verdict(tmp_path):
    f = tmp_path / "x.dll"
    f.write_bytes(b"not a pe")
    try:
        pe_equiv.load(f)
    except pe_equiv.PEError:
        return
    raise AssertionError("expected PEError")


def test_the_modules_own_export_name_is_contract():
    # Every cargo cell shipped its bin's hashed name here until pefixup --dllname.
    good, cell = make_pe(["Alpha"]), make_pe(["Alpha"], dll=b"t-4d1aeb251b7d5eac.exe")
    assert pe_equiv.compare(pe_equiv.PE(good), pe_equiv.PE(cell)) == [
        "exports: - module t.dll",
        "exports: + module t-4d1aeb251b7d5eac.exe",
    ]
