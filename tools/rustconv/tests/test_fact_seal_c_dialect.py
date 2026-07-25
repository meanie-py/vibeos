"""Tests for the plain-C fact-sealing dialect.

Each test here is written to FAIL if the guard it covers is removed, because every
guard corresponds to a real translation defect:

* CamelCase enum constants going unprotected is the same class as the 61 invented
  kernel bugcheck codes (field manual §9.113) — invisible to rustc.
* Sealing STORAGE (a local, a parameter, a struct field) is the actively harmful
  direction: the contract tells the model the token is an opaque value when it is
  really a variable to read or assign.
* The ``cpp`` dialect must stay byte-identical forever, or cached *paid* shell32
  workers silently re-bill.
"""

import pytest

from rustconv.ddk_enum_constants import DDK_ENUM_CONSTANTS
from rustconv.fact_seal import FactSealError, seal_facts, unseal_facts

DRIVER_BODY = """
NTSTATUS Probe(PIRP Irp) {
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG Length = sizeof(ULONG);
    PVOID Info = NULL;
    Info = ExAllocatePoolWithTag(NonPagedPool, 0x200, 'tseT');
    KeWaitForSingleObject(&Ev, Executive, KernelMode, FALSE, NULL);
    Adapter->NdisReserved = 1;
    Ctx.NdisReserved = 2;
    return Status;
}
"""


def sealed_names(source: str, dialect: str = "c") -> set[str]:
    return {fact.original for fact in seal_facts(source, dialect=dialect).facts}


def test_camelcase_enum_constants_are_sealed_in_c_dialect():
    names = sealed_names(DRIVER_BODY)
    # Choosing the wrong sibling of any of these enums is invisible to rustc.
    assert {"NonPagedPool", "Executive", "KernelMode"} <= names


def test_camelcase_enum_constants_are_not_sealed_in_cpp_dialect():
    """The cpp dialect is FROZEN — new protections must not leak into it."""
    assert "NonPagedPool" not in sealed_names(DRIVER_BODY, dialect="cpp")


def test_declared_locals_and_parameters_are_never_sealed():
    names = sealed_names(DRIVER_BODY)
    # `Info` is a genuine DDK enum constant name AND a local here; storage wins.
    assert "Info" not in names
    assert "Irp" not in names


def test_struct_field_access_is_never_sealed():
    """`NdisReserved` is a real enum constant name and a real NDIS struct field."""
    assert "NdisReserved" in DDK_ENUM_CONSTANTS  # the allowlist really does contain it
    assert "NdisReserved" not in sealed_names(DRIVER_BODY)


def test_value_inside_sizeof_is_still_sealed():
    """Deliberate non-feature: no `sizeof(...)` exclusion.

    `sizeof(UNICODE_NULL)` is a real ReactOS idiom where the operand is a VALUE.
    An earlier revision excluded every sizeof operand to suppress type-name noise
    and thereby left 7 real UNICODE_NULL occurrences unprotected. Over-sealing a
    type is benign — `size_of::<NDIS_REQUEST>()` restores to valid Rust — so the
    guard was removed. Direction matters: never under-seal a value.
    """
    assert "UNICODE_NULL" in sealed_names("n = Len - sizeof(UNICODE_NULL);")
    assert "STATUS_SUCCESS" in sealed_names("x = STATUS_SUCCESS;")


def test_callee_named_like_an_enum_constant_is_not_sealed():
    assert "AcquireSpinLock" in DDK_ENUM_CONSTANTS
    assert "AcquireSpinLock" not in sealed_names("AcquireSpinLock(&lock);")


def test_enum_constant_round_trips_to_its_exact_c_spelling():
    """Restoring must give back the C spelling verbatim — bindgen matches it."""
    sealed = seal_facts("p = ExAllocatePool(NonPagedPool, 8);", dialect="c")
    rust = "unsafe fn f() {" + sealed.text + "}"
    restored = unseal_facts(rust, sealed)
    assert "NonPagedPool" in restored
    assert "8" in restored


def test_enum_fact_omission_is_a_hard_failure():
    """A dropped enum token must not pass silently — that is the whole point."""
    sealed = seal_facts("p = ExAllocatePool(NonPagedPool, 8);", dialect="c")
    with pytest.raises(FactSealError, match=r"missing"):
        unseal_facts("unsafe fn f() { p = ExAllocatePool(); }", sealed)


def test_unknown_dialect_fails_loud_rather_than_weakening_sealing():
    with pytest.raises(ValueError, match="unknown dialect"):
        seal_facts("x = 1;", dialect="C")  # note: case-sensitive on purpose


def test_mixed_case_numeric_defines_are_sealed():
    """`NERR_Success` is a #define with mixed case — neither matcher caught it before."""
    assert "NERR_Success" in sealed_names("dwResult = NERR_Success;")
    assert "CmResourceTypePort" in sealed_names("if (Desc->Type == CmResourceTypePort) {}")


def test_control_flow_macros_are_never_sealed():
    """Sealing RpcTryExcept/RpcEndExcept would corrupt the SEH structure itself.

    These are mixed-case object-like #defines, so only the numeric-replacement-text
    classifier keeps them out. 114 occurrences in the target corpus.
    """
    for macro in ("RpcTryExcept", "RpcEndExcept"):
        assert macro not in DDK_ENUM_CONSTANTS
        assert macro not in sealed_names(f"{macro}\n{{ x = 1; }}")


def test_type_aliases_are_never_sealed():
    """LDAPControl/LDAPMod/IUnknown are #define'd TYPE names, not values."""
    for type_name in ("LDAPControl", "LDAPMod", "IUnknown"):
        assert type_name not in DDK_ENUM_CONSTANTS


def test_generated_allowlist_excludes_all_caps_and_storage_words():
    # ALL_CAPS names are already covered by the symbol matcher; overlap would
    # double-seal. Storage-ambiguous words are excluded by policy.
    assert not any(name.isupper() for name in DDK_ENUM_CONSTANTS)
    for storage_word in ("Status", "Length", "Info", "Buffer", "Flags", "Type"):
        assert storage_word not in DDK_ENUM_CONSTANTS
