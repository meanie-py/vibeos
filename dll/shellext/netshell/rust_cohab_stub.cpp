/* netshell is Rust (sdk/lib/rust/netshell-rust); a CMake target needs one TU. */
typedef int netshell_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char netshell_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char netshell_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char netshell_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char netshell_export_anchor_3 __asm__("_DllUnregisterServer@0");
extern char netshell_export_anchor_4 __asm__("_NcFreeNetconProperties@4");
extern char netshell_export_anchor_5 __asm__("_NcIsValidConnectionName@4");
extern char netshell_export_anchor_6 __asm__("_NetSetupRequestWizardPages@12");
__attribute__((used)) static const void *const netshell_export_anchors[] = {
    &netshell_export_anchor_0,
    &netshell_export_anchor_1,
    &netshell_export_anchor_2,
    &netshell_export_anchor_3,
    &netshell_export_anchor_4,
    &netshell_export_anchor_5,
    &netshell_export_anchor_6,
    0,
};
#elif defined(__x86_64__)
extern char netshell_export_anchor_0 __asm__("DllCanUnloadNow");
extern char netshell_export_anchor_1 __asm__("DllGetClassObject");
extern char netshell_export_anchor_2 __asm__("DllRegisterServer");
extern char netshell_export_anchor_3 __asm__("DllUnregisterServer");
extern char netshell_export_anchor_4 __asm__("NcFreeNetconProperties");
extern char netshell_export_anchor_5 __asm__("NcIsValidConnectionName");
extern char netshell_export_anchor_6 __asm__("NetSetupRequestWizardPages");
__attribute__((used)) static const void *const netshell_export_anchors[] = {
    &netshell_export_anchor_0,
    &netshell_export_anchor_1,
    &netshell_export_anchor_2,
    &netshell_export_anchor_3,
    &netshell_export_anchor_4,
    &netshell_export_anchor_5,
    &netshell_export_anchor_6,
    0,
};
#elif defined(__aarch64__)
extern char netshell_export_anchor_0 __asm__("DllCanUnloadNow");
extern char netshell_export_anchor_1 __asm__("DllGetClassObject");
extern char netshell_export_anchor_2 __asm__("DllRegisterServer");
extern char netshell_export_anchor_3 __asm__("DllUnregisterServer");
extern char netshell_export_anchor_4 __asm__("NcFreeNetconProperties");
extern char netshell_export_anchor_5 __asm__("NcIsValidConnectionName");
extern char netshell_export_anchor_6 __asm__("NetSetupRequestWizardPages");
__attribute__((used)) static const void *const netshell_export_anchors[] = {
    &netshell_export_anchor_0,
    &netshell_export_anchor_1,
    &netshell_export_anchor_2,
    &netshell_export_anchor_3,
    &netshell_export_anchor_4,
    &netshell_export_anchor_5,
    &netshell_export_anchor_6,
    0,
};
#endif
