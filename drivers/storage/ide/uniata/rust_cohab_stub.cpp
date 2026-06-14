/* Cohabitation placeholder TU for the uniata.sys kernelmodedriver MODULE.
 *
 * The UniATA IDE/SATA/AHCI miniport (atacmd_map.cpp/bm_devs.cpp/id_ata.cpp/
 * id_badblock.cpp/id_dma.cpp/id_init.cpp/id_probe.cpp/id_queue.cpp/id_sata.cpp —
 * DriverEntry, the SCSIPORT HwInitialize/HwFindAdapter/HwStartIo/HwInterrupt
 * miniport machinery, the PCI bus-master/AHCI/SATA register stacks, the chip
 * detection tables and the bad-block list) is translated to Rust
 * (sdk/lib/rust/uniata-rust, linked as libuniata_rust.a). The 9 .cpp bodies are
 * removed from the add_library SOURCES and kept on disk per the never-delete rule.
 * ros_glue/ros_glue.cpp stays as the C TU: it is the data home for the mutable
 * MajorVersion/MinorVersion/BuildNumber version globals, which the Rust crate
 * references (and DriverEntry writes) as externs.  First C++-tier translation,
 * riding the rustconv tree-sitter-cpp front-end.
 */
typedef int uniata_rust_cohab_placeholder;
