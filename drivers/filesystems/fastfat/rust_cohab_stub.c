/* Cohabitation placeholder TU for the fastfat kernelmodedriver MODULE.
 *
 * The FAT filesystem driver (all 34 .c TUs: fatinit.c DriverEntry + unload,
 * the FSD dispatch/FSP queue (fspdisp.c workque.c), create/close/cleanup/read/
 * write paths, allocsup/dirsup/easup/strucsup/cachesup/deviosup support layers,
 * fsctrl mount/verify, fileinfo/volinfo queries, lockctrl/oplocks, pnp/shutdown,
 * and the splay/name/time/verify helpers) is translated to Rust
 * (sdk/lib/rust/fastfat-rust, linked as libfastfat_rust.a). The 34 .c bodies are
 * removed from the add_library SOURCES and kept on disk per the never-delete rule.
 * DriverEntry + the whole driver come from the linked Rust staticlib; the
 * Io/Ke/Zw/Ex/Mm/Cc/FsRtl entry points stay ntoskrnl imports
 * (add_importlibs(fastfat ntoskrnl hal)). cmake keeps the MODULE link
 * (set_module_type kernelmodedriver). fastfat has no GUID data TU, so this stub
 * is the only C source the MODULE needs. DeepSeek+c2rust translation.
 */
typedef int fastfat_rust_cohab_placeholder;
