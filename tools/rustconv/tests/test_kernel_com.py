from pathlib import Path

import pytest

from rustconv.__main__ import _extend_cpp_class_scan
from rustconv.cpp_bridge import BridgeError
from rustconv.kernel_com import generate_owner
from rustconv.scanner import scan_file

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "drivers/wdm/audio/backpln/portcls/resource.cpp"
BASE = SOURCE.with_name("private.hpp")
HEADER = SOURCE.with_name("resource_rust_bridge.h")


def generate(source=None, base=None):
    scan = _extend_cpp_class_scan(scan_file(SOURCE), SOURCE, "CResourceList")
    return generate_owner(
        source or SOURCE.read_text(),
        base or BASE.read_text(),
        scan,
        "CResourceList",
        HEADER.read_text(),
    )


def test_owner_preserves_state_lifetime_and_retained_worker_dispatch():
    generated = generate()
    assert "offset_of!(CResourceList, interface) == 0" in generated
    assert "AtomicI32::new(0)" in generated
    assert "m_PoolType: NonPagedPool" in generated
    assert "Ordering::SeqCst).wrapping_sub(1)" in generated
    assert generated.index(
        "kernel_assert!(count < 0x10000);", generated.index("fn release")
    ) < generated.index("if count == 0")
    assert generated.count("free_item((*object).") == 2
    assert "free_object(object.cast())" in generated
    assert generated.count("let mut ctx = context(this);") == 8


@pytest.mark.parametrize(
    "before,after",
    [
        ("CUnknownImpl<IResourceList>", "CUnknownImpl<IResourceList, IServiceSink>"),
        ("m_MaxEntries(0)", "m_MaxEntries(compute_capacity())"),
        (
            "FreeItem(m_TranslatedResourceList, TAG_PORTCLASS);",
            "FreeItem(m_TranslatedResourceList, TAG_PORTCLASS); Surprise();",
        ),
    ],
)
def test_unsupported_ownership_changes_fail_before_emission(before, after):
    with pytest.raises(BridgeError):
        generate(source=SOURCE.read_text().replace(before, after))


def test_changed_atomic_base_is_not_assumed_equivalent():
    with pytest.raises(BridgeError, match="atomic/lifetime"):
        generate(base=BASE.read_text().replace("InterlockedDecrement(&m_Ref)", "--m_Ref"))


def test_kernel_assert_preamble_is_derived_without_payment():
    from rustconv.kernel_preamble import derive_kernel_preamble

    scan = scan_file(SOURCE.with_name("resource_rust_reference.c"))
    result = derive_kernel_preamble(scan.preamble_text)
    assert result and "macro_rules! PC_ASSERT_IRQL_EQUAL" in result
    assert "RtlAssert(" in result
    assert derive_kernel_preamble(scan.preamble_text + "\nstatic int lost;") is None
    assert (
        derive_kernel_preamble(
            scan.preamble_text.replace("KeGetCurrentIrql()==(x)", "KeGetCurrentIrql()<=(x)")
        )
        is None
    )
