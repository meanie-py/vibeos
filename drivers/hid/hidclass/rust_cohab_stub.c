/* Cohabitation placeholder TU for the hidclass kernelmodedriver MODULE.
 *
 * The HID FDO/PDO class driver (fdo.c/hidclass.c/pdo.c/guid.c: DllInitialize, the exported
 * HidRegisterMinidriver, AddDevice, the IRP_MJ_* dispatch + create/close fileop, the
 * per-collection PDO creation, the FDO/PDO PnP descriptor-fetch state machine, and the
 * IO_WORKITEM read-report completion path) is translated to Rust (sdk/lib/rust/hidclass-rust,
 * linked as libhidclass_rust.a). The four .c bodies are removed from the add_library SOURCES
 * and kept on disk per the never-delete rule. spec2def still emits hidclass.def (exports
 * satisfied by the Rust #[no_mangle] symbols) + the import lib hidusb links. Io/Ke/Ex/Mm/Rtl
 * stay ntoskrnl imports; HidP_* stay hidparse imports. (HidNotifyPresence is declared in
 * hidport.h but has NO implementation anywhere in ReactOS and is not a .spec export, so it is
 * intentionally not provided.) ENTRYPOINT 0; this stub is the only C TU the MODULE needs.
 * hidclass is NOT boot-loaded in the QEMU smoke path, so the bar is link-clean x3 + no smoke
 * regression.
 */
typedef int hidclass_rust_cohab_placeholder;
