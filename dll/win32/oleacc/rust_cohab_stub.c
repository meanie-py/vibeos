/* oleacc is Rust (sdk/lib/rust/oleacc-rust); a CMake target needs one TU. */
typedef int oleacc_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char oleacc_export_anchor_0 __asm__("_AccessibleChildren@20");
extern char oleacc_export_anchor_1 __asm__("_AccessibleObjectFromEvent@20");
extern char oleacc_export_anchor_2 __asm__("_AccessibleObjectFromPoint@16");
extern char oleacc_export_anchor_3 __asm__("_AccessibleObjectFromWindow@16");
extern char oleacc_export_anchor_4 __asm__("_CreateStdAccessibleObject@16");
extern char oleacc_export_anchor_5 __asm__("_DllGetClassObject@12");
extern char oleacc_export_anchor_6 __asm__("_DllRegisterServer@0");
extern char oleacc_export_anchor_7 __asm__("_DllUnregisterServer@0");
extern char oleacc_export_anchor_8 __asm__("_GetOleaccVersionInfo@8");
extern char oleacc_export_anchor_9 __asm__("_GetProcessHandleFromHwnd@4");
extern char oleacc_export_anchor_10 __asm__("_GetRoleTextA@12");
extern char oleacc_export_anchor_11 __asm__("_GetRoleTextW@12");
extern char oleacc_export_anchor_12 __asm__("_GetStateTextA@12");
extern char oleacc_export_anchor_13 __asm__("_GetStateTextW@12");
extern char oleacc_export_anchor_14 __asm__("_LresultFromObject@12");
extern char oleacc_export_anchor_15 __asm__("_ObjectFromLresult@16");
extern char oleacc_export_anchor_16 __asm__("_WindowFromAccessibleObject@8");
__attribute__((used)) static const void *const oleacc_export_anchors[] = {
    &oleacc_export_anchor_0,
    &oleacc_export_anchor_1,
    &oleacc_export_anchor_2,
    &oleacc_export_anchor_3,
    &oleacc_export_anchor_4,
    &oleacc_export_anchor_5,
    &oleacc_export_anchor_6,
    &oleacc_export_anchor_7,
    &oleacc_export_anchor_8,
    &oleacc_export_anchor_9,
    &oleacc_export_anchor_10,
    &oleacc_export_anchor_11,
    &oleacc_export_anchor_12,
    &oleacc_export_anchor_13,
    &oleacc_export_anchor_14,
    &oleacc_export_anchor_15,
    &oleacc_export_anchor_16,
    0,
};
#elif defined(__x86_64__)
extern char oleacc_export_anchor_0 __asm__("AccessibleChildren");
extern char oleacc_export_anchor_1 __asm__("AccessibleObjectFromEvent");
extern char oleacc_export_anchor_2 __asm__("AccessibleObjectFromPoint");
extern char oleacc_export_anchor_3 __asm__("AccessibleObjectFromWindow");
extern char oleacc_export_anchor_4 __asm__("CreateStdAccessibleObject");
extern char oleacc_export_anchor_5 __asm__("DllGetClassObject");
extern char oleacc_export_anchor_6 __asm__("DllRegisterServer");
extern char oleacc_export_anchor_7 __asm__("DllUnregisterServer");
extern char oleacc_export_anchor_8 __asm__("GetOleaccVersionInfo");
extern char oleacc_export_anchor_9 __asm__("GetProcessHandleFromHwnd");
extern char oleacc_export_anchor_10 __asm__("GetRoleTextA");
extern char oleacc_export_anchor_11 __asm__("GetRoleTextW");
extern char oleacc_export_anchor_12 __asm__("GetStateTextA");
extern char oleacc_export_anchor_13 __asm__("GetStateTextW");
extern char oleacc_export_anchor_14 __asm__("LresultFromObject");
extern char oleacc_export_anchor_15 __asm__("ObjectFromLresult");
extern char oleacc_export_anchor_16 __asm__("WindowFromAccessibleObject");
__attribute__((used)) static const void *const oleacc_export_anchors[] = {
    &oleacc_export_anchor_0,
    &oleacc_export_anchor_1,
    &oleacc_export_anchor_2,
    &oleacc_export_anchor_3,
    &oleacc_export_anchor_4,
    &oleacc_export_anchor_5,
    &oleacc_export_anchor_6,
    &oleacc_export_anchor_7,
    &oleacc_export_anchor_8,
    &oleacc_export_anchor_9,
    &oleacc_export_anchor_10,
    &oleacc_export_anchor_11,
    &oleacc_export_anchor_12,
    &oleacc_export_anchor_13,
    &oleacc_export_anchor_14,
    &oleacc_export_anchor_15,
    &oleacc_export_anchor_16,
    0,
};
#elif defined(__aarch64__)
extern char oleacc_export_anchor_0 __asm__("AccessibleChildren");
extern char oleacc_export_anchor_1 __asm__("AccessibleObjectFromEvent");
extern char oleacc_export_anchor_2 __asm__("AccessibleObjectFromPoint");
extern char oleacc_export_anchor_3 __asm__("AccessibleObjectFromWindow");
extern char oleacc_export_anchor_4 __asm__("CreateStdAccessibleObject");
extern char oleacc_export_anchor_5 __asm__("DllGetClassObject");
extern char oleacc_export_anchor_6 __asm__("DllRegisterServer");
extern char oleacc_export_anchor_7 __asm__("DllUnregisterServer");
extern char oleacc_export_anchor_8 __asm__("GetOleaccVersionInfo");
extern char oleacc_export_anchor_9 __asm__("GetProcessHandleFromHwnd");
extern char oleacc_export_anchor_10 __asm__("GetRoleTextA");
extern char oleacc_export_anchor_11 __asm__("GetRoleTextW");
extern char oleacc_export_anchor_12 __asm__("GetStateTextA");
extern char oleacc_export_anchor_13 __asm__("GetStateTextW");
extern char oleacc_export_anchor_14 __asm__("LresultFromObject");
extern char oleacc_export_anchor_15 __asm__("ObjectFromLresult");
extern char oleacc_export_anchor_16 __asm__("WindowFromAccessibleObject");
__attribute__((used)) static const void *const oleacc_export_anchors[] = {
    &oleacc_export_anchor_0,
    &oleacc_export_anchor_1,
    &oleacc_export_anchor_2,
    &oleacc_export_anchor_3,
    &oleacc_export_anchor_4,
    &oleacc_export_anchor_5,
    &oleacc_export_anchor_6,
    &oleacc_export_anchor_7,
    &oleacc_export_anchor_8,
    &oleacc_export_anchor_9,
    &oleacc_export_anchor_10,
    &oleacc_export_anchor_11,
    &oleacc_export_anchor_12,
    &oleacc_export_anchor_13,
    &oleacc_export_anchor_14,
    &oleacc_export_anchor_15,
    &oleacc_export_anchor_16,
    0,
};
#endif
