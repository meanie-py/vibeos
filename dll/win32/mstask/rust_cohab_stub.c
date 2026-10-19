/* mstask is Rust (sdk/lib/rust/mstask-rust); a CMake target needs one TU. */
typedef int mstask_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char mstask_export_anchor_0 __asm__("_ConvertAtJobsToTasks@0");
extern char mstask_export_anchor_1 __asm__("_DllCanUnloadNow@0");
extern char mstask_export_anchor_2 __asm__("_DllGetClassObject@12");
extern char mstask_export_anchor_3 __asm__("_DllRegisterServer@0");
extern char mstask_export_anchor_4 __asm__("_DllUnregisterServer@0");
extern char mstask_export_anchor_5 __asm__("_GetNetScheduleAccountInformation@12");
extern char mstask_export_anchor_6 __asm__("_NetrJobAdd@12");
extern char mstask_export_anchor_7 __asm__("_NetrJobDel@12");
extern char mstask_export_anchor_8 __asm__("_NetrJobEnum@20");
extern char mstask_export_anchor_9 __asm__("_NetrJobGetInfo@12");
extern char mstask_export_anchor_10 __asm__("_SAGetAccountInformation@16");
extern char mstask_export_anchor_11 __asm__("_SAGetNSAccountInformation@12");
extern char mstask_export_anchor_12 __asm__("_SASetAccountInformation@20");
extern char mstask_export_anchor_13 __asm__("_SASetNSAccountInformation@12");
extern char mstask_export_anchor_14 __asm__("_SetNetScheduleAccountInformation@12");
__attribute__((used)) static const void *const mstask_export_anchors[] = {
    &mstask_export_anchor_0,
    &mstask_export_anchor_1,
    &mstask_export_anchor_2,
    &mstask_export_anchor_3,
    &mstask_export_anchor_4,
    &mstask_export_anchor_5,
    &mstask_export_anchor_6,
    &mstask_export_anchor_7,
    &mstask_export_anchor_8,
    &mstask_export_anchor_9,
    &mstask_export_anchor_10,
    &mstask_export_anchor_11,
    &mstask_export_anchor_12,
    &mstask_export_anchor_13,
    &mstask_export_anchor_14,
    0,
};
#elif defined(__x86_64__)
extern char mstask_export_anchor_0 __asm__("ConvertAtJobsToTasks");
extern char mstask_export_anchor_1 __asm__("DllCanUnloadNow");
extern char mstask_export_anchor_2 __asm__("DllGetClassObject");
extern char mstask_export_anchor_3 __asm__("DllRegisterServer");
extern char mstask_export_anchor_4 __asm__("DllUnregisterServer");
extern char mstask_export_anchor_5 __asm__("GetNetScheduleAccountInformation");
extern char mstask_export_anchor_6 __asm__("NetrJobAdd");
extern char mstask_export_anchor_7 __asm__("NetrJobDel");
extern char mstask_export_anchor_8 __asm__("NetrJobEnum");
extern char mstask_export_anchor_9 __asm__("NetrJobGetInfo");
extern char mstask_export_anchor_10 __asm__("SAGetAccountInformation");
extern char mstask_export_anchor_11 __asm__("SAGetNSAccountInformation");
extern char mstask_export_anchor_12 __asm__("SASetAccountInformation");
extern char mstask_export_anchor_13 __asm__("SASetNSAccountInformation");
extern char mstask_export_anchor_14 __asm__("SetNetScheduleAccountInformation");
__attribute__((used)) static const void *const mstask_export_anchors[] = {
    &mstask_export_anchor_0,
    &mstask_export_anchor_1,
    &mstask_export_anchor_2,
    &mstask_export_anchor_3,
    &mstask_export_anchor_4,
    &mstask_export_anchor_5,
    &mstask_export_anchor_6,
    &mstask_export_anchor_7,
    &mstask_export_anchor_8,
    &mstask_export_anchor_9,
    &mstask_export_anchor_10,
    &mstask_export_anchor_11,
    &mstask_export_anchor_12,
    &mstask_export_anchor_13,
    &mstask_export_anchor_14,
    0,
};
#elif defined(__aarch64__)
extern char mstask_export_anchor_0 __asm__("ConvertAtJobsToTasks");
extern char mstask_export_anchor_1 __asm__("DllCanUnloadNow");
extern char mstask_export_anchor_2 __asm__("DllGetClassObject");
extern char mstask_export_anchor_3 __asm__("DllRegisterServer");
extern char mstask_export_anchor_4 __asm__("DllUnregisterServer");
extern char mstask_export_anchor_5 __asm__("GetNetScheduleAccountInformation");
extern char mstask_export_anchor_6 __asm__("NetrJobAdd");
extern char mstask_export_anchor_7 __asm__("NetrJobDel");
extern char mstask_export_anchor_8 __asm__("NetrJobEnum");
extern char mstask_export_anchor_9 __asm__("NetrJobGetInfo");
extern char mstask_export_anchor_10 __asm__("SAGetAccountInformation");
extern char mstask_export_anchor_11 __asm__("SAGetNSAccountInformation");
extern char mstask_export_anchor_12 __asm__("SASetAccountInformation");
extern char mstask_export_anchor_13 __asm__("SASetNSAccountInformation");
extern char mstask_export_anchor_14 __asm__("SetNetScheduleAccountInformation");
__attribute__((used)) static const void *const mstask_export_anchors[] = {
    &mstask_export_anchor_0,
    &mstask_export_anchor_1,
    &mstask_export_anchor_2,
    &mstask_export_anchor_3,
    &mstask_export_anchor_4,
    &mstask_export_anchor_5,
    &mstask_export_anchor_6,
    &mstask_export_anchor_7,
    &mstask_export_anchor_8,
    &mstask_export_anchor_9,
    &mstask_export_anchor_10,
    &mstask_export_anchor_11,
    &mstask_export_anchor_12,
    &mstask_export_anchor_13,
    &mstask_export_anchor_14,
    0,
};
#endif
