/* cryptnet is Rust (sdk/lib/rust/cryptnet-rust); a CMake target needs one TU. */
typedef int cryptnet_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char cryptnet_export_anchor_0 __asm__("_CertDllVerifyRevocation@28");
extern char cryptnet_export_anchor_1 __asm__("_CryptGetObjectUrl@32");
extern char cryptnet_export_anchor_2 __asm__("_CryptRetrieveObjectByUrlA@36");
extern char cryptnet_export_anchor_3 __asm__("_CryptRetrieveObjectByUrlW@36");
extern char cryptnet_export_anchor_4 __asm__("_DllRegisterServer@0");
extern char cryptnet_export_anchor_5 __asm__("_DllUnregisterServer@0");
__attribute__((used)) static const void *const cryptnet_export_anchors[] = {
    &cryptnet_export_anchor_0,
    &cryptnet_export_anchor_1,
    &cryptnet_export_anchor_2,
    &cryptnet_export_anchor_3,
    &cryptnet_export_anchor_4,
    &cryptnet_export_anchor_5,
    0,
};
#elif defined(__x86_64__)
extern char cryptnet_export_anchor_0 __asm__("CertDllVerifyRevocation");
extern char cryptnet_export_anchor_1 __asm__("CryptGetObjectUrl");
extern char cryptnet_export_anchor_2 __asm__("CryptRetrieveObjectByUrlA");
extern char cryptnet_export_anchor_3 __asm__("CryptRetrieveObjectByUrlW");
extern char cryptnet_export_anchor_4 __asm__("DllRegisterServer");
extern char cryptnet_export_anchor_5 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const cryptnet_export_anchors[] = {
    &cryptnet_export_anchor_0,
    &cryptnet_export_anchor_1,
    &cryptnet_export_anchor_2,
    &cryptnet_export_anchor_3,
    &cryptnet_export_anchor_4,
    &cryptnet_export_anchor_5,
    0,
};
#elif defined(__aarch64__)
extern char cryptnet_export_anchor_0 __asm__("CertDllVerifyRevocation");
extern char cryptnet_export_anchor_1 __asm__("CryptGetObjectUrl");
extern char cryptnet_export_anchor_2 __asm__("CryptRetrieveObjectByUrlA");
extern char cryptnet_export_anchor_3 __asm__("CryptRetrieveObjectByUrlW");
extern char cryptnet_export_anchor_4 __asm__("DllRegisterServer");
extern char cryptnet_export_anchor_5 __asm__("DllUnregisterServer");
__attribute__((used)) static const void *const cryptnet_export_anchors[] = {
    &cryptnet_export_anchor_0,
    &cryptnet_export_anchor_1,
    &cryptnet_export_anchor_2,
    &cryptnet_export_anchor_3,
    &cryptnet_export_anchor_4,
    &cryptnet_export_anchor_5,
    0,
};
#endif
