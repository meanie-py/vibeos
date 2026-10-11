/* Cohabitation placeholder TU for the kdcom kerneldll MODULE.
 *
 * On the non-MSVC (llvm-mingw) build, _WINKD_=FALSE makes this directory
 * build the target named `kdcom` (kdcom.dll) from the 116-LOC stub
 * kdrosdbg.c. That C source has been translated to Rust
 * (sdk/lib/rust/kdcom-rust/src/kdrosdbg.rs, linked as libkdcom_rust.a);
 * it is commented out of the add_library SOURCES below and kept on disk
 * per the never-delete-a-translation rule. All 8 kdrosdbg.spec exports
 * (KdD0Transition, KdD3Transition, KdDebuggerInitialize0/1, KdSendPacket,
 * KdReceivePacket, KdSave, KdRestore) come from the linked Rust staticlib.
 * Mirrors the i8042prt-rust / framebuf-rust / sysfb-rust cohabitation.
 */
typedef int kdcom_rust_cohab_placeholder;
