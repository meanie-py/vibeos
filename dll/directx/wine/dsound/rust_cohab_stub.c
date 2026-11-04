/* dsound is Rust (sdk/lib/rust/dsound-rust); a CMake target needs one TU. */
typedef int dsound_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char dsound_export_anchor_0 __asm__("_DirectSoundCaptureCreate8@12");
extern char dsound_export_anchor_1 __asm__("_DirectSoundCaptureCreate@12");
extern char dsound_export_anchor_2 __asm__("_DirectSoundCaptureEnumerateA@8");
extern char dsound_export_anchor_3 __asm__("_DirectSoundCaptureEnumerateW@8");
extern char dsound_export_anchor_4 __asm__("_DirectSoundCreate8@12");
extern char dsound_export_anchor_5 __asm__("_DirectSoundCreate@12");
extern char dsound_export_anchor_6 __asm__("_DirectSoundEnumerateA@8");
extern char dsound_export_anchor_7 __asm__("_DirectSoundEnumerateW@8");
extern char dsound_export_anchor_8 __asm__("_DirectSoundFullDuplexCreate@40");
extern char dsound_export_anchor_9 __asm__("_DllCanUnloadNow@0");
extern char dsound_export_anchor_10 __asm__("_DllGetClassObject@12");
extern char dsound_export_anchor_11 __asm__("_DllRegisterServer@0");
extern char dsound_export_anchor_12 __asm__("_DllUnregisterServer@0");
extern char dsound_export_anchor_13 __asm__("_GetDeviceID@8");
__attribute__((used)) static const void *const dsound_export_anchors[] = {
    &dsound_export_anchor_0,
    &dsound_export_anchor_1,
    &dsound_export_anchor_2,
    &dsound_export_anchor_3,
    &dsound_export_anchor_4,
    &dsound_export_anchor_5,
    &dsound_export_anchor_6,
    &dsound_export_anchor_7,
    &dsound_export_anchor_8,
    &dsound_export_anchor_9,
    &dsound_export_anchor_10,
    &dsound_export_anchor_11,
    &dsound_export_anchor_12,
    &dsound_export_anchor_13,
    0,
};
#elif defined(__x86_64__)
extern char dsound_export_anchor_0 __asm__("DirectSoundCaptureCreate");
extern char dsound_export_anchor_1 __asm__("DirectSoundCaptureCreate8");
extern char dsound_export_anchor_2 __asm__("DirectSoundCaptureEnumerateA");
extern char dsound_export_anchor_3 __asm__("DirectSoundCaptureEnumerateW");
extern char dsound_export_anchor_4 __asm__("DirectSoundCreate");
extern char dsound_export_anchor_5 __asm__("DirectSoundCreate8");
extern char dsound_export_anchor_6 __asm__("DirectSoundEnumerateA");
extern char dsound_export_anchor_7 __asm__("DirectSoundEnumerateW");
extern char dsound_export_anchor_8 __asm__("DirectSoundFullDuplexCreate");
extern char dsound_export_anchor_9 __asm__("DllCanUnloadNow");
extern char dsound_export_anchor_10 __asm__("DllGetClassObject");
extern char dsound_export_anchor_11 __asm__("DllRegisterServer");
extern char dsound_export_anchor_12 __asm__("DllUnregisterServer");
extern char dsound_export_anchor_13 __asm__("GetDeviceID");
__attribute__((used)) static const void *const dsound_export_anchors[] = {
    &dsound_export_anchor_0,
    &dsound_export_anchor_1,
    &dsound_export_anchor_2,
    &dsound_export_anchor_3,
    &dsound_export_anchor_4,
    &dsound_export_anchor_5,
    &dsound_export_anchor_6,
    &dsound_export_anchor_7,
    &dsound_export_anchor_8,
    &dsound_export_anchor_9,
    &dsound_export_anchor_10,
    &dsound_export_anchor_11,
    &dsound_export_anchor_12,
    &dsound_export_anchor_13,
    0,
};
#elif defined(__aarch64__)
extern char dsound_export_anchor_0 __asm__("DirectSoundCaptureCreate");
extern char dsound_export_anchor_1 __asm__("DirectSoundCaptureCreate8");
extern char dsound_export_anchor_2 __asm__("DirectSoundCaptureEnumerateA");
extern char dsound_export_anchor_3 __asm__("DirectSoundCaptureEnumerateW");
extern char dsound_export_anchor_4 __asm__("DirectSoundCreate");
extern char dsound_export_anchor_5 __asm__("DirectSoundCreate8");
extern char dsound_export_anchor_6 __asm__("DirectSoundEnumerateA");
extern char dsound_export_anchor_7 __asm__("DirectSoundEnumerateW");
extern char dsound_export_anchor_8 __asm__("DirectSoundFullDuplexCreate");
extern char dsound_export_anchor_9 __asm__("DllCanUnloadNow");
extern char dsound_export_anchor_10 __asm__("DllGetClassObject");
extern char dsound_export_anchor_11 __asm__("DllRegisterServer");
extern char dsound_export_anchor_12 __asm__("DllUnregisterServer");
extern char dsound_export_anchor_13 __asm__("GetDeviceID");
__attribute__((used)) static const void *const dsound_export_anchors[] = {
    &dsound_export_anchor_0,
    &dsound_export_anchor_1,
    &dsound_export_anchor_2,
    &dsound_export_anchor_3,
    &dsound_export_anchor_4,
    &dsound_export_anchor_5,
    &dsound_export_anchor_6,
    &dsound_export_anchor_7,
    &dsound_export_anchor_8,
    &dsound_export_anchor_9,
    &dsound_export_anchor_10,
    &dsound_export_anchor_11,
    &dsound_export_anchor_12,
    &dsound_export_anchor_13,
    0,
};
#endif
