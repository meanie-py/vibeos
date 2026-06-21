/* Cohabitation placeholder TU for the mouhid kernelmodedriver MODULE.
 *
 * The HID mouse client (mouhid.c: DriverEntry +
 * AddDevice + the IRP_MJ_{CREATE,CLOSE,FLUSH_BUFFERS,DEVICE_CONTROL,INTERNAL_DEVICE_CONTROL,
 * POWER,PNP,SYSTEM_CONTROL} dispatch + the continuous read loop + completion routines + the
 * HID-report→MOUSE_INPUT_DATA translation +
 * the class-callback push) is translated to Rust (sdk/lib/rust/mouhid-rust, linked as
 * libmouhid_rust.a). mouhid.c is removed from the add_library SOURCES and kept on disk per the
 * never-delete rule. DriverEntry + the whole driver come from the linked Rust staticlib; the
 * Io/Ke/Ex/Mm/Rtl entry points stay ntoskrnl imports and the HidP_* parser functions stay
 * hidparse imports (add_importlibs(mouhid ntoskrnl hal hidparse)). USE_DUMMY_PSEH is in effect;
 * the Rust translation contains no SEH. mouhid has no GUID data TU, so this stub is the only C
 * source the MODULE needs. mouhid is NOT boot-loaded in the QEMU smoke path, so the bar is
 * link-clean x3 + no smoke regression.
 */
typedef int mouhid_rust_cohab_placeholder;
