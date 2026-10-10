"""Deterministic trivial-C-body lowering (the $0, no-LLM path).

The accept tests are the cheap half. The NEAR-MISS tests are the point: every one
of them is a body that *looks* like an accepted shape and must come back ``None``.
If a near-miss ever starts being accepted, a wrong body ships with no cost signal
and no reviewer, which is strictly worse than paying for the translation.
"""

import pytest

from rustconv.callgraph import FunctionDep
from rustconv.trivial_c import (
    TrivialLoweringError,
    _lower_constant_expression,
    lower_trivial_c_function,
    lower_trivial_c_functions,
    verify_trivial_lowering,
)


def dep(body: str, *, name: str, ret: str = "DWORD", params=(), static=False, cpp_class=None):
    return FunctionDep(
        name=name,
        body_text=body,
        return_type=ret,
        params=list(params),
        is_static=static,
        cpp_class=cpp_class,
    )


def lower(body: str, keep_diagnostics: bool = False, **kwargs):
    return lower_trivial_c_function(dep(body, **kwargs), keep_diagnostics=keep_diagnostics)


# --------------------------------------------------------------------------- #
# shape 1 — constant-returning stub
# --------------------------------------------------------------------------- #


def test_symbolic_constant_return_is_lowered_verbatim():
    result = lower(
        "DWORD WINAPI Stub(void)\n{\n    return ERROR_CALL_NOT_IMPLEMENTED;\n}",
        name="Stub",
        params=[("void", "")],
    )
    assert result is not None
    assert result.shape == "constant_return"
    assert "#[no_mangle]" in result.rust
    assert 'pub unsafe extern "system" fn Stub() -> DWORD' in result.rust
    assert "ERROR_CALL_NOT_IMPLEMENTED" in result.rust


def test_hex_constant_is_not_rebased_to_decimal():
    result = lower("DWORD WINAPI S(void)\n{\n    return 0x200;\n}", name="S", params=[("void", "")])
    assert result is not None
    assert "0x200" in result.rust
    assert "512" not in result.rust


def test_ntstatus_stub_keeps_the_c_spelling():
    result = lower(
        "NTSTATUS NTAPI Drv(PDEVICE_OBJECT dev)\n{\n    return STATUS_NOT_IMPLEMENTED;\n}",
        name="Drv",
        ret="NTSTATUS",
        params=[("PDEVICE_OBJECT", "dev")],
    )
    assert result is not None
    assert result.rust.endswith("    STATUS_NOT_IMPLEMENTED\n}")
    assert "dev: PDEVICE_OBJECT" in result.rust


def test_bitwise_constant_expression_keeps_both_symbols():
    result = lower(
        "DWORD WINAPI F(void)\n{\n    return FLAG_A | FLAG_B;\n}", name="F", params=[("void", "")]
    )
    assert result is not None
    assert "FLAG_A | FLAG_B" in result.rust


def test_fact_sealed_constant_token_passes_straight_through():
    # A sealed body is just another constant spelling as far as this module is
    # concerned; the harness restores the value after the fact.
    result = lower(
        "DWORD WINAPI F(void)\n{\n    return __RUSTCONV_FACT_0007__;\n}",
        name="F",
        params=[("void", "")],
    )
    assert result is not None
    assert "__RUSTCONV_FACT_0007__" in result.rust


def test_windows_boolean_and_null_spellings_survive_the_oracle():
    # REGRESSION: tree-sitter-c folds `TRUE`/`FALSE`/`NULL`/`nullptr` into
    # dedicated `true`/`false`/`null` nodes while tree-sitter-rust reports the
    # same token as an `identifier`. The oracle caught that asymmetry on
    # wsock32's `DllMain` and refused the lowering; both spellings are now
    # enumerated. Without the fix these four raise TrivialLoweringError.
    for ret, expr in (
        ("BOOL", "TRUE"),
        ("BOOL", "FALSE"),
        ("BOOLEAN", "FALSE"),
        ("PVOID", "NULL"),
    ):
        result = lower(
            f"{ret} WINAPI F(void)\n{{\n    return {expr};\n}}",
            name="F",
            ret=ret,
            params=[("void", "")],
        )
        assert result is not None, (ret, expr)
        assert expr in result.rust


def test_static_c_function_gets_no_no_mangle():
    result = lower(
        "static DWORD Helper(void)\n{\n    return ERROR_SUCCESS;\n}",
        name="Helper",
        params=[("void", "")],
        static=True,
    )
    assert result is not None
    assert "#[no_mangle]" not in result.rust
    assert "pub " not in result.rust


# --------------------------------------------------------------------------- #
# shape 2 — pure forwarder
# --------------------------------------------------------------------------- #


def test_value_returning_forwarder():
    result = lower(
        "INT WINAPI Cpy(AsnAny *dst, AsnAny *src)\n{\n    return asn_any_copy(dst, src);\n}",
        name="Cpy",
        ret="INT",
        params=[("AsnAny *", "dst"), ("AsnAny *", "src")],
    )
    assert result is not None
    assert result.shape == "forwarder"
    assert "asn_any_copy(dst, src)" in result.rust
    assert "dst: *mut AsnAny" in result.rust


def test_void_forwarder_keeps_statement_form():
    result = lower(
        "VOID WINAPI Free(AsnAny *any)\n{\n    asn_any_free(any);\n}",
        name="Free",
        ret="VOID",
        params=[("AsnAny *", "any")],
    )
    assert result is not None
    assert result.shape == "forwarder"
    assert "asn_any_free(any);" in result.rust
    assert "->" not in result.signature


def test_zero_argument_forwarder():
    result = lower(
        "VOID WINAPI Init(void)\n{\n    RealInit();\n}",
        name="Init",
        ret="void",
        params=[("void", "")],
    )
    assert result is not None
    assert "RealInit();" in result.rust


# --------------------------------------------------------------------------- #
# shape 3 — getter / setter
# --------------------------------------------------------------------------- #


def test_getter_through_pointer_parameter():
    result = lower(
        "ULONG NTAPI GetBase(PADAPTER Adapter)\n{\n    return Adapter->IoBase;\n}",
        name="GetBase",
        ret="ULONG",
        params=[("PADAPTER *", "Adapter")],
    )
    assert result is not None
    assert result.shape == "getter"
    assert "(*Adapter).IoBase" in result.rust


def test_setter_through_pointer_parameter():
    result = lower(
        "VOID NTAPI SetMask(PADAPTER Adapter, ULONG Mask)\n{\n"
        "    Adapter->InterruptMask = Mask;\n}",
        name="SetMask",
        ret="VOID",
        params=[("PADAPTER *", "Adapter"), ("ULONG", "Mask")],
    )
    assert result is not None
    assert result.shape == "setter"
    assert "(*Adapter).InterruptMask = Mask;" in result.rust


def test_setter_with_constant_value_keeps_the_spelling():
    result = lower(
        "VOID NTAPI Clear(PADAPTER a)\n{\n    a->Flags = FLAG_NONE;\n}",
        name="Clear",
        ret="VOID",
        params=[("PADAPTER *", "a")],
    )
    assert result is not None
    assert "(*a).Flags = FLAG_NONE;" in result.rust


# --------------------------------------------------------------------------- #
# shape 4 — debug-only / empty
# --------------------------------------------------------------------------- #


def test_empty_void_body():
    result = lower("VOID WINAPI Nop(void)\n{\n}", name="Nop", ret="VOID", params=[("void", "")])
    assert result is not None
    assert result.shape == "empty"
    assert result.rust.endswith("fn Nop() {}")


def test_bare_return_is_an_empty_body():
    result = lower(
        "VOID WINAPI Nop(void)\n{\n    return;\n}", name="Nop", ret="VOID", params=[("void", "")]
    )
    assert result is not None
    assert result.shape == "empty"


def test_debug_only_body_becomes_a_comment_never_a_noop_call():
    # AGENTS.md / field manual 9.65: an out-of-line no-op fn is an optimization
    # barrier that uncloaks ABI mismatches into ud2. The lowering must not emit
    # any call at all.
    result = lower(
        'VOID WINAPI Stub(INT level)\n{\n    FIXME("(%d)\\n", level);\n}',
        name="Stub",
        ret="VOID",
        params=[("INT", "level")],
    )
    assert result is not None
    assert result.shape == "empty"
    assert result.dropped_debug == ('FIXME("(%d)\\n", level);',)
    assert '// FIXME("(%d)\\n", level);' in result.rust
    assert "FIXME(" not in result.rust.replace('// FIXME("(%d)\\n", level);', "")


def test_debug_statement_with_pure_wine_helper_is_dropped():
    result = lower(
        'DWORD WINAPI Stub(LPCSTR s)\n{\n    FIXME("(%s),stub!\\n", debugstr_a(s));\n'
        "    return 0;\n}",
        name="Stub",
        params=[("LPCSTR", "s")],
    )
    assert result is not None
    assert result.shape == "constant_return"
    assert len(result.dropped_debug) == 1
    assert result.rust.rstrip().endswith("0\n}")


def test_bare_unimplemented_macro_is_dropped():
    result = lower(
        "NTSTATUS NTAPI Stub(void)\n{\n    UNIMPLEMENTED;\n    return STATUS_NOT_IMPLEMENTED;\n}",
        name="Stub",
        ret="NTSTATUS",
        params=[("void", "")],
    )
    assert result is not None
    assert result.dropped_debug == ("UNIMPLEMENTED;",)


def test_wine_channel_form_debug_statement_is_dropped():
    result = lower(
        'DWORD WINAPI Stub(void)\n{\n    ERR_(winediag)("nope\\n");\n    return 1;\n}',
        name="Stub",
        params=[("void", "")],
    )
    assert result is not None
    assert result.dropped_debug == ('ERR_(winediag)("nope\\n");',)


def test_multiple_debug_statements_keep_source_order():
    result = lower(
        'VOID WINAPI Stub(void)\n{\n    TRACE("a\\n");\n    WARN("b\\n");\n}',
        name="Stub",
        ret="VOID",
        params=[("void", "")],
    )
    assert result is not None
    assert result.dropped_debug == ('TRACE("a\\n");', 'WARN("b\\n");')


# --------------------------------------------------------------------------- #
# NEAR MISSES — every one of these MUST be None
# --------------------------------------------------------------------------- #


def test_forwarder_with_a_cast_on_one_argument_is_rejected():
    assert (
        lower(
            "INT WINAPI F(AsnAny *dst, AsnAny *src)\n{\n    return copy((PVOID)dst, src);\n}",
            name="F",
            ret="INT",
            params=[("AsnAny *", "dst"), ("AsnAny *", "src")],
        )
        is None
    )


def test_forwarder_with_reordered_arguments_is_rejected():
    assert (
        lower(
            "INT WINAPI F(AsnAny *dst, AsnAny *src)\n{\n    return copy(src, dst);\n}",
            name="F",
            ret="INT",
            params=[("AsnAny *", "dst"), ("AsnAny *", "src")],
        )
        is None
    )


def test_forwarder_with_a_literal_substituted_for_a_parameter_is_rejected():
    assert (
        lower(
            "INT WINAPI F(AsnAny *dst, DWORD flags)\n{\n    return copy(dst, 0);\n}",
            name="F",
            ret="INT",
            params=[("AsnAny *", "dst"), ("DWORD", "flags")],
        )
        is None
    )


def test_forwarder_with_a_dropped_argument_is_rejected():
    assert (
        lower(
            "INT WINAPI F(AsnAny *dst, DWORD flags)\n{\n    return copy(dst);\n}",
            name="F",
            ret="INT",
            params=[("AsnAny *", "dst"), ("DWORD", "flags")],
        )
        is None
    )


def test_forwarder_with_an_address_of_argument_is_rejected():
    assert (
        lower(
            "INT WINAPI F(DWORD n)\n{\n    return copy(&n);\n}",
            name="F",
            ret="INT",
            params=[("DWORD", "n")],
        )
        is None
    )


def test_forwarder_with_a_nested_call_argument_is_rejected():
    # `HeapAlloc(GetProcessHeap(), 0, n)` is the shape that dominates the real
    # allocator wrappers — it is NOT an identity forwarder.
    assert (
        lower(
            "PVOID WINAPI Alloc(DWORD n)\n{\n    return HeapAlloc(GetProcessHeap(), 0, n);\n}",
            name="Alloc",
            ret="PVOID",
            params=[("DWORD", "n")],
        )
        is None
    )


def test_forwarder_with_an_extra_statement_is_rejected():
    assert (
        lower(
            "INT WINAPI F(AsnAny *dst)\n{\n    dst->len = 0;\n    return copy(dst);\n}",
            name="F",
            ret="INT",
            params=[("AsnAny *", "dst")],
        )
        is None
    )


def test_self_recursive_forwarder_is_rejected():
    assert (
        lower(
            "INT WINAPI F(DWORD n)\n{\n    return F(n);\n}",
            name="F",
            ret="INT",
            params=[("DWORD", "n")],
        )
        is None
    )


def test_getter_with_pointer_arithmetic_is_rejected():
    assert (
        lower(
            "ULONG NTAPI G(PADAPTER a)\n{\n    return (a + 1)->IoBase;\n}",
            name="G",
            ret="ULONG",
            params=[("PADAPTER *", "a")],
        )
        is None
    )


def test_getter_with_an_offset_expression_is_rejected():
    assert (
        lower(
            "ULONG NTAPI G(PADAPTER a)\n{\n    return a->IoBase + R_IS;\n}",
            name="G",
            ret="ULONG",
            params=[("PADAPTER *", "a")],
        )
        is None
    )


def test_chained_member_getter_is_rejected():
    assert (
        lower(
            "ULONG NTAPI G(PADAPTER a)\n{\n    return a->Inner->IoBase;\n}",
            name="G",
            ret="ULONG",
            params=[("PADAPTER *", "a")],
        )
        is None
    )


def test_getter_on_a_double_pointer_parameter_is_rejected():
    assert (
        lower(
            "ULONG NTAPI G(PADAPTER *a)\n{\n    return a->IoBase;\n}",
            name="G",
            ret="ULONG",
            params=[("PADAPTER * *", "a")],
        )
        is None
    )


def test_getter_on_a_non_parameter_base_is_rejected():
    assert (
        lower(
            "ULONG NTAPI G(void)\n{\n    return GlobalAdapter->IoBase;\n}",
            name="G",
            ret="ULONG",
            params=[("void", "")],
        )
        is None
    )


def test_debug_statement_with_a_side_effecting_argument_is_rejected():
    assert (
        lower(
            'VOID WINAPI Stub(INT i)\n{\n    FIXME("(%d)\\n", i++);\n}',
            name="Stub",
            ret="VOID",
            params=[("INT", "i")],
        )
        is None
    )


def test_debug_statement_calling_an_unknown_helper_is_rejected():
    assert (
        lower(
            'VOID WINAPI Stub(INT i)\n{\n    FIXME("(%d)\\n", Compute(i));\n}',
            name="Stub",
            ret="VOID",
            params=[("INT", "i")],
        )
        is None
    )


def test_debug_statement_with_an_assignment_argument_is_rejected():
    assert (
        lower(
            'VOID WINAPI Stub(INT i)\n{\n    FIXME("(%d)\\n", i = 3);\n}',
            name="Stub",
            ret="VOID",
            params=[("INT", "i")],
        )
        is None
    )


def test_arithmetic_constant_expression_is_never_folded():
    # The whole point: `38 + 1` must not silently become `39`. C promotes to int
    # and truncates; Rust evaluates in the return type. We cannot prove those
    # agree, so this stays on the paid path.
    result = lower(
        "DWORD WINAPI F(void)\n{\n    return 38 + 1;\n}", name="F", params=[("void", "")]
    )
    assert result is None


def test_shift_and_subtraction_constants_are_rejected():
    for expr in ("1 << 3", "0 - 1", "MAX / 2", "A * B", "A % B"):
        assert _lower_constant_expression(expr) is None


def test_bitwise_constants_are_accepted_and_verbatim():
    assert _lower_constant_expression("FLAG_A|FLAG_B") == "FLAG_A|FLAG_B"
    assert _lower_constant_expression("~0") == "!0"
    assert _lower_constant_expression("(A | B) & C") == "(A | B) & C"
    assert _lower_constant_expression("A |") is None
    assert _lower_constant_expression("(A") is None


def test_cast_in_a_returned_constant_is_rejected():
    assert lower("DWORD WINAPI F(void)\n{\n    return (DWORD)-1;\n}", name="F") is None


def test_integer_literal_returned_from_a_pointer_typed_function_is_rejected():
    # `return 0;` from an HRASCONN-returning C function needs null_mut(), which
    # is an interpretation of the constant, not a transliteration of it.
    assert (
        lower(
            "PVOID WINAPI F(void)\n{\n    return 0;\n}",
            name="F",
            ret="PVOID *",
            params=[("void", "")],
        )
        is None
    )


def test_empty_body_in_a_value_returning_function_is_rejected():
    assert lower("DWORD WINAPI F(void)\n{\n}", name="F", params=[("void", "")]) is None


def test_string_literal_body_is_rejected():
    assert (
        lower(
            'LPCSTR WINAPI F(void)\n{\n    return "hello";\n}',
            name="F",
            ret="LPCSTR",
            params=[("void", "")],
        )
        is None
    )


def test_variadic_function_is_rejected():
    # The scanner silently drops `...` from params; a Rust definition of a
    # C-variadic needs nightly, so the head text is the only reliable signal.
    assert (
        lower(
            'VOID __cdecl Log(INT level, LPSTR format, ...)\n{\n    FIXME("(%d)\\n", level);\n}',
            name="Log",
            ret="VOID",
            params=[("INT", "level"), ("LPSTR", "format")],
        )
        is None
    )


def test_knr_definition_is_rejected():
    assert (
        lower(
            "int f(a, b)\n    int a;\n    int b;\n{\n    return 0;\n}",
            name="f",
            ret="int",
            params=[("int", "a"), ("int", "b")],
        )
        is None
    )


def test_array_parameter_is_rejected():
    assert (
        lower(
            "int f(char *argv[])\n{\n    return 0;\n}",
            name="f",
            ret="int",
            params=[("char *", "argv")],
        )
        is None
    )


def test_function_pointer_parameter_is_rejected():
    assert (
        lower(
            "int f(int (*cmp)(int, int))\n{\n    return 0;\n}",
            name="f",
            ret="int",
            params=[("int", "cmp")],
        )
        is None
    )


def test_control_flow_body_is_rejected():
    assert (
        lower(
            "DWORD WINAPI F(DWORD n)\n{\n    if (n) return 1;\n    return 0;\n}",
            name="F",
            params=[("DWORD", "n")],
        )
        is None
    )


def test_out_parameter_write_is_rejected():
    # Deliberate non-shape: a #[no_mangle] output-pointer write owes a
    # write_volatile (captures(none)), which is not a transliteration.
    assert (
        lower(
            "DWORD WINAPI F(DWORD *n)\n{\n    *n = 0;\n    return 0;\n}",
            name="F",
            params=[("DWORD *", "n")],
        )
        is None
    )


def test_parameter_return_requires_identical_type_spellings():
    assert (
        lower(
            "HWND WINAPI F(HWND h)\n{\n    return h;\n}",
            name="F",
            ret="HWND",
            params=[("HWND", "h")],
        )
        is not None
    )
    assert (
        lower("DWORD WINAPI F(HWND h)\n{\n    return h;\n}", name="F", params=[("HWND", "h")])
        is None
    )


def test_cpp_method_is_left_to_the_com_pipeline():
    assert (
        lower(
            "HRESULT CFoo::Bar(void)\n{\n    return E_NOTIMPL;\n}",
            name="CFoo_Bar",
            ret="HRESULT",
            params=[("void", "")],
            cpp_class="CFoo",
        )
        is None
    )


def test_unnamed_parameter_blocks_forwarder_identity_but_not_a_constant_stub():
    unnamed = [("DWORD", "")]
    assert (
        lower("VOID WINAPI F(DWORD)\n{\n    Real();\n}", name="F", ret="VOID", params=unnamed)
        is None
    )
    stub = lower("DWORD WINAPI F(DWORD)\n{\n    return 0;\n}", name="F", params=unnamed)
    assert stub is not None
    assert "_: DWORD" in stub.rust


# --------------------------------------------------------------------------- #
# the safety net itself
# --------------------------------------------------------------------------- #


def test_every_accepted_lowering_passes_the_independent_oracle():
    cases = [
        dep(
            "DWORD WINAPI A(void)\n{\n    return ERROR_SUCCESS;\n}",
            name="A",
            params=[("void", "")],
        ),
        dep(
            "INT WINAPI B(AsnAny *dst, AsnAny *src)\n{\n    return copy(dst, src);\n}",
            name="B",
            ret="INT",
            params=[("AsnAny *", "dst"), ("AsnAny *", "src")],
        ),
        dep(
            "ULONG NTAPI C(PADAPTER a)\n{\n    return a->IoBase;\n}",
            name="C",
            ret="ULONG",
            params=[("PADAPTER *", "a")],
        ),
        dep(
            "VOID NTAPI D(PADAPTER a, ULONG m)\n{\n    a->Mask = m;\n}",
            name="D",
            ret="VOID",
            params=[("PADAPTER *", "a"), ("ULONG", "m")],
        ),
        dep(
            'VOID WINAPI E(void)\n{\n    TRACE("x\\n");\n}',
            name="E",
            ret="VOID",
            params=[("void", "")],
        ),
    ]
    lowered = lower_trivial_c_functions(cases)
    assert set(lowered) == {"A", "B", "C", "D", "E"}
    for case in cases:
        assert verify_trivial_lowering(case, lowered[case.name]) == []


def test_c_suffixed_integer_literal_is_rejected():
    # REGRESSION (iphlpapi `return 0L;`): `0L` is not a Rust literal, and emitting
    # `0` would be normalizing a constant. Reject rather than either.
    for expr in ("0L", "1UL", "0xFFul", "2u"):
        assert (
            lower(
                f"DWORD WINAPI F(void)\n{{\n    return {expr};\n}}", name="F", params=[("void", "")]
            )
            is None
        ), expr


def test_rust_keyword_parameter_names_are_raw_escaped():
    # REGRESSION (rpcrt4 `RpcMgmtSetAuthorizationFn(... fn)`): a bare `fn: T`
    # opens a nested item and silently changes the signature's ARITY.
    result = lower(
        "BOOL WINAPI F(TAGTYPE type, BOOL ref, PVOID fn)\n{\n    return Real(type, ref, fn);\n}",
        name="F",
        ret="BOOL",
        params=[("TAGTYPE", "type"), ("BOOL", "ref"), ("PVOID", "fn")],
    )
    assert result is not None
    assert "r#type: TAGTYPE" in result.rust
    assert "Real(r#type, r#ref, r#fn)" in result.rust


def test_unrawable_keyword_parameter_is_rejected():
    for name in ("self", "crate", "super", "Self"):
        assert (
            lower(
                f"DWORD WINAPI F(PVOID {name})\n{{\n    return 0;\n}}",
                name="F",
                params=[("PVOID", name)],
            )
            is None
        ), name


def test_cxx_this_parameter_getter_lowers():
    # REGRESSION (msvcrt/concurrency.c): the C grammar reports `this` as a keyword
    # node, not an identifier, so the oracle saw an asymmetric sequence.
    result = lower(
        "HRESULT F(err_t *this)\n{\n    return this->hr;\n}",
        name="F",
        ret="HRESULT",
        params=[("err_t *", "this")],
    )
    assert result is not None
    assert "(*this).hr" in result.rust


def test_unmappable_return_type_is_rejected():
    # REGRESSION (dbghelp `static struct module* f(..)`): the scanner recovers the
    # return type as the bare spelling `*`, which renders as `-> *mut `.
    assert (
        lower(
            "static struct module* f(struct process* pcs)\n{\n    return NULL;\n}",
            name="f",
            ret="*",
            params=[("struct process*", "pcs")],
            static=True,
        )
        is None
    )


def test_unmappable_parameter_type_is_rejected():
    assert (
        lower(
            "DWORD WINAPI F(unsigned long long n)\n{\n    return 0;\n}",
            name="F",
            params=[("unsigned long long", "n")],
        )
        is None
    )


def test_oracle_rejects_a_folded_constant():
    case = dep("DWORD WINAPI F(void)\n{\n    return 0x200;\n}", name="F", params=[("void", "")])
    good = lower_trivial_c_function(case)
    assert good is not None
    tampered = type(good)(
        good.name, good.shape, good.signature, good.rust.replace("0x200", "512"), ()
    )
    findings = verify_trivial_lowering(case, tampered)
    assert any("constants lost" in item for item in findings)
    assert any("constants invented" in item for item in findings)


def test_oracle_rejects_reordered_forwarder_arguments():
    case = dep(
        "INT WINAPI F(AsnAny *dst, AsnAny *src)\n{\n    return copy(dst, src);\n}",
        name="F",
        ret="INT",
        params=[("AsnAny *", "dst"), ("AsnAny *", "src")],
    )
    good = lower_trivial_c_function(case)
    assert good is not None
    swapped = good.rust.replace("copy(dst, src)", "copy(src, dst)")
    tampered = type(good)(good.name, good.shape, good.signature, swapped, ())
    assert any(
        "identifier sequence differs" in item for item in verify_trivial_lowering(case, tampered)
    )


def test_oracle_rejects_an_injected_branch():
    case = dep(
        "DWORD WINAPI F(void)\n{\n    return ERROR_SUCCESS;\n}", name="F", params=[("void", "")]
    )
    good = lower_trivial_c_function(case)
    assert good is not None
    branched = good.rust.replace(
        "    ERROR_SUCCESS\n", "    if true { ERROR_SUCCESS } else { ERROR_SUCCESS }\n"
    )
    tampered = type(good)(good.name, good.shape, good.signature, branched, ())
    assert any("not control-free" in item for item in verify_trivial_lowering(case, tampered))


def test_oracle_rejects_a_dropped_call():
    case = dep(
        "VOID WINAPI F(AsnAny *any)\n{\n    free_any(any);\n}",
        name="F",
        ret="VOID",
        params=[("AsnAny *", "any")],
    )
    good = lower_trivial_c_function(case)
    assert good is not None
    tampered = type(good)(
        good.name, good.shape, good.signature, good.rust.replace("free_any(any);", ""), ()
    )
    findings = verify_trivial_lowering(case, tampered)
    assert any("calls count differs" in item for item in findings)


def test_matcher_oracle_disagreement_fails_loud(monkeypatch):
    import rustconv.trivial_c as trivial_c

    monkeypatch.setattr(trivial_c, "_member", lambda base, op, field: f"(*{base}).Wrong{field}")
    case = dep(
        "ULONG NTAPI G(PADAPTER a)\n{\n    return a->IoBase;\n}",
        name="G",
        ret="ULONG",
        params=[("PADAPTER *", "a")],
    )
    with pytest.raises(TrivialLoweringError, match="failed its own oracle"):
        trivial_c.lower_trivial_c_function(case)


def test_kept_diagnostic_is_re_emitted_as_the_crates_macro():
    result = lower(
        'DWORD WINAPI Stub(INT level)\n{\n    ERR("bad  level %d\\n", level);\n    return 0;\n}',
        name="Stub",
        params=[("INT", "level")],
        keep_diagnostics=True,
    )
    assert result is not None
    assert result.shape == "constant_return"
    # Raw, so the format string's double space survives.
    assert 'ERR!("bad  level %d\\n", level);' in result.rust
    assert "// ERR" not in result.rust


def test_kept_diagnostic_calling_a_helper_goes_to_the_model():
    assert (
        lower(
            'DWORD WINAPI Stub(LPCSTR s)\n{\n    WARN("%s\\n", debugstr_a(s));\n    return 0;\n}',
            name="Stub",
            params=[("LPCSTR", "s")],
            keep_diagnostics=True,
        )
        is None
    )


def test_kept_channel_form_goes_to_the_model():
    assert (
        lower(
            'DWORD WINAPI Stub(void)\n{\n    ERR_(winediag)("nope\\n");\n    return 1;\n}',
            name="Stub",
            keep_diagnostics=True,
        )
        is None
    )


def test_trace_is_still_dropped_when_diagnostics_are_kept():
    result = lower(
        'VOID WINAPI Stub(INT level)\n{\n    TRACE("(%d)\\n", level);\n}',
        name="Stub",
        ret="VOID",
        params=[("INT", "level")],
        keep_diagnostics=True,
    )
    assert result is not None
    assert '// TRACE("(%d)\\n", level);' in result.rust
