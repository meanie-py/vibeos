/* Cohabitation placeholder TU for the scsiport.sys kernelmodedriver MODULE.
 *
 * The SCSI port driver (fdo.c/ioctl.c/pdo.c/power.c/registry.c/scsi.c/scsiport.c/
 * stubs.c — DriverEntry, the PnP FDO/PDO dispatch, the SCSI dispatch + StartIo +
 * ISR/DPC machinery, and the ~45 scsiport.spec ScsiPort* exports) is translated to
 * Rust (sdk/lib/rust/scsiport-rust, linked as libscsiport_rust.a). The 8 .c bodies
 * are removed from the add_library SOURCES and kept on disk per the never-delete
 * rule. guid.c stays as the C TU: it is the DEFINE_GUID data home (initguid.h) for
 * GUID_DEVINTERFACE_STORAGEPORT, which the Rust crate references as an extern.
 * ScsiPortNotification/ScsiDebugPrint are cdecl varargs (c_variadic feature ->
 * cargo build runs under RUSTC_BOOTSTRAP=1). sptilib stays a C static lib.
 * DeepSeek+c2rust translation.
 */
typedef int scsiport_rust_cohab_placeholder;
