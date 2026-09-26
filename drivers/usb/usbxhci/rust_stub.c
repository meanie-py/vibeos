/* The usbxhci MODULE is entirely Rust (sdk/lib/rust/usbxhci-rust, libusbxhci_rust.a).
 * add_library needs at least one C translation unit; this is it. There is no C
 * implementation of this driver: it was written in Rust from the start.
 */
typedef int usbxhci_rust_placeholder;
