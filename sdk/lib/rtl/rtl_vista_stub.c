/*
 * rtl_vista_stub.c — placeholder TU for the `rtl_vista` static library.
 *
 * The Vista RTL synchronization sources (condvar.c / runonce.c / srw.c /
 * locale.c) were translated to Rust (sdk/lib/rust/rtl-sync/). The shipping
 * ntdll_vista.dll now links them via reactos-link `.rust_lib("rtl-sync",
 * ["vista"])` (see dll/ntdll/nt_0600/ntdll_vista-cargo/build.rs); the other
 * consumer, rtl_unittest, resolves RtlInitializeSRWLock / *ConditionVariable /
 * RtlRunOnce* / RtlLcidToLocaleName from the main `rtl` library (rtl-rust →
 * rtl-sync `cabi`), which precedes rtl_vista on its link line.
 *
 * `add_library` needs at least one source, so this empty TU keeps the
 * `rtl_vista` target valid without re-introducing any C synchronization code.
 */

/* Keep the translation unit non-empty for archivers that reject empty objects. */
const int __rtl_vista_rust_placeholder = 0;
