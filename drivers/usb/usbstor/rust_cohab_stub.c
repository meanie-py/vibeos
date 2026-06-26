/* Cohabitation placeholder TU for the usbstor kernelmodedriver MODULE.
 * The USB mass-storage class driver (descriptor/disk/error/fdo/misc/pdo/queue/scsi/usbstor +
 * guid: DriverEntry/AddDevice, USB Bulk-Only Transport CBW/CSW, FDO/PDO PnP, SCSI command
 * translation over URBs) is translated to Rust (sdk/lib/rust/usbstor-rust, libusbstor_rust.a).
 * The .c files are kept on disk per the never-delete rule. ntoskrnl/hal/usbd link via
 * add_importlibs; sptilib (SCSI pass-through helper) + PSEH still link as C. Bar: link-clean x3
 * (NOT boot-loaded in smoke -- the smoke boots from CD/ramdisk, not USB mass storage).
 */
typedef int usbstor_rust_cohab_placeholder;
