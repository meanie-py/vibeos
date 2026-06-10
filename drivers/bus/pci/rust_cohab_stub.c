/*
 * pci.sys Rust cohabitation stub.
 *
 * pci.c / fdo.c / pdo.c (DriverEntry + the device-ID string builders + the FDO
 * and PDO PnP/power dispatch + the BUS_INTERFACE_STANDARD callbacks) are
 * translated to Rust (sdk/lib/rust/pci-rust, kept on disk per the never-delete
 * rule). The MODULE link needs at least one C TU; this placeholder supplies it
 * while the real code comes from libpci_rust.a.
 */
typedef int pci_rust_cohab_placeholder;
