/* Cohabitation placeholder TU for the fs_rec kernelmodedriver MODULE.
 *
 * The filesystem recognizer (fs_rec.c DriverEntry + FsRecRegisterFs/FsRecLoadFileSystem
 * + the dispatch routines; blockdev.c device I/O helpers; and one sniffer per FS type:
 * fat/fatx/cdfs/ntfs/udfs/ext/btrfs/reiserfs/ffs.c) is translated to Rust
 * (sdk/lib/rust/fs_rec-rust, linked as libfs_rec_rust.a). The 11 .c bodies are removed
 * from the add_library SOURCES and kept on disk per the never-delete rule. DriverEntry +
 * the whole driver come from the linked Rust staticlib; the Io/Ke/Zw/Ex/Mm/FsRtl entry
 * points stay ntoskrnl imports (add_importlibs(fs_rec ntoskrnl hal)). cmake keeps the
 * MODULE link (set_module_type kernelmodedriver). fs_rec has no GUID data TU, so this
 * stub is the only C source the MODULE needs. DeepSeek+c2rust translation.
 */
typedef int fs_rec_rust_cohab_placeholder;
