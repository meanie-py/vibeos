/* Cohabitation placeholder TU for the i8042prt kernelmodedriver MODULE.
 *
 * All 10 C sources (createclose.c hwhacks.c i8042prt.c keyboard.c misc.c
 * mouse.c pnp.c ps2pp.c readwrite.c registry.c) have been translated to Rust
 * (sdk/lib/rust/i8042prt-rust, linked as libi8042prt_rust.a). They are
 * commented out of the add_library SOURCES below; the .c files are kept on
 * disk per the never-delete-a-translation rule. All real symbols — DriverEntry
 * and the whole driver — come from the linked Rust staticlib. guid.c stays C
 * (it only emits the poclass GUID definitions via initguid.h and must NOT use
 * the PCH). Mirrors the arm64input-rust / hal-rust cohabitation.
 */
typedef int i8042prt_rust_cohab_placeholder;
