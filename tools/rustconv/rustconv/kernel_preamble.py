"""Derive the exact, bounded PC_ASSERT preamble without a model call."""

import re

from .lex_mask import mask_code

_ASSERT = r"""#define PC_ASSERT(exp) \
 (VOID)((!(exp)) ? \
 RtlAssert((PVOID) #exp, (PVOID)__FILE__, __LINE__, NULL ), FALSE : TRUE)"""
_IRQL = "#define PC_ASSERT_IRQL_EQUAL(x) PC_ASSERT(KeGetCurrentIrql()==(x))"


def _normalize(source: str) -> str:
    return re.sub(r"\s+", "", mask_code(source.replace("\\\n", "")))


def derive_kernel_preamble(source: str) -> str | None:
    """None means unsupported, never 'safe to omit'. Only these complete macros match."""
    stripped = re.sub(
        r"^\s*#\s*(?:include\b[^\n]*|pragma\s+once|define\s+NDEBUG)\s*$", "", source, flags=re.M
    )
    if _normalize(stripped) != _normalize(_ASSERT + "\n" + _IRQL):
        return None
    return r"""// Source-derived PC_ASSERT: diagnostic call and expression evaluation retained.
macro_rules! PC_ASSERT {
    ($condition:expr) => {{
        if !$condition {
            ntkrnl_sys::ddk::RtlAssert(
                concat!(stringify!($condition), "\0").as_ptr() as _,
                concat!(file!(), "\0").as_ptr() as _, line!(), core::ptr::null_mut());
        }
    }};
}
macro_rules! PC_ASSERT_IRQL_EQUAL {
    ($level:expr) => {
        PC_ASSERT!((ntkrnl_sys::ke::KeGetCurrentIrql() as i32) == ($level as i32));
    };
}
"""
