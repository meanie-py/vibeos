/* Cohabitation placeholder TU for the usbd kernelmodedriver MODULE.
 *
 * usbd.c (the USBD library driver: DriverEntry/DllInitialize + the USBD_*
 * descriptor parsers / bandwidth / registry helpers) is translated to Rust
 * (sdk/lib/rust/usbd-rust, linked as libusbd_rust.a). usbd.c is removed from
 * the add_library SOURCES and kept on disk per the never-delete rule. The
 * usbd.spec exports (USBD_*) + DriverEntry come from the linked Rust staticlib;
 * the USB DDK types it needs come from usb-sys (bindgen of usbdi.h/usbdlib.h).
 * cmake keeps the MODULE link (set_module_type kernelmodedriver + spec2def .def
 * + add_importlibs(ntoskrnl hal)). DeepSeek+c2rust translation (Batch 2).
 */
typedef int usbd_rust_cohab_placeholder;
