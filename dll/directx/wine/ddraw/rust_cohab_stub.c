/* ddraw is Rust (sdk/lib/rust/ddraw-rust); a CMake target needs one TU. */
typedef int ddraw_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char ddraw_export_anchor_0 __asm__("_DirectDrawCreate@12");
extern char ddraw_export_anchor_1 __asm__("_DirectDrawCreateClipper@12");
extern char ddraw_export_anchor_2 __asm__("_DirectDrawCreateEx@16");
extern char ddraw_export_anchor_3 __asm__("_DirectDrawEnumerateA@8");
extern char ddraw_export_anchor_4 __asm__("_DirectDrawEnumerateExA@12");
extern char ddraw_export_anchor_5 __asm__("_DirectDrawEnumerateExW@12");
extern char ddraw_export_anchor_6 __asm__("_DirectDrawEnumerateW@8");
extern char ddraw_export_anchor_7 __asm__("_DllCanUnloadNow@0");
extern char ddraw_export_anchor_8 __asm__("_DllGetClassObject@12");
extern char ddraw_export_anchor_9 __asm__("_DllRegisterServer@0");
extern char ddraw_export_anchor_10 __asm__("_DllUnregisterServer@0");
extern char ddraw_export_anchor_11 __asm__("_GetSurfaceFromDC@12");
__attribute__((used)) static const void *const ddraw_export_anchors[] = {
    &ddraw_export_anchor_0,
    &ddraw_export_anchor_1,
    &ddraw_export_anchor_2,
    &ddraw_export_anchor_3,
    &ddraw_export_anchor_4,
    &ddraw_export_anchor_5,
    &ddraw_export_anchor_6,
    &ddraw_export_anchor_7,
    &ddraw_export_anchor_8,
    &ddraw_export_anchor_9,
    &ddraw_export_anchor_10,
    &ddraw_export_anchor_11,
    0,
};
#elif defined(__x86_64__)
extern char ddraw_export_anchor_0 __asm__("DirectDrawCreate");
extern char ddraw_export_anchor_1 __asm__("DirectDrawCreateClipper");
extern char ddraw_export_anchor_2 __asm__("DirectDrawCreateEx");
extern char ddraw_export_anchor_3 __asm__("DirectDrawEnumerateA");
extern char ddraw_export_anchor_4 __asm__("DirectDrawEnumerateExA");
extern char ddraw_export_anchor_5 __asm__("DirectDrawEnumerateExW");
extern char ddraw_export_anchor_6 __asm__("DirectDrawEnumerateW");
extern char ddraw_export_anchor_7 __asm__("DllCanUnloadNow");
extern char ddraw_export_anchor_8 __asm__("DllGetClassObject");
extern char ddraw_export_anchor_9 __asm__("DllRegisterServer");
extern char ddraw_export_anchor_10 __asm__("DllUnregisterServer");
extern char ddraw_export_anchor_11 __asm__("GetSurfaceFromDC");
__attribute__((used)) static const void *const ddraw_export_anchors[] = {
    &ddraw_export_anchor_0,
    &ddraw_export_anchor_1,
    &ddraw_export_anchor_2,
    &ddraw_export_anchor_3,
    &ddraw_export_anchor_4,
    &ddraw_export_anchor_5,
    &ddraw_export_anchor_6,
    &ddraw_export_anchor_7,
    &ddraw_export_anchor_8,
    &ddraw_export_anchor_9,
    &ddraw_export_anchor_10,
    &ddraw_export_anchor_11,
    0,
};
#elif defined(__aarch64__)
extern char ddraw_export_anchor_0 __asm__("DirectDrawCreate");
extern char ddraw_export_anchor_1 __asm__("DirectDrawCreateClipper");
extern char ddraw_export_anchor_2 __asm__("DirectDrawCreateEx");
extern char ddraw_export_anchor_3 __asm__("DirectDrawEnumerateA");
extern char ddraw_export_anchor_4 __asm__("DirectDrawEnumerateExA");
extern char ddraw_export_anchor_5 __asm__("DirectDrawEnumerateExW");
extern char ddraw_export_anchor_6 __asm__("DirectDrawEnumerateW");
extern char ddraw_export_anchor_7 __asm__("DllCanUnloadNow");
extern char ddraw_export_anchor_8 __asm__("DllGetClassObject");
extern char ddraw_export_anchor_9 __asm__("DllRegisterServer");
extern char ddraw_export_anchor_10 __asm__("DllUnregisterServer");
extern char ddraw_export_anchor_11 __asm__("GetSurfaceFromDC");
__attribute__((used)) static const void *const ddraw_export_anchors[] = {
    &ddraw_export_anchor_0,
    &ddraw_export_anchor_1,
    &ddraw_export_anchor_2,
    &ddraw_export_anchor_3,
    &ddraw_export_anchor_4,
    &ddraw_export_anchor_5,
    &ddraw_export_anchor_6,
    &ddraw_export_anchor_7,
    &ddraw_export_anchor_8,
    &ddraw_export_anchor_9,
    &ddraw_export_anchor_10,
    &ddraw_export_anchor_11,
    0,
};
#endif
