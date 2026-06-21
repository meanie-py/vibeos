/* Cohabitation placeholder TU for the msfs kernelmodedriver MODULE.
 *
 * The Mailslot filesystem (msfs.c DriverEntry + the IRP_MJ_{CREATE,CREATE_MAILSLOT,CLOSE,
 * READ,WRITE,QUERY_INFORMATION,SET_INFORMATION,FILE_SYSTEM_CONTROL} dispatch routines in
 * create.c/rw.c/finfo.c/fsctrl.c + the IO_CSQ callbacks & timeout DPC in msfssup.c) is
 * translated to Rust (sdk/lib/rust/msfs-rust, linked as libmsfs_rust.a). The six .c bodies
 * are removed from the add_library SOURCES and kept on disk per the never-delete rule.
 * DriverEntry + the whole driver come from the linked Rust staticlib; the Io/Ke/Ex/Mm/Rtl
 * entry points stay ntoskrnl imports (add_importlibs(msfs ntoskrnl hal)). cmake keeps the
 * MODULE link (set_module_type kernelmodedriver). msfs has no GUID data TU, so this stub is
 * the only C source the MODULE needs. msfs is NOT boot-loaded (no 'x' in txtsetup.sif), so
 * the bar is link-clean x3 + no smoke regression. Hand translation (not LLM).
 */
typedef int msfs_rust_cohab_placeholder;
