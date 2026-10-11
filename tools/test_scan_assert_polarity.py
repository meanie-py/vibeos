"""scan_assert_polarity: an ASSERT lowered with the condition as its own guard."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_assert_polarity as scanner


def site(guard: str, msg: str) -> str:
    return f'    if {guard} {{\n        RtlAssert(\n            b"{msg}\\0".as_ptr() as PVOID,\n'


def test_the_pcix_inversion():
    assert scanner.scan_text(site("!NT_SUCCESS(Status)", "!NT_SUCCESS(Status)")) == [
        (1, "!NT_SUCCESS(Status)", "!NT_SUCCESS(Status)")
    ]


def test_correct_lowerings_pass():
    assert scanner.scan_text(site("NT_SUCCESS(Status)", "!NT_SUCCESS(Status)")) == []
    assert scanner.scan_text(site("!(ResultLength > 0)", "ResultLength > 0")) == []
    assert (
        scanner.scan_text(
            site(
                "(*Entry).VendorID == PCI_INVALID_VENDORID as u16",
                "Entry->VendorID != PCI_INVALID_VENDORID",
            )
        )
        == []
    )
    assert scanner.scan_text(site("p.is_null()", "p")) == []


def test_rust_spelling_is_normalised():
    assert scanner.scan_text(site("(*Fdo).Fake == TRUE as u8", "Fdo->Fake == TRUE"))
    assert scanner.scan_text(site("x.is_null()", "x == NULL"))
