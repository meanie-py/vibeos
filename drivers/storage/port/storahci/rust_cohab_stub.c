/* Cohabitation placeholder TU for the storahci kernelmodedriver MODULE.
 * storahci.c (the AHCI storport miniport: HwFindAdapter/HwInitialize/HwStartIo/HwInterrupt +
 * DPC completion, PRDT build, ATA/ATAPI CFIS, port activation) is translated to Rust
 * (sdk/lib/rust/storahci-rust, libstorahci_rust.a). storahci.c is kept on disk per the
 * never-delete rule. StorPort* link from storport.sys (add_importlibs storport ntoskrnl hal).
 * storahci stays DISABLED from shipping (no add_cd_file); bar is link-clean x3.
 */
typedef int storahci_rust_cohab_placeholder;
