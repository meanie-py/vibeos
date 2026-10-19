/* msrle32 is Rust (sdk/lib/rust/msrle32-rust); a CMake target needs one TU. */
typedef int msrle32_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char msrle32_export_anchor_0 __asm__("_MSRLE32_DriverProc@20");
__attribute__((used)) static const void *const msrle32_export_anchors[] = {
    &msrle32_export_anchor_0,
    0,
};
#elif defined(__x86_64__)
extern char msrle32_export_anchor_0 __asm__("MSRLE32_DriverProc");
__attribute__((used)) static const void *const msrle32_export_anchors[] = {
    &msrle32_export_anchor_0,
    0,
};
#elif defined(__aarch64__)
extern char msrle32_export_anchor_0 __asm__("MSRLE32_DriverProc");
__attribute__((used)) static const void *const msrle32_export_anchors[] = {
    &msrle32_export_anchor_0,
    0,
};
#endif
