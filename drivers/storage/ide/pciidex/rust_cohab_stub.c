/* Cohabitation placeholder TU for the pciidex.sys kernelmodedriver MODULE.
 *
 * The PCI IDE bus extension (fdo.c/miniport.c/pciidex.c/pdo.c/power.c — DriverEntry,
 * the PnP FDO/PDO dispatch, the miniport bus-data shims, and the pciidex.spec exports
 * PciIdeXInitialize/GetBusData/SetBusData/DebugPrint) is translated to Rust
 * (sdk/lib/rust/pciidex-rust, linked as libpciidex_rust.a). The 5 .c bodies are removed
 * from the add_library SOURCES and kept on disk per the never-delete rule. The whole
 * driver + the spec exports come from the linked Rust staticlib; the Io/Ke/Zw/Ex/Mm/Hal
 * entry points stay ntoskrnl/hal imports (add_importlibs(pciidex ntoskrnl hal)). The
 * wdmguid.h GUIDs (GUID_BUS_INTERFACE_STANDARD / GUID_TRANSLATOR_INTERFACE_STANDARD) are
 * materialized as module-private statics inside the Rust crate (the c2rust anchor
 * inlined initguid.h), so there is no GUID data TU to keep — this stub is the only C
 * source the MODULE needs. DeepSeek+c2rust translation.
 */
typedef int pciidex_rust_cohab_placeholder;
