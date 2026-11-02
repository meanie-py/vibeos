/* wow64 is Rust (sdk/lib/rust/wow64-rust); a CMake target needs one TU. */
typedef int wow64_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
__attribute__((used)) static const void *const wow64_export_anchors[] = {
    0,
};
#elif defined(__x86_64__)
extern char wow64_export_anchor_0 __asm__("Wow64LdrpInitialize");
extern char wow64_export_anchor_1 __asm__("Wow64ProcessPendingCrossProcessItems");
extern char wow64_export_anchor_2 __asm__("Wow64StartGuestThreadFrom64");
extern char wow64_export_anchor_3 __asm__("Wow64SystemServiceEx");
__attribute__((used)) static const void *const wow64_export_anchors[] = {
    &wow64_export_anchor_0,
    &wow64_export_anchor_1,
    &wow64_export_anchor_2,
    &wow64_export_anchor_3,
    0,
};
#elif defined(__aarch64__)
extern char wow64_export_anchor_0 __asm__("Wow64LdrpInitialize");
extern char wow64_export_anchor_1 __asm__("Wow64ProcessPendingCrossProcessItems");
extern char wow64_export_anchor_2 __asm__("Wow64StartGuestThreadFrom64");
extern char wow64_export_anchor_3 __asm__("Wow64SystemServiceEx");
__attribute__((used)) static const void *const wow64_export_anchors[] = {
    &wow64_export_anchor_0,
    &wow64_export_anchor_1,
    &wow64_export_anchor_2,
    &wow64_export_anchor_3,
    0,
};
#endif
