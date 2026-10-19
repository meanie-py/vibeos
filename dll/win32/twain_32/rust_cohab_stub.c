/* twain_32 is Rust (sdk/lib/rust/twain_32-rust); a CMake target needs one TU. */
typedef int twain_32_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char twain_32_export_anchor_0 __asm__("_DSM_Entry@24");
__attribute__((used)) static const void *const twain_32_export_anchors[] = {
    &twain_32_export_anchor_0,
    0,
};
#elif defined(__x86_64__)
extern char twain_32_export_anchor_0 __asm__("DSM_Entry");
__attribute__((used)) static const void *const twain_32_export_anchors[] = {
    &twain_32_export_anchor_0,
    0,
};
#elif defined(__aarch64__)
extern char twain_32_export_anchor_0 __asm__("DSM_Entry");
__attribute__((used)) static const void *const twain_32_export_anchors[] = {
    &twain_32_export_anchor_0,
    0,
};
#endif
