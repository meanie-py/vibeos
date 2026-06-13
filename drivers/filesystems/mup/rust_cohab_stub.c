/* Cohabitation placeholder TU for the mup kernelmodedriver MODULE.
 *
 * The Multiple UNC Provider (mup.c DriverEntry + the IRP dispatch routines + the prefix-
 * table / provider-registration / query-path machinery; dfs.c DFS stubs) is translated to
 * Rust (sdk/lib/rust/mup-rust, linked as libmup_rust.a). The two .c bodies are removed from
 * the add_library SOURCES and kept on disk per the never-delete rule. DriverEntry + the
 * whole driver come from the linked Rust staticlib; the Io/Ke/Zw/Nt/Ex/Mm/Ob/FsRtl/Rtl
 * entry points stay ntoskrnl imports (add_importlibs(mup ntoskrnl hal)). cmake keeps the
 * MODULE link (set_module_type kernelmodedriver). mup has no GUID data TU, so this stub is
 * the only C source the MODULE needs. mup is NOT boot-loaded (no txtsetup.sif entry), so
 * the bar is link-clean x3 + no smoke regression. Hand translation (not LLM).
 */
typedef int mup_rust_cohab_placeholder;
