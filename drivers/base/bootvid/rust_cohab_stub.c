/* Cohabitation placeholder TU for the bootvid kerneldll MODULE.
 *
 * Post-BIOS-prune, bootvid.dll is the generic linear-framebuffer (GOP)
 * driver: framebuf/bootvid.c (which #includes the shared ARC-tree impl
 * <drivers/bootvid/framebuf.c>) + the shared common.c/console.c/fontdata.c.
 * All of that is translated to Rust (sdk/lib/rust/bootvid-rust, linked as
 * libbootvid_rust.a); the C sources are commented out of SOURCE and kept on
 * disk per the never-delete-a-translation rule. All 11 bootvid.spec exports
 * (Vid*) come from the linked Rust staticlib. Mirrors the i8042prt-rust /
 * kdcom-rust / framebuf-rust / sysfb-rust cohabitation.
 */
typedef int bootvid_rust_cohab_placeholder;
