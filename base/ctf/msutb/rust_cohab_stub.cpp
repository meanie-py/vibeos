/* msutb is Rust (sdk/lib/rust/msutb-rust); a CMake target needs one TU. */
typedef int msutb_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char msutb_export_anchor_0 __asm__("_ClosePopupTipbar@0");
extern char msutb_export_anchor_1 __asm__("_DllCanUnloadNow@0");
extern char msutb_export_anchor_2 __asm__("_DllGetClassObject@12");
extern char msutb_export_anchor_3 __asm__("_DllRegisterServer@0");
extern char msutb_export_anchor_4 __asm__("_DllUnregisterServer@0");
extern char msutb_export_anchor_5 __asm__("_GetLibTls@0");
extern char msutb_export_anchor_6 __asm__("_GetPopupTipbar@8");
extern char msutb_export_anchor_7 __asm__("_SetRegisterLangBand@4");
__attribute__((used)) static const void *const msutb_export_anchors[] = {
    &msutb_export_anchor_0,
    &msutb_export_anchor_1,
    &msutb_export_anchor_2,
    &msutb_export_anchor_3,
    &msutb_export_anchor_4,
    &msutb_export_anchor_5,
    &msutb_export_anchor_6,
    &msutb_export_anchor_7,
    0,
};
#elif defined(__x86_64__)
extern char msutb_export_anchor_0 __asm__("ClosePopupTipbar");
extern char msutb_export_anchor_1 __asm__("DllCanUnloadNow");
extern char msutb_export_anchor_2 __asm__("DllGetClassObject");
extern char msutb_export_anchor_3 __asm__("DllRegisterServer");
extern char msutb_export_anchor_4 __asm__("DllUnregisterServer");
extern char msutb_export_anchor_5 __asm__("GetLibTls");
extern char msutb_export_anchor_6 __asm__("GetPopupTipbar");
extern char msutb_export_anchor_7 __asm__("SetRegisterLangBand");
__attribute__((used)) static const void *const msutb_export_anchors[] = {
    &msutb_export_anchor_0,
    &msutb_export_anchor_1,
    &msutb_export_anchor_2,
    &msutb_export_anchor_3,
    &msutb_export_anchor_4,
    &msutb_export_anchor_5,
    &msutb_export_anchor_6,
    &msutb_export_anchor_7,
    0,
};
#elif defined(__aarch64__)
extern char msutb_export_anchor_0 __asm__("ClosePopupTipbar");
extern char msutb_export_anchor_1 __asm__("DllCanUnloadNow");
extern char msutb_export_anchor_2 __asm__("DllGetClassObject");
extern char msutb_export_anchor_3 __asm__("DllRegisterServer");
extern char msutb_export_anchor_4 __asm__("DllUnregisterServer");
extern char msutb_export_anchor_5 __asm__("GetLibTls");
extern char msutb_export_anchor_6 __asm__("GetPopupTipbar");
extern char msutb_export_anchor_7 __asm__("SetRegisterLangBand");
__attribute__((used)) static const void *const msutb_export_anchors[] = {
    &msutb_export_anchor_0,
    &msutb_export_anchor_1,
    &msutb_export_anchor_2,
    &msutb_export_anchor_3,
    &msutb_export_anchor_4,
    &msutb_export_anchor_5,
    &msutb_export_anchor_6,
    &msutb_export_anchor_7,
    0,
};
#endif
