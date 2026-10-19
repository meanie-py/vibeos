/* ntobjshex is Rust (sdk/lib/rust/ntobjshex-rust); a CMake target needs one TU. */
typedef int ntobjshex_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char ntobjshex_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char ntobjshex_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char ntobjshex_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char ntobjshex_export_anchor_3 __asm__("_DllUnregisterServer@0");
__attribute__((used)) static const void *const ntobjshex_export_anchors[] = {
    &ntobjshex_export_anchor_0,
    &ntobjshex_export_anchor_1,
    &ntobjshex_export_anchor_2,
    &ntobjshex_export_anchor_3,
    0,
};
#elif defined(__x86_64__)
extern char ntobjshex_export_anchor_0 __asm__("DllCanUnloadNow");
extern char ntobjshex_export_anchor_1 __asm__("DllGetClassObject");
extern char ntobjshex_export_anchor_2 __asm__("DllRegisterServer");
extern char ntobjshex_export_anchor_3 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const ntobjshex_export_anchors[] = {
    &ntobjshex_export_anchor_0,
    &ntobjshex_export_anchor_1,
    &ntobjshex_export_anchor_2,
    &ntobjshex_export_anchor_3,
    0,
};
#elif defined(__aarch64__)
extern char ntobjshex_export_anchor_0 __asm__("DllCanUnloadNow");
extern char ntobjshex_export_anchor_1 __asm__("DllGetClassObject");
extern char ntobjshex_export_anchor_2 __asm__("DllRegisterServer");
extern char ntobjshex_export_anchor_3 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const ntobjshex_export_anchors[] = {
    &ntobjshex_export_anchor_0,
    &ntobjshex_export_anchor_1,
    &ntobjshex_export_anchor_2,
    &ntobjshex_export_anchor_3,
    0,
};
#endif
