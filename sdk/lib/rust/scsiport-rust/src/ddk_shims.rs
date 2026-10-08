// @generated — no-op shims for DDK diagnostic macros the C uses as statements
// (PAGED_CODE(), UNREFERENCED_PARAMETER(x), UNIMPLEMENTED*, ASSERT_*). These are
// header macros, not link exports; the translation rendered them as fn calls, so we
// provide matching no-op fns. Generic over arg type so any expression is accepted.
#![allow(non_snake_case, unused_variables)]
#[inline(always)] pub fn UNIMPLEMENTED_DBGBREAK() {}
#[inline(always)] pub fn UNREFERENCED_PARAMETER<T>(_: T) {}
#[inline(always)] pub fn ASSERT_FDO<T>(_: T) {}
#[inline(always)] pub fn ASSERT_PDO<T>(_: T) {}
#[inline(always)] pub fn ASSERTMSG<A, B>(_: A, _: B) {}
#[inline(always)] pub fn DBG_UNREFERENCED_LOCAL_VARIABLE<T>(_: T) {}
// RtlAssert — NDEBUG-dead assertion handler. The modules declared it with conflicting
// signatures/arg-orders; one generic no-op shim matches release semantics (ASSERT/
// PAGED_CODE compile out under NDEBUG). REVIEW: assertion-failure path is now silent.
#[inline(always)] pub unsafe fn RtlAssert<A, B, L, C>(_a: A, _b: B, _line: L, _msg: C) {}
