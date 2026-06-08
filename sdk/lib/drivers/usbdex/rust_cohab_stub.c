/* Cohabitation placeholder TU for the usbdex static library.
 *
 * usbdex.c (the "UsbdEx lib": USBD_QueryUsbCapability / _CreateHandle /
 * _CloseHandle / _UrbAllocate / _IsochUrbAllocate / _UrbFree /
 * _AssignUrbToIoStackLocation) is translated to Rust
 * (sdk/lib/rust/usbdex-rust, linked as libusbdex_rust.a). usbdex.c is removed
 * from the add_library SOURCES and kept on disk per the never-delete rule.
 * All 7 symbols come from the linked Rust staticlib (cdecl — usbdlib.h has no
 * NTAPI on these). Mirrors the cportlib-rust static-lib cohabitation.
 */
typedef int usbdex_rust_cohab_placeholder;
