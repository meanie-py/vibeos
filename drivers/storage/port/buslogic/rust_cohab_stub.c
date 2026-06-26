/* Cohabitation placeholder TU for the buslogic kernelmodedriver MODULE.
 * The BusLogic BT-958 SCSI HBA miniport is translated to Rust
 * (sdk/lib/rust/buslogic-rust, libbuslogic_rust.a). The single .c body
 * (BusLogic958.c) is replaced by Rust and kept on disk per the never-delete
 * rule. #[no_mangle] DriverEntry in the rlib takes over the link; the driver
 * has no exports (it imports ScsiPort* and registers via ScsiPortInitialize).
 * Bar: link-clean x3 (NOT boot-loaded in smoke -- the live SCSI/IDE path is
 * uniata.sys, not this legacy BusLogic HBA).
 */
typedef int buslogic_rust_cohab_placeholder;
