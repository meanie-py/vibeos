/* sfc is Rust (sdk/lib/rust/sfc-rust); a CMake target needs one TU. */
typedef int sfc_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char sfc_export_anchor_0 __asm__("_SRSetRestorePointA@8");
extern char sfc_export_anchor_1 __asm__("_SRSetRestorePointW@8");
extern char sfc_export_anchor_2 __asm__("_SfpVerifyFile@12");
extern char sfc_export_anchor_3 __asm__("_sfc_8@0");
extern char sfc_export_anchor_4 __asm__("_sfc_9@0");
__attribute__((used)) static const void *const sfc_export_anchors[] = {
    &sfc_export_anchor_0,
    &sfc_export_anchor_1,
    &sfc_export_anchor_2,
    &sfc_export_anchor_3,
    &sfc_export_anchor_4,
    0,
};
#elif defined(__x86_64__)
extern char sfc_export_anchor_0 __asm__("SRSetRestorePointA");
extern char sfc_export_anchor_1 __asm__("SRSetRestorePointW");
extern char sfc_export_anchor_2 __asm__("SfpVerifyFile");
extern char sfc_export_anchor_3 __asm__("sfc_8");
extern char sfc_export_anchor_4 __asm__("sfc_9");
__attribute__((used)) static const void *const sfc_export_anchors[] = {
    &sfc_export_anchor_0,
    &sfc_export_anchor_1,
    &sfc_export_anchor_2,
    &sfc_export_anchor_3,
    &sfc_export_anchor_4,
    0,
};
#elif defined(__aarch64__)
extern char sfc_export_anchor_0 __asm__("SRSetRestorePointA");
extern char sfc_export_anchor_1 __asm__("SRSetRestorePointW");
extern char sfc_export_anchor_2 __asm__("SfpVerifyFile");
extern char sfc_export_anchor_3 __asm__("sfc_8");
extern char sfc_export_anchor_4 __asm__("sfc_9");
__attribute__((used)) static const void *const sfc_export_anchors[] = {
    &sfc_export_anchor_0,
    &sfc_export_anchor_1,
    &sfc_export_anchor_2,
    &sfc_export_anchor_3,
    &sfc_export_anchor_4,
    0,
};
#endif
