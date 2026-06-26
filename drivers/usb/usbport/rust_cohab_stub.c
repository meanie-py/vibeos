/* Cohabitation placeholder TU for the usbport kernelmodedriver MODULE.
 * The USB host-controller PORT framework is translated to Rust
 * (sdk/lib/rust/usbport-rust, libusbport_rust.a). All 15 .c bodies (debug/device/
 * endpoint/iface/ioctl/iso/pnp/power/queue/roothub/trfsplit/urb/usb2/usbport) are
 * replaced by Rust and kept on disk per the never-delete rule. guid.c stays compiled
 * as a data-only TU (DEFINE_GUID). The two exports USBPORT_GetHciMn /
 * USBPORT_RegisterUSBPortDriver are #[no_mangle] extern "system" fns in the rlib;
 * spec2def(usbport.spec) emits usbport.def so the linker exports them.
 * Bar: link-clean x3 (NOT boot-loaded in smoke -- boots from CD/ramdisk, not USB).
 */
typedef int usbport_rust_cohab_placeholder;
