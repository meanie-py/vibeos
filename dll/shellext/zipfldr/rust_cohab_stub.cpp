/* zipfldr is Rust (sdk/lib/rust/zipfldr-rust); a CMake target needs one TU. */
typedef int zipfldr_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char zipfldr_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char zipfldr_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char zipfldr_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char zipfldr_export_anchor_3 __asm__("_DllUnregisterServer@0");
extern char zipfldr_export_anchor_4 __asm__("_RouteTheCallW@16");
__attribute__((used)) static const void *const zipfldr_export_anchors[] = {
    &zipfldr_export_anchor_0,
    &zipfldr_export_anchor_1,
    &zipfldr_export_anchor_2,
    &zipfldr_export_anchor_3,
    &zipfldr_export_anchor_4,
    0,
};
#elif defined(__x86_64__)
extern char zipfldr_export_anchor_0 __asm__("DllCanUnloadNow");
extern char zipfldr_export_anchor_1 __asm__("DllGetClassObject");
extern char zipfldr_export_anchor_2 __asm__("DllRegisterServer");
extern char zipfldr_export_anchor_3 __asm__("DllUnregisterServer");
extern char zipfldr_export_anchor_4 __asm__("RouteTheCallW");
__attribute__((used)) static const void *const zipfldr_export_anchors[] = {
    &zipfldr_export_anchor_0,
    &zipfldr_export_anchor_1,
    &zipfldr_export_anchor_2,
    &zipfldr_export_anchor_3,
    &zipfldr_export_anchor_4,
    0,
};
#elif defined(__aarch64__)
extern char zipfldr_export_anchor_0 __asm__("DllCanUnloadNow");
extern char zipfldr_export_anchor_1 __asm__("DllGetClassObject");
extern char zipfldr_export_anchor_2 __asm__("DllRegisterServer");
extern char zipfldr_export_anchor_3 __asm__("DllUnregisterServer");
extern char zipfldr_export_anchor_4 __asm__("RouteTheCallW");
__attribute__((used)) static const void *const zipfldr_export_anchors[] = {
    &zipfldr_export_anchor_0,
    &zipfldr_export_anchor_1,
    &zipfldr_export_anchor_2,
    &zipfldr_export_anchor_3,
    &zipfldr_export_anchor_4,
    0,
};
#endif
