"""scan_pack_drift: a C pshpack struct whose Rust port is not packed."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import scan_pack_drift as spd


def _repo(tmp_path: Path, rust_repr: str) -> Path:
    mod = tmp_path / "base" / "system" / "dp"
    mod.mkdir(parents=True)
    (mod / "CMakeLists.txt").write_text(
        "list(APPEND SOURCE\n"
        "    # partlist.c  # replaced by Rust (sdk/lib/rust/dp-rust/src/partlist.rs)\n)\n"
    )
    (mod / "partlist.c").write_text(
        "#include <pshpack1.h>\n"
        "typedef struct _PARTITION_SECTOR { UCHAR BootCode[440]; ULONG Signature; } "
        "PARTITION_SECTOR, *PPARTITION_SECTOR;\n"
        "#include <poppack.h>\n"
        "typedef struct _LOOSE { ULONG a; } LOOSE;\n"
    )
    crate = tmp_path / "sdk" / "lib" / "rust" / "dp-rust" / "src"
    crate.mkdir(parents=True)
    (crate / "partlist.rs").write_text(
        f"{rust_repr}\npub struct _PARTITION_SECTOR {{\n    pub BootCode: [u8; 440],\n}}\n"
        "#[repr(C)]\npub struct LOOSE {\n    pub a: u32,\n}\n"
    )
    return tmp_path


def test_an_unpacked_port_of_a_packed_struct_is_flagged(tmp_path, monkeypatch):
    monkeypatch.setattr(spd, "REPO", _repo(tmp_path, "#[repr(C)]"))
    assert spd.scan() == [
        "base/system/dp/partlist.c: pack-drift _PARTITION_SECTOR is pshpack1 in the C, "
        "#[repr(C)] in the Rust"
    ]


def test_a_packed_port_passes(tmp_path, monkeypatch):
    monkeypatch.setattr(spd, "REPO", _repo(tmp_path, "#[repr(C, packed)]"))
    assert spd.scan() == []
