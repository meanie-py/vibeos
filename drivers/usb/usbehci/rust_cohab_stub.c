/* Cohabitation placeholder TU for the usbehci kernelmodedriver MODULE.
 * USB host-controller miniport translated to Rust (sdk/lib/rust/usbehci-rust, libusbehci_rust.a).
 * The .c bodies are kept on disk per the never-delete rule (debug/roothub/usbehci.c). guid.c
 * stays compiled as a data-only TU (DEFINE_GUID). Imports link via add_importlibs.
 * Bar: link-clean x3 (NOT boot-loaded in smoke -- boots from CD/ramdisk, not USB).
 */
typedef int usbehci_rust_cohab_placeholder;
