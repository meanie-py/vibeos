"""scan_unresolved_imports: a shipped PE importing a name its exporter does not export."""

import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_unresolved_imports as scanner

SECTION_RVA = 0x1000
FILE_ALIGN = 0x200


def build_pe(
    *, pe64: bool, imports: dict[str, list[str | int]], exports: list[str], dll_name: str = "x.dll"
) -> bytes:
    """A minimal PE: headers plus one section holding the import and export tables."""
    body = bytearray()

    def put(blob: bytes, align: int = 4) -> int:
        while len(body) % align:
            body.append(0)
        at = len(body)
        body.extend(blob)
        return SECTION_RVA + at

    def cstr(s: str) -> int:
        return put(s.encode() + b"\0", 2)

    width = 8 if pe64 else 4
    ordflag = 1 << (63 if pe64 else 31)
    descriptors = []
    for dll, names in imports.items():
        thunks = []
        for n in names:
            thunks.append(
                ordflag | n if isinstance(n, int) else put(b"\0\0" + n.encode() + b"\0", 2)
            )
        ilt = put(b"".join(struct.pack("<Q" if pe64 else "<I", t) for t in [*thunks, 0]), width)
        descriptors.append((ilt, cstr(dll)))
    import_dir = put(
        b"".join(struct.pack("<IIIII", ilt, 0, 0, name, ilt) for ilt, name in descriptors)
        + b"\0" * 20
    )
    name_rvas = [cstr(e) for e in exports]
    names_table = put(b"".join(struct.pack("<I", r) for r in name_rvas))
    export_dir = put(
        struct.pack(
            "<IIHHIIIIIII", 0, 0, 0, 0, cstr(dll_name), 1, 0, len(exports), 0, names_table, 0
        )
    )

    opt_size = 240 if pe64 else 224
    pe_off = 0x40
    headers = bytearray(FILE_ALIGN)
    headers[:2] = b"MZ"
    struct.pack_into("<I", headers, 0x3C, pe_off)
    headers[pe_off : pe_off + 4] = b"PE\0\0"
    struct.pack_into(
        "<HHIIIHH", headers, pe_off + 4, 0x8664 if pe64 else 0x14C, 1, 0, 0, 0, opt_size, 0
    )
    opt = pe_off + 24
    struct.pack_into("<H", headers, opt, 0x20B if pe64 else 0x10B)
    dd = opt + (112 if pe64 else 96)
    struct.pack_into("<I", headers, dd - 4, 16)
    struct.pack_into("<II", headers, dd, export_dir, 0x28)
    struct.pack_into("<II", headers, dd + 8, import_dir, 20 * (len(descriptors) + 1))
    sect = opt + opt_size
    raw = bytes(body) + b"\0" * (-len(body) % FILE_ALIGN)
    headers[sect : sect + 8] = b".rdata\0\0"
    struct.pack_into("<IIII", headers, sect + 8, len(body), SECTION_RVA, len(raw), FILE_ALIGN)
    return bytes(headers) + raw


def test_pe32_reads_decorated_import_and_skips_ordinals():
    image = scanner.Pe(
        build_pe(
            pe64=False, imports={"ntoskrnl.exe": ["RtlAssert@16", 7, "IoCreateDevice"]}, exports=[]
        )
    )
    assert image.imports() == {"ntoskrnl.exe": ["RtlAssert@16", "IoCreateDevice"]}


def test_pe32plus_reads_imports_and_exports():
    image = scanner.Pe(
        build_pe(pe64=True, imports={"hal.dll": ["KeStallExecutionProcessor"]}, exports=["A", "B"])
    )
    assert image.imports() == {"hal.dll": ["KeStallExecutionProcessor"]}
    assert image.exports() == {"A", "B"}


def test_the_pcix_incident():
    findings = scanner.resolve(
        {"pcix.sys": {"ntoskrnl.exe": ["RtlAssert@16", "RtlAssert"]}},
        {"pcix.sys": set(), "ntoskrnl.exe": {"RtlAssert"}, "ntkrnlmp.exe": {"RtlAssert"}},
    )
    assert findings == [
        "UNRESOLVED pcix.sys -> ntoskrnl.exe!RtlAssert@16",
        "UNRESOLVED pcix.sys -> ntkrnlmp.exe!RtlAssert@16",
    ]


def test_kernel_imports_are_judged_against_every_variant():
    # Exported by the UP kernel only: works until the loader picks ntkrnlmp.
    findings = scanner.resolve(
        {"a.sys": {"NTOSKRNL.EXE": ["Foo"]}},
        {"a.sys": set(), "ntoskrnl.exe": {"Foo"}, "ntkrnlmp.exe": set()},
    )
    assert findings == ["UNRESOLVED a.sys -> ntkrnlmp.exe!Foo"]


def test_missing_module_and_unreadable_image_are_findings():
    findings = scanner.resolve(
        {"a.dll": {"gone.dll": ["X"]}, "b.dll": None, "freeldr.sys": None},
        {"a.dll": set(), "b.dll": None, "freeldr.sys": None},
    )
    assert findings == ["MISSING_DLL a.dll -> gone.dll", "UNREADABLE b.dll"]


def test_loader_provided_imports_are_not_judged():
    assert (
        scanner.resolve(
            {"rosload.exe": {"freeldr.sys": ["MmAllocateMemory"]}}, {"rosload.exe": set()}
        )
        == []
    )


def test_a_cargo_cells_hashed_export_name_is_a_finding():
    image = scanner.Pe(
        build_pe(pe64=True, imports={}, exports=["A"], dll_name="dcomp-4d1aeb251b7d5eac.exe")
    )
    assert image.module_name() == "dcomp-4d1aeb251b7d5eac.exe"
    assert scanner.misnamed(
        {
            "dcomp.dll": image.module_name(),
            "ntdll.dll": "NTDLL.dll",  # case is not a mismatch
            "lsass.exe": None,  # exports nothing
            "kernel32test_versioned.dll": "redirtest1.dll",  # renamed on the medium on purpose
        }
    ) == ["EXPORT_NAME dcomp.dll: export directory names dcomp-4d1aeb251b7d5eac.exe"]
