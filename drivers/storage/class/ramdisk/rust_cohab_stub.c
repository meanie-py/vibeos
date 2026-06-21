/* Cohabitation placeholder TU for the ramdisk kernelmodedriver MODULE.
 * ramdisk.c (the RAM-disk class driver: DriverEntry/AddDevice + bus FDO/drive PDO PnP,
 * Read/Write via mapped pages, IOCTL_DISK_* + FSCTL_CREATE_RAM_DISK, worker thread) is
 * translated to Rust (sdk/lib/rust/ramdisk-rust, libramdisk_rust.a). ramdisk.c is kept on
 * disk per the never-delete rule. ntoskrnl/hal link via add_importlibs. Bar: link-clean x3
 * (NOT boot-loaded in smoke -- the boot ramdisk is freeldr, not this driver).
 */
typedef int ramdisk_rust_cohab_placeholder;
