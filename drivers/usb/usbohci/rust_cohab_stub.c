/* Cohabitation placeholder TU for the usbohci kernelmodedriver MODULE.
 * USB host-controller miniport translated to Rust (sdk/lib/rust/usbohci-rust, libusbohci_rust.a).
 * The .c bodies are kept on disk per the never-delete rule (debug/roothub/usbohci.c). guid.c
 * stays compiled as a data-only TU (DEFINE_GUID). Imports link via add_importlibs.
 * Bar: link-clean x3 (NOT boot-loaded in smoke -- boots from CD/ramdisk, not USB).
 */
typedef int usbohci_rust_cohab_placeholder;
