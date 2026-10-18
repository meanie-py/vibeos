/* msctf is Rust (sdk/lib/rust/msctf-rust); a CMake target needs one TU. */
typedef int msctf_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char msctf_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char msctf_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char msctf_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char msctf_export_anchor_3 __asm__("_DllUnregisterServer@0");
extern char msctf_export_anchor_4 __asm__("_SetInputScope@8");
extern char msctf_export_anchor_5 __asm__("_SetInputScopeXML@8");
extern char msctf_export_anchor_6 __asm__("_SetInputScopes@28");
extern char msctf_export_anchor_7 __asm__("_TF_CUASAppFix@4");
extern char msctf_export_anchor_8 __asm__("_TF_CheckThreadInputIdle@8");
extern char msctf_export_anchor_9 __asm__("_TF_ClearLangBarAddIns@4");
extern char msctf_export_anchor_10 __asm__("_TF_CreateCategoryMgr@4");
extern char msctf_export_anchor_11 __asm__("_TF_CreateCicLoadMutex@4");
extern char msctf_export_anchor_12 __asm__("_TF_CreateDisplayAttributeMgr@4");
extern char msctf_export_anchor_13 __asm__("_TF_CreateInputProcessorProfiles@4");
extern char msctf_export_anchor_14 __asm__("_TF_CreateLangBarItemMgr@4");
extern char msctf_export_anchor_15 __asm__("_TF_CreateLangBarMgr@4");
extern char msctf_export_anchor_16 __asm__("_TF_CreateThreadMgr@4");
extern char msctf_export_anchor_17 __asm__("_TF_DllDetachInOther@0");
extern char msctf_export_anchor_18 __asm__("_TF_GetGlobalCompartment@4");
extern char msctf_export_anchor_19 __asm__("_TF_GetInputScope@8");
extern char msctf_export_anchor_20 __asm__("_TF_GetLangIcon@12");
extern char msctf_export_anchor_21 __asm__("_TF_GetMlngHKL@16");
extern char msctf_export_anchor_22 __asm__("_TF_GetMlngIconIndex@4");
extern char msctf_export_anchor_23 __asm__("_TF_GetThreadFlags@16");
extern char msctf_export_anchor_24 __asm__("_TF_GetThreadMgr@4");
extern char msctf_export_anchor_25 __asm__("_TF_InatExtractIcon@4");
extern char msctf_export_anchor_26 __asm__("_TF_InitMlngInfo@0");
extern char msctf_export_anchor_27 __asm__("_TF_InitSystem@0");
extern char msctf_export_anchor_28 __asm__("_TF_InvalidAssemblyListCache@0");
extern char msctf_export_anchor_29 __asm__("_TF_InvalidAssemblyListCacheIfExist@0");
extern char msctf_export_anchor_30 __asm__("_TF_IsCtfmonRunning@0");
extern char msctf_export_anchor_31 __asm__("_TF_IsFullScreenWindowAcitvated@0");
extern char msctf_export_anchor_32 __asm__("_TF_IsInMarshaling@4");
extern char msctf_export_anchor_33 __asm__("_TF_MlngInfoCount@0");
extern char msctf_export_anchor_34 __asm__("_TF_PostAllThreadMsg@8");
extern char msctf_export_anchor_35 __asm__("_TF_RegisterLangBarAddIn@12");
extern char msctf_export_anchor_36 __asm__("_TF_RunInputCPL@0");
extern char msctf_export_anchor_37 __asm__("_TF_UninitSystem@0");
extern char msctf_export_anchor_38 __asm__("_TF_UnregisterLangBarAddIn@8");
__attribute__((used)) static const void *const msctf_export_anchors[] = {
    &msctf_export_anchor_0,
    &msctf_export_anchor_1,
    &msctf_export_anchor_2,
    &msctf_export_anchor_3,
    &msctf_export_anchor_4,
    &msctf_export_anchor_5,
    &msctf_export_anchor_6,
    &msctf_export_anchor_7,
    &msctf_export_anchor_8,
    &msctf_export_anchor_9,
    &msctf_export_anchor_10,
    &msctf_export_anchor_11,
    &msctf_export_anchor_12,
    &msctf_export_anchor_13,
    &msctf_export_anchor_14,
    &msctf_export_anchor_15,
    &msctf_export_anchor_16,
    &msctf_export_anchor_17,
    &msctf_export_anchor_18,
    &msctf_export_anchor_19,
    &msctf_export_anchor_20,
    &msctf_export_anchor_21,
    &msctf_export_anchor_22,
    &msctf_export_anchor_23,
    &msctf_export_anchor_24,
    &msctf_export_anchor_25,
    &msctf_export_anchor_26,
    &msctf_export_anchor_27,
    &msctf_export_anchor_28,
    &msctf_export_anchor_29,
    &msctf_export_anchor_30,
    &msctf_export_anchor_31,
    &msctf_export_anchor_32,
    &msctf_export_anchor_33,
    &msctf_export_anchor_34,
    &msctf_export_anchor_35,
    &msctf_export_anchor_36,
    &msctf_export_anchor_37,
    &msctf_export_anchor_38,
    0,
};
#elif defined(__x86_64__)
extern char msctf_export_anchor_0 __asm__("DllCanUnloadNow");
extern char msctf_export_anchor_1 __asm__("DllGetClassObject");
extern char msctf_export_anchor_2 __asm__("DllRegisterServer");
extern char msctf_export_anchor_3 __asm__("DllUnregisterServer");
extern char msctf_export_anchor_4 __asm__("SetInputScope");
extern char msctf_export_anchor_5 __asm__("SetInputScopeXML");
extern char msctf_export_anchor_6 __asm__("SetInputScopes");
extern char msctf_export_anchor_7 __asm__("TF_CUASAppFix");
extern char msctf_export_anchor_8 __asm__("TF_CheckThreadInputIdle");
extern char msctf_export_anchor_9 __asm__("TF_ClearLangBarAddIns");
extern char msctf_export_anchor_10 __asm__("TF_CreateCategoryMgr");
extern char msctf_export_anchor_11 __asm__("TF_CreateCicLoadMutex");
extern char msctf_export_anchor_12 __asm__("TF_CreateDisplayAttributeMgr");
extern char msctf_export_anchor_13 __asm__("TF_CreateInputProcessorProfiles");
extern char msctf_export_anchor_14 __asm__("TF_CreateLangBarItemMgr");
extern char msctf_export_anchor_15 __asm__("TF_CreateLangBarMgr");
extern char msctf_export_anchor_16 __asm__("TF_CreateThreadMgr");
extern char msctf_export_anchor_17 __asm__("TF_DllDetachInOther");
extern char msctf_export_anchor_18 __asm__("TF_GetGlobalCompartment");
extern char msctf_export_anchor_19 __asm__("TF_GetInputScope");
extern char msctf_export_anchor_20 __asm__("TF_GetLangIcon");
extern char msctf_export_anchor_21 __asm__("TF_GetMlngHKL");
extern char msctf_export_anchor_22 __asm__("TF_GetMlngIconIndex");
extern char msctf_export_anchor_23 __asm__("TF_GetThreadFlags");
extern char msctf_export_anchor_24 __asm__("TF_GetThreadMgr");
extern char msctf_export_anchor_25 __asm__("TF_InatExtractIcon");
extern char msctf_export_anchor_26 __asm__("TF_InitMlngInfo");
extern char msctf_export_anchor_27 __asm__("TF_InitSystem");
extern char msctf_export_anchor_28 __asm__("TF_InvalidAssemblyListCache");
extern char msctf_export_anchor_29 __asm__("TF_InvalidAssemblyListCacheIfExist");
extern char msctf_export_anchor_30 __asm__("TF_IsCtfmonRunning");
extern char msctf_export_anchor_31 __asm__("TF_IsFullScreenWindowAcitvated");
extern char msctf_export_anchor_32 __asm__("TF_IsInMarshaling");
extern char msctf_export_anchor_33 __asm__("TF_MlngInfoCount");
extern char msctf_export_anchor_34 __asm__("TF_PostAllThreadMsg");
extern char msctf_export_anchor_35 __asm__("TF_RegisterLangBarAddIn");
extern char msctf_export_anchor_36 __asm__("TF_RunInputCPL");
extern char msctf_export_anchor_37 __asm__("TF_UninitSystem");
extern char msctf_export_anchor_38 __asm__("TF_UnregisterLangBarAddIn");
__attribute__((used)) static const void *const msctf_export_anchors[] = {
    &msctf_export_anchor_0,
    &msctf_export_anchor_1,
    &msctf_export_anchor_2,
    &msctf_export_anchor_3,
    &msctf_export_anchor_4,
    &msctf_export_anchor_5,
    &msctf_export_anchor_6,
    &msctf_export_anchor_7,
    &msctf_export_anchor_8,
    &msctf_export_anchor_9,
    &msctf_export_anchor_10,
    &msctf_export_anchor_11,
    &msctf_export_anchor_12,
    &msctf_export_anchor_13,
    &msctf_export_anchor_14,
    &msctf_export_anchor_15,
    &msctf_export_anchor_16,
    &msctf_export_anchor_17,
    &msctf_export_anchor_18,
    &msctf_export_anchor_19,
    &msctf_export_anchor_20,
    &msctf_export_anchor_21,
    &msctf_export_anchor_22,
    &msctf_export_anchor_23,
    &msctf_export_anchor_24,
    &msctf_export_anchor_25,
    &msctf_export_anchor_26,
    &msctf_export_anchor_27,
    &msctf_export_anchor_28,
    &msctf_export_anchor_29,
    &msctf_export_anchor_30,
    &msctf_export_anchor_31,
    &msctf_export_anchor_32,
    &msctf_export_anchor_33,
    &msctf_export_anchor_34,
    &msctf_export_anchor_35,
    &msctf_export_anchor_36,
    &msctf_export_anchor_37,
    &msctf_export_anchor_38,
    0,
};
#elif defined(__aarch64__)
extern char msctf_export_anchor_0 __asm__("DllCanUnloadNow");
extern char msctf_export_anchor_1 __asm__("DllGetClassObject");
extern char msctf_export_anchor_2 __asm__("DllRegisterServer");
extern char msctf_export_anchor_3 __asm__("DllUnregisterServer");
extern char msctf_export_anchor_4 __asm__("SetInputScope");
extern char msctf_export_anchor_5 __asm__("SetInputScopeXML");
extern char msctf_export_anchor_6 __asm__("SetInputScopes");
extern char msctf_export_anchor_7 __asm__("TF_CUASAppFix");
extern char msctf_export_anchor_8 __asm__("TF_CheckThreadInputIdle");
extern char msctf_export_anchor_9 __asm__("TF_ClearLangBarAddIns");
extern char msctf_export_anchor_10 __asm__("TF_CreateCategoryMgr");
extern char msctf_export_anchor_11 __asm__("TF_CreateCicLoadMutex");
extern char msctf_export_anchor_12 __asm__("TF_CreateDisplayAttributeMgr");
extern char msctf_export_anchor_13 __asm__("TF_CreateInputProcessorProfiles");
extern char msctf_export_anchor_14 __asm__("TF_CreateLangBarItemMgr");
extern char msctf_export_anchor_15 __asm__("TF_CreateLangBarMgr");
extern char msctf_export_anchor_16 __asm__("TF_CreateThreadMgr");
extern char msctf_export_anchor_17 __asm__("TF_DllDetachInOther");
extern char msctf_export_anchor_18 __asm__("TF_GetGlobalCompartment");
extern char msctf_export_anchor_19 __asm__("TF_GetInputScope");
extern char msctf_export_anchor_20 __asm__("TF_GetLangIcon");
extern char msctf_export_anchor_21 __asm__("TF_GetMlngHKL");
extern char msctf_export_anchor_22 __asm__("TF_GetMlngIconIndex");
extern char msctf_export_anchor_23 __asm__("TF_GetThreadFlags");
extern char msctf_export_anchor_24 __asm__("TF_GetThreadMgr");
extern char msctf_export_anchor_25 __asm__("TF_InatExtractIcon");
extern char msctf_export_anchor_26 __asm__("TF_InitMlngInfo");
extern char msctf_export_anchor_27 __asm__("TF_InitSystem");
extern char msctf_export_anchor_28 __asm__("TF_InvalidAssemblyListCache");
extern char msctf_export_anchor_29 __asm__("TF_InvalidAssemblyListCacheIfExist");
extern char msctf_export_anchor_30 __asm__("TF_IsCtfmonRunning");
extern char msctf_export_anchor_31 __asm__("TF_IsFullScreenWindowAcitvated");
extern char msctf_export_anchor_32 __asm__("TF_IsInMarshaling");
extern char msctf_export_anchor_33 __asm__("TF_MlngInfoCount");
extern char msctf_export_anchor_34 __asm__("TF_PostAllThreadMsg");
extern char msctf_export_anchor_35 __asm__("TF_RegisterLangBarAddIn");
extern char msctf_export_anchor_36 __asm__("TF_RunInputCPL");
extern char msctf_export_anchor_37 __asm__("TF_UninitSystem");
extern char msctf_export_anchor_38 __asm__("TF_UnregisterLangBarAddIn");
__attribute__((used)) static const void *const msctf_export_anchors[] = {
    &msctf_export_anchor_0,
    &msctf_export_anchor_1,
    &msctf_export_anchor_2,
    &msctf_export_anchor_3,
    &msctf_export_anchor_4,
    &msctf_export_anchor_5,
    &msctf_export_anchor_6,
    &msctf_export_anchor_7,
    &msctf_export_anchor_8,
    &msctf_export_anchor_9,
    &msctf_export_anchor_10,
    &msctf_export_anchor_11,
    &msctf_export_anchor_12,
    &msctf_export_anchor_13,
    &msctf_export_anchor_14,
    &msctf_export_anchor_15,
    &msctf_export_anchor_16,
    &msctf_export_anchor_17,
    &msctf_export_anchor_18,
    &msctf_export_anchor_19,
    &msctf_export_anchor_20,
    &msctf_export_anchor_21,
    &msctf_export_anchor_22,
    &msctf_export_anchor_23,
    &msctf_export_anchor_24,
    &msctf_export_anchor_25,
    &msctf_export_anchor_26,
    &msctf_export_anchor_27,
    &msctf_export_anchor_28,
    &msctf_export_anchor_29,
    &msctf_export_anchor_30,
    &msctf_export_anchor_31,
    &msctf_export_anchor_32,
    &msctf_export_anchor_33,
    &msctf_export_anchor_34,
    &msctf_export_anchor_35,
    &msctf_export_anchor_36,
    &msctf_export_anchor_37,
    &msctf_export_anchor_38,
    0,
};
#endif
