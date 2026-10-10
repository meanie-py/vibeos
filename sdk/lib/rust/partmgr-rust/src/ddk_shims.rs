// @generated — no-op shims for DDK diagnostic macros the C uses as statements
// (PAGED_CODE(), UNREFERENCED_PARAMETER(x), UNIMPLEMENTED*, ASSERT_*). These are
// header macros, not link exports; the translation rendered them as fn calls, so we
// provide matching no-op fns. Generic over arg type so any expression is accepted.
#![allow(non_snake_case, unused_variables)]
#[inline(always)] pub fn UNIMPLEMENTED_DBGBREAK() {}
#[inline(always)] pub fn ASSERT_FDO<T>(_: T) {}
#[inline(always)] pub fn ASSERT_PDO<T>(_: T) {}
#[inline(always)] pub fn ASSERTMSG<A, B>(_: A, _: B) {}
