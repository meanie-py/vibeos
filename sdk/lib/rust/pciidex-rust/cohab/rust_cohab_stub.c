/* Cohabitation placeholder TU for the pciidex.sys kernelmodedriver MODULE.
 *
 * pciidex.sys is upstream's 2026 PCI IDE bus extension: the controller/channel FDO+PDO core
 * (fdo/pdo/pciidex/miniport/power/acpi.c) and the chipset/ layer, including a software AHCI
 * HBA driver. All of it is translated to Rust in sdk/lib/rust/pciidex-rust and linked as
 * libpciidex_rust.a; the .c files stay on disk, out of the build. The pciidex.spec exports
 * come from the Rust #[no_mangle] symbols, except the variadic PciIdeXDebugPrint (see
 * rust_varargs_shim.c). The pre-rewrite translation is parked as pciidex-legacy-rust.
 */
typedef int pciidex_rust_cohab_placeholder;
