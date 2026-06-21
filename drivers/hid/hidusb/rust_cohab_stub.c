/* Cohabitation placeholder TU for the hidusb kernelmodedriver MODULE.
 *
 * The USB HID minidriver (hidusb.c: DriverEntry/AddDevice, the async URB build/submit path
 * (Hid_DispatchUrb event-wait), the IoAllocateWorkItem pipe/port RESET error-recovery, the
 * multi-step USB descriptor acquisition, and HidUsb_ReadReport + completion) is translated to
 * Rust (sdk/lib/rust/hidusb-rust, linked as libhidusb_rust.a). hidusb.c is removed from the
 * add_library SOURCES and kept on disk per the never-delete rule. It calls HidRegisterMinidriver
 * (hidclass import) + USBD_CreateConfigurationRequestEx/USBD_ParseConfigurationDescriptorEx
 * (usbd import); the four UsbBuild* usbdlib macros are inlined in the Rust. Io/Ke/Ex/Mm/Rtl
 * stay ntoskrnl imports. This stub is the only C TU the MODULE needs. hidusb is NOT boot-loaded
 * in the QEMU smoke path, so the bar is link-clean x3 + no smoke regression.
 */
typedef int hidusb_rust_cohab_placeholder;
