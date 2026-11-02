/* ksuser is Rust (sdk/lib/rust/ksuser-rust); a CMake target needs one TU. */
typedef int ksuser_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char ksuser_export_anchor_0 __asm__("_KsCreateAllocator@12");
extern char ksuser_export_anchor_1 __asm__("_KsCreateClock@12");
extern char ksuser_export_anchor_2 __asm__("_KsCreatePin@16");
extern char ksuser_export_anchor_3 __asm__("_KsCreateTopologyNode@16");
__attribute__((used)) static const void *const ksuser_export_anchors[] = {
    &ksuser_export_anchor_0,
    &ksuser_export_anchor_1,
    &ksuser_export_anchor_2,
    &ksuser_export_anchor_3,
    0,
};
#elif defined(__x86_64__)
extern char ksuser_export_anchor_0 __asm__("KsCreateAllocator");
extern char ksuser_export_anchor_1 __asm__("KsCreateClock");
extern char ksuser_export_anchor_2 __asm__("KsCreatePin");
extern char ksuser_export_anchor_3 __asm__("KsCreateTopologyNode");
__attribute__((used)) static const void *const ksuser_export_anchors[] = {
    &ksuser_export_anchor_0,
    &ksuser_export_anchor_1,
    &ksuser_export_anchor_2,
    &ksuser_export_anchor_3,
    0,
};
#elif defined(__aarch64__)
extern char ksuser_export_anchor_0 __asm__("KsCreateAllocator");
extern char ksuser_export_anchor_1 __asm__("KsCreateClock");
extern char ksuser_export_anchor_2 __asm__("KsCreatePin");
extern char ksuser_export_anchor_3 __asm__("KsCreateTopologyNode");
__attribute__((used)) static const void *const ksuser_export_anchors[] = {
    &ksuser_export_anchor_0,
    &ksuser_export_anchor_1,
    &ksuser_export_anchor_2,
    &ksuser_export_anchor_3,
    0,
};
#endif
