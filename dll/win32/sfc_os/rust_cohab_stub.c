/* sfc_os is Rust (sdk/lib/rust/sfc_os-rust); a CMake target needs one TU. */
typedef int sfc_os_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char sfc_os_export_anchor_0 __asm__("_SfcFileException@12");
extern char sfc_os_export_anchor_1 __asm__("_SfcGetNextProtectedFile@8");
extern char sfc_os_export_anchor_2 __asm__("_SfcIsFileProtected@8");
extern char sfc_os_export_anchor_3 __asm__("_SfcIsKeyProtected@12");
extern char sfc_os_export_anchor_4 __asm__("_SfcWLEventLogoff@4");
extern char sfc_os_export_anchor_5 __asm__("_SfcWLEventLogon@4");
__attribute__((used)) static const void *const sfc_os_export_anchors[] = {
    &sfc_os_export_anchor_0,
    &sfc_os_export_anchor_1,
    &sfc_os_export_anchor_2,
    &sfc_os_export_anchor_3,
    &sfc_os_export_anchor_4,
    &sfc_os_export_anchor_5,
    0,
};
#elif defined(__x86_64__)
extern char sfc_os_export_anchor_0 __asm__("SfcFileException");
extern char sfc_os_export_anchor_1 __asm__("SfcGetNextProtectedFile");
extern char sfc_os_export_anchor_2 __asm__("SfcIsFileProtected");
extern char sfc_os_export_anchor_3 __asm__("SfcIsKeyProtected");
extern char sfc_os_export_anchor_4 __asm__("SfcWLEventLogoff");
extern char sfc_os_export_anchor_5 __asm__("SfcWLEventLogon");
__attribute__((used)) static const void *const sfc_os_export_anchors[] = {
    &sfc_os_export_anchor_0,
    &sfc_os_export_anchor_1,
    &sfc_os_export_anchor_2,
    &sfc_os_export_anchor_3,
    &sfc_os_export_anchor_4,
    &sfc_os_export_anchor_5,
    0,
};
#elif defined(__aarch64__)
extern char sfc_os_export_anchor_0 __asm__("SfcFileException");
extern char sfc_os_export_anchor_1 __asm__("SfcGetNextProtectedFile");
extern char sfc_os_export_anchor_2 __asm__("SfcIsFileProtected");
extern char sfc_os_export_anchor_3 __asm__("SfcIsKeyProtected");
extern char sfc_os_export_anchor_4 __asm__("SfcWLEventLogoff");
extern char sfc_os_export_anchor_5 __asm__("SfcWLEventLogon");
__attribute__((used)) static const void *const sfc_os_export_anchors[] = {
    &sfc_os_export_anchor_0,
    &sfc_os_export_anchor_1,
    &sfc_os_export_anchor_2,
    &sfc_os_export_anchor_3,
    &sfc_os_export_anchor_4,
    &sfc_os_export_anchor_5,
    0,
};
#endif
