/* dinput is Rust (sdk/lib/rust/dinput-rust); a CMake target needs one TU. */
typedef int dinput_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char dinput_export_anchor_0 __asm__("_DirectInputCreateA@16");
extern char dinput_export_anchor_1 __asm__("_DirectInputCreateEx@20");
extern char dinput_export_anchor_2 __asm__("_DirectInputCreateW@16");
extern char dinput_export_anchor_3 __asm__("_DllCanUnloadNow@0");
extern char dinput_export_anchor_4 __asm__("_DllGetClassObject@12");
extern char dinput_export_anchor_5 __asm__("_DllRegisterServer@0");
extern char dinput_export_anchor_6 __asm__("_DllUnregisterServer@0");
__attribute__((used)) static const void *const dinput_export_anchors[] = {
    &dinput_export_anchor_0,
    &dinput_export_anchor_1,
    &dinput_export_anchor_2,
    &dinput_export_anchor_3,
    &dinput_export_anchor_4,
    &dinput_export_anchor_5,
    &dinput_export_anchor_6,
    0,
};
#elif defined(__x86_64__)
extern char dinput_export_anchor_0 __asm__("DirectInputCreateA");
extern char dinput_export_anchor_1 __asm__("DirectInputCreateEx");
extern char dinput_export_anchor_2 __asm__("DirectInputCreateW");
extern char dinput_export_anchor_3 __asm__("DllCanUnloadNow");
extern char dinput_export_anchor_4 __asm__("DllGetClassObject");
extern char dinput_export_anchor_5 __asm__("DllRegisterServer");
extern char dinput_export_anchor_6 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const dinput_export_anchors[] = {
    &dinput_export_anchor_0,
    &dinput_export_anchor_1,
    &dinput_export_anchor_2,
    &dinput_export_anchor_3,
    &dinput_export_anchor_4,
    &dinput_export_anchor_5,
    &dinput_export_anchor_6,
    0,
};
#elif defined(__aarch64__)
extern char dinput_export_anchor_0 __asm__("DirectInputCreateA");
extern char dinput_export_anchor_1 __asm__("DirectInputCreateEx");
extern char dinput_export_anchor_2 __asm__("DirectInputCreateW");
extern char dinput_export_anchor_3 __asm__("DllCanUnloadNow");
extern char dinput_export_anchor_4 __asm__("DllGetClassObject");
extern char dinput_export_anchor_5 __asm__("DllRegisterServer");
extern char dinput_export_anchor_6 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const dinput_export_anchors[] = {
    &dinput_export_anchor_0,
    &dinput_export_anchor_1,
    &dinput_export_anchor_2,
    &dinput_export_anchor_3,
    &dinput_export_anchor_4,
    &dinput_export_anchor_5,
    &dinput_export_anchor_6,
    0,
};
#endif
