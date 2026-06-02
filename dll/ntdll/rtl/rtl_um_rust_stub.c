/*
 * rtl_um cohabitation stub.
 *
 * All of rtl_um's translation units (uilist.c / version.c / libsupp.c) have
 * been replaced by Rust (ntdll-rust/src/rtl/*.rs) and commented out of the
 * rtl_um OBJECT library in CMakeLists.txt. Their symbols now come from
 * libntdll_rust.a (linked into ntdll.dll). A CMake OBJECT library needs at
 * least one source to remain valid, so this stub keeps rtl_um well-formed.
 * The .c files stay on disk per the never-delete rule.
 *
 * See docs/engineering/ntdll-translation-audit.md (Phase 2).
 */
const int __rtl_um_rust_cohab_stub = 0;
