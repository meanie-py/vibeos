/* shcore is Rust (sdk/lib/rust/shcore-rust); a CMake target needs one TU. */
typedef int shcore_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char shcore_export_anchor_0 __asm__("_CommandLineToArgvW@8");
extern char shcore_export_anchor_1 __asm__("_CreateRandomAccessStreamOverStream@16");
extern char shcore_export_anchor_2 __asm__("_DllCanUnloadNow@0");
extern char shcore_export_anchor_3 __asm__("_GetCurrentProcessExplicitAppUserModelID@4");
extern char shcore_export_anchor_4 __asm__("_GetFeatureEnabledState@8");
extern char shcore_export_anchor_5 __asm__("_GetProcessReference@4");
extern char shcore_export_anchor_6 __asm__("_GetScaleFactorForDevice@4");
extern char shcore_export_anchor_7 __asm__("_IStream_Reset@4");
extern char shcore_export_anchor_8 __asm__("_IStream_Size@8");
extern char shcore_export_anchor_9 __asm__("_IUnknown_AtomicRelease@4");
extern char shcore_export_anchor_10 __asm__("_IUnknown_GetSite@12");
extern char shcore_export_anchor_11 __asm__("_IUnknown_QueryService@16");
extern char shcore_export_anchor_12 __asm__("_IUnknown_Set@8");
extern char shcore_export_anchor_13 __asm__("_IUnknown_SetSite@8");
extern char shcore_export_anchor_14 __asm__("_IsOS@4");
extern char shcore_export_anchor_15 __asm__("_RegisterScaleChangeEvent@8");
extern char shcore_export_anchor_16 __asm__("_RegisterScaleChangeNotifications@16");
extern char shcore_export_anchor_17 __asm__("_SHAnsiToAnsi@12");
extern char shcore_export_anchor_18 __asm__("_SHAnsiToUnicode@12");
extern char shcore_export_anchor_19 __asm__("_SHCopyKeyA@16");
extern char shcore_export_anchor_20 __asm__("_SHCopyKeyW@16");
extern char shcore_export_anchor_21 __asm__("_SHCreateMemStream@8");
extern char shcore_export_anchor_22 __asm__("_SHCreateStreamOnFileA@12");
extern char shcore_export_anchor_23 __asm__("_SHCreateStreamOnFileEx@24");
extern char shcore_export_anchor_24 __asm__("_SHCreateStreamOnFileW@12");
extern char shcore_export_anchor_25 __asm__("_SHCreateThread@16");
extern char shcore_export_anchor_26 __asm__("_SHCreateThreadRef@8");
extern char shcore_export_anchor_27 __asm__("_SHDeleteEmptyKeyA@8");
extern char shcore_export_anchor_28 __asm__("_SHDeleteEmptyKeyW@8");
extern char shcore_export_anchor_29 __asm__("_SHDeleteKeyA@8");
extern char shcore_export_anchor_30 __asm__("_SHDeleteKeyW@8");
extern char shcore_export_anchor_31 __asm__("_SHDeleteValueA@12");
extern char shcore_export_anchor_32 __asm__("_SHDeleteValueW@12");
extern char shcore_export_anchor_33 __asm__("_SHEnumKeyExA@16");
extern char shcore_export_anchor_34 __asm__("_SHEnumKeyExW@16");
extern char shcore_export_anchor_35 __asm__("_SHEnumValueA@28");
extern char shcore_export_anchor_36 __asm__("_SHEnumValueW@28");
extern char shcore_export_anchor_37 __asm__("_SHGetThreadRef@4");
extern char shcore_export_anchor_38 __asm__("_SHGetValueA@24");
extern char shcore_export_anchor_39 __asm__("_SHGetValueW@24");
extern char shcore_export_anchor_40 __asm__("_SHOpenRegStream2A@16");
extern char shcore_export_anchor_41 __asm__("_SHOpenRegStream2W@16");
extern char shcore_export_anchor_42 __asm__("_SHOpenRegStreamA@16");
extern char shcore_export_anchor_43 __asm__("_SHOpenRegStreamW@16");
extern char shcore_export_anchor_44 __asm__("_SHQueryInfoKeyA@20");
extern char shcore_export_anchor_45 __asm__("_SHQueryInfoKeyW@20");
extern char shcore_export_anchor_46 __asm__("_SHQueryValueExA@24");
extern char shcore_export_anchor_47 __asm__("_SHQueryValueExW@24");
extern char shcore_export_anchor_48 __asm__("_SHRegDuplicateHKey@4");
extern char shcore_export_anchor_49 __asm__("_SHRegGetIntW@12");
extern char shcore_export_anchor_50 __asm__("_SHRegGetPathA@20");
extern char shcore_export_anchor_51 __asm__("_SHRegGetPathW@20");
extern char shcore_export_anchor_52 __asm__("_SHRegSetPathA@20");
extern char shcore_export_anchor_53 __asm__("_SHRegSetPathW@20");
extern char shcore_export_anchor_54 __asm__("_SHReleaseThreadRef@0");
extern char shcore_export_anchor_55 __asm__("_SHSetThreadRef@4");
extern char shcore_export_anchor_56 __asm__("_SHSetValueA@24");
extern char shcore_export_anchor_57 __asm__("_SHSetValueW@24");
extern char shcore_export_anchor_58 __asm__("_SHStrDupA@8");
extern char shcore_export_anchor_59 __asm__("_SHStrDupW@8");
extern char shcore_export_anchor_60 __asm__("_SHUnicodeToAnsi@12");
extern char shcore_export_anchor_61 __asm__("_SHUnicodeToUnicode@12");
extern char shcore_export_anchor_62 __asm__("_SetCurrentProcessExplicitAppUserModelID@4");
extern char shcore_export_anchor_63 __asm__("_SetProcessReference@4");
extern char shcore_export_anchor_64 __asm__("_SubscribeFeatureStateChangeNotification@12");
extern char shcore_export_anchor_65 __asm__("__IStream_Read@12");
extern char shcore_export_anchor_66 __asm__("__IStream_Write@12");
__attribute__((used)) static const void *const shcore_export_anchors[] = {
    &shcore_export_anchor_0,
    &shcore_export_anchor_1,
    &shcore_export_anchor_2,
    &shcore_export_anchor_3,
    &shcore_export_anchor_4,
    &shcore_export_anchor_5,
    &shcore_export_anchor_6,
    &shcore_export_anchor_7,
    &shcore_export_anchor_8,
    &shcore_export_anchor_9,
    &shcore_export_anchor_10,
    &shcore_export_anchor_11,
    &shcore_export_anchor_12,
    &shcore_export_anchor_13,
    &shcore_export_anchor_14,
    &shcore_export_anchor_15,
    &shcore_export_anchor_16,
    &shcore_export_anchor_17,
    &shcore_export_anchor_18,
    &shcore_export_anchor_19,
    &shcore_export_anchor_20,
    &shcore_export_anchor_21,
    &shcore_export_anchor_22,
    &shcore_export_anchor_23,
    &shcore_export_anchor_24,
    &shcore_export_anchor_25,
    &shcore_export_anchor_26,
    &shcore_export_anchor_27,
    &shcore_export_anchor_28,
    &shcore_export_anchor_29,
    &shcore_export_anchor_30,
    &shcore_export_anchor_31,
    &shcore_export_anchor_32,
    &shcore_export_anchor_33,
    &shcore_export_anchor_34,
    &shcore_export_anchor_35,
    &shcore_export_anchor_36,
    &shcore_export_anchor_37,
    &shcore_export_anchor_38,
    &shcore_export_anchor_39,
    &shcore_export_anchor_40,
    &shcore_export_anchor_41,
    &shcore_export_anchor_42,
    &shcore_export_anchor_43,
    &shcore_export_anchor_44,
    &shcore_export_anchor_45,
    &shcore_export_anchor_46,
    &shcore_export_anchor_47,
    &shcore_export_anchor_48,
    &shcore_export_anchor_49,
    &shcore_export_anchor_50,
    &shcore_export_anchor_51,
    &shcore_export_anchor_52,
    &shcore_export_anchor_53,
    &shcore_export_anchor_54,
    &shcore_export_anchor_55,
    &shcore_export_anchor_56,
    &shcore_export_anchor_57,
    &shcore_export_anchor_58,
    &shcore_export_anchor_59,
    &shcore_export_anchor_60,
    &shcore_export_anchor_61,
    &shcore_export_anchor_62,
    &shcore_export_anchor_63,
    &shcore_export_anchor_64,
    &shcore_export_anchor_65,
    &shcore_export_anchor_66,
    0,
};
#elif defined(__x86_64__)
extern char shcore_export_anchor_0 __asm__("CommandLineToArgvW");
extern char shcore_export_anchor_1 __asm__("CreateRandomAccessStreamOverStream");
extern char shcore_export_anchor_2 __asm__("DllCanUnloadNow");
extern char shcore_export_anchor_3 __asm__("GetCurrentProcessExplicitAppUserModelID");
extern char shcore_export_anchor_4 __asm__("GetFeatureEnabledState");
extern char shcore_export_anchor_5 __asm__("GetProcessReference");
extern char shcore_export_anchor_6 __asm__("GetScaleFactorForDevice");
extern char shcore_export_anchor_7 __asm__("IStream_Reset");
extern char shcore_export_anchor_8 __asm__("IStream_Size");
extern char shcore_export_anchor_9 __asm__("IUnknown_AtomicRelease");
extern char shcore_export_anchor_10 __asm__("IUnknown_GetSite");
extern char shcore_export_anchor_11 __asm__("IUnknown_QueryService");
extern char shcore_export_anchor_12 __asm__("IUnknown_Set");
extern char shcore_export_anchor_13 __asm__("IUnknown_SetSite");
extern char shcore_export_anchor_14 __asm__("IsOS");
extern char shcore_export_anchor_15 __asm__("RegisterScaleChangeEvent");
extern char shcore_export_anchor_16 __asm__("RegisterScaleChangeNotifications");
extern char shcore_export_anchor_17 __asm__("SHAnsiToAnsi");
extern char shcore_export_anchor_18 __asm__("SHAnsiToUnicode");
extern char shcore_export_anchor_19 __asm__("SHCopyKeyA");
extern char shcore_export_anchor_20 __asm__("SHCopyKeyW");
extern char shcore_export_anchor_21 __asm__("SHCreateMemStream");
extern char shcore_export_anchor_22 __asm__("SHCreateStreamOnFileA");
extern char shcore_export_anchor_23 __asm__("SHCreateStreamOnFileEx");
extern char shcore_export_anchor_24 __asm__("SHCreateStreamOnFileW");
extern char shcore_export_anchor_25 __asm__("SHCreateThread");
extern char shcore_export_anchor_26 __asm__("SHCreateThreadRef");
extern char shcore_export_anchor_27 __asm__("SHDeleteEmptyKeyA");
extern char shcore_export_anchor_28 __asm__("SHDeleteEmptyKeyW");
extern char shcore_export_anchor_29 __asm__("SHDeleteKeyA");
extern char shcore_export_anchor_30 __asm__("SHDeleteKeyW");
extern char shcore_export_anchor_31 __asm__("SHDeleteValueA");
extern char shcore_export_anchor_32 __asm__("SHDeleteValueW");
extern char shcore_export_anchor_33 __asm__("SHEnumKeyExA");
extern char shcore_export_anchor_34 __asm__("SHEnumKeyExW");
extern char shcore_export_anchor_35 __asm__("SHEnumValueA");
extern char shcore_export_anchor_36 __asm__("SHEnumValueW");
extern char shcore_export_anchor_37 __asm__("SHGetThreadRef");
extern char shcore_export_anchor_38 __asm__("SHGetValueA");
extern char shcore_export_anchor_39 __asm__("SHGetValueW");
extern char shcore_export_anchor_40 __asm__("SHOpenRegStream2A");
extern char shcore_export_anchor_41 __asm__("SHOpenRegStream2W");
extern char shcore_export_anchor_42 __asm__("SHOpenRegStreamA");
extern char shcore_export_anchor_43 __asm__("SHOpenRegStreamW");
extern char shcore_export_anchor_44 __asm__("SHQueryInfoKeyA");
extern char shcore_export_anchor_45 __asm__("SHQueryInfoKeyW");
extern char shcore_export_anchor_46 __asm__("SHQueryValueExA");
extern char shcore_export_anchor_47 __asm__("SHQueryValueExW");
extern char shcore_export_anchor_48 __asm__("SHRegDuplicateHKey");
extern char shcore_export_anchor_49 __asm__("SHRegGetIntW");
extern char shcore_export_anchor_50 __asm__("SHRegGetPathA");
extern char shcore_export_anchor_51 __asm__("SHRegGetPathW");
extern char shcore_export_anchor_52 __asm__("SHRegSetPathA");
extern char shcore_export_anchor_53 __asm__("SHRegSetPathW");
extern char shcore_export_anchor_54 __asm__("SHReleaseThreadRef");
extern char shcore_export_anchor_55 __asm__("SHSetThreadRef");
extern char shcore_export_anchor_56 __asm__("SHSetValueA");
extern char shcore_export_anchor_57 __asm__("SHSetValueW");
extern char shcore_export_anchor_58 __asm__("SHStrDupA");
extern char shcore_export_anchor_59 __asm__("SHStrDupW");
extern char shcore_export_anchor_60 __asm__("SHUnicodeToAnsi");
extern char shcore_export_anchor_61 __asm__("SHUnicodeToUnicode");
extern char shcore_export_anchor_62 __asm__("SetCurrentProcessExplicitAppUserModelID");
extern char shcore_export_anchor_63 __asm__("SetProcessReference");
extern char shcore_export_anchor_64 __asm__("SubscribeFeatureStateChangeNotification");
extern char shcore_export_anchor_65 __asm__("_IStream_Read");
extern char shcore_export_anchor_66 __asm__("_IStream_Write");
__attribute__((used)) static const void *const shcore_export_anchors[] = {
    &shcore_export_anchor_0,
    &shcore_export_anchor_1,
    &shcore_export_anchor_2,
    &shcore_export_anchor_3,
    &shcore_export_anchor_4,
    &shcore_export_anchor_5,
    &shcore_export_anchor_6,
    &shcore_export_anchor_7,
    &shcore_export_anchor_8,
    &shcore_export_anchor_9,
    &shcore_export_anchor_10,
    &shcore_export_anchor_11,
    &shcore_export_anchor_12,
    &shcore_export_anchor_13,
    &shcore_export_anchor_14,
    &shcore_export_anchor_15,
    &shcore_export_anchor_16,
    &shcore_export_anchor_17,
    &shcore_export_anchor_18,
    &shcore_export_anchor_19,
    &shcore_export_anchor_20,
    &shcore_export_anchor_21,
    &shcore_export_anchor_22,
    &shcore_export_anchor_23,
    &shcore_export_anchor_24,
    &shcore_export_anchor_25,
    &shcore_export_anchor_26,
    &shcore_export_anchor_27,
    &shcore_export_anchor_28,
    &shcore_export_anchor_29,
    &shcore_export_anchor_30,
    &shcore_export_anchor_31,
    &shcore_export_anchor_32,
    &shcore_export_anchor_33,
    &shcore_export_anchor_34,
    &shcore_export_anchor_35,
    &shcore_export_anchor_36,
    &shcore_export_anchor_37,
    &shcore_export_anchor_38,
    &shcore_export_anchor_39,
    &shcore_export_anchor_40,
    &shcore_export_anchor_41,
    &shcore_export_anchor_42,
    &shcore_export_anchor_43,
    &shcore_export_anchor_44,
    &shcore_export_anchor_45,
    &shcore_export_anchor_46,
    &shcore_export_anchor_47,
    &shcore_export_anchor_48,
    &shcore_export_anchor_49,
    &shcore_export_anchor_50,
    &shcore_export_anchor_51,
    &shcore_export_anchor_52,
    &shcore_export_anchor_53,
    &shcore_export_anchor_54,
    &shcore_export_anchor_55,
    &shcore_export_anchor_56,
    &shcore_export_anchor_57,
    &shcore_export_anchor_58,
    &shcore_export_anchor_59,
    &shcore_export_anchor_60,
    &shcore_export_anchor_61,
    &shcore_export_anchor_62,
    &shcore_export_anchor_63,
    &shcore_export_anchor_64,
    &shcore_export_anchor_65,
    &shcore_export_anchor_66,
    0,
};
#elif defined(__aarch64__)
extern char shcore_export_anchor_0 __asm__("CommandLineToArgvW");
extern char shcore_export_anchor_1 __asm__("CreateRandomAccessStreamOverStream");
extern char shcore_export_anchor_2 __asm__("DllCanUnloadNow");
extern char shcore_export_anchor_3 __asm__("GetCurrentProcessExplicitAppUserModelID");
extern char shcore_export_anchor_4 __asm__("GetFeatureEnabledState");
extern char shcore_export_anchor_5 __asm__("GetProcessReference");
extern char shcore_export_anchor_6 __asm__("GetScaleFactorForDevice");
extern char shcore_export_anchor_7 __asm__("IStream_Reset");
extern char shcore_export_anchor_8 __asm__("IStream_Size");
extern char shcore_export_anchor_9 __asm__("IUnknown_AtomicRelease");
extern char shcore_export_anchor_10 __asm__("IUnknown_GetSite");
extern char shcore_export_anchor_11 __asm__("IUnknown_QueryService");
extern char shcore_export_anchor_12 __asm__("IUnknown_Set");
extern char shcore_export_anchor_13 __asm__("IUnknown_SetSite");
extern char shcore_export_anchor_14 __asm__("IsOS");
extern char shcore_export_anchor_15 __asm__("RegisterScaleChangeEvent");
extern char shcore_export_anchor_16 __asm__("RegisterScaleChangeNotifications");
extern char shcore_export_anchor_17 __asm__("SHAnsiToAnsi");
extern char shcore_export_anchor_18 __asm__("SHAnsiToUnicode");
extern char shcore_export_anchor_19 __asm__("SHCopyKeyA");
extern char shcore_export_anchor_20 __asm__("SHCopyKeyW");
extern char shcore_export_anchor_21 __asm__("SHCreateMemStream");
extern char shcore_export_anchor_22 __asm__("SHCreateStreamOnFileA");
extern char shcore_export_anchor_23 __asm__("SHCreateStreamOnFileEx");
extern char shcore_export_anchor_24 __asm__("SHCreateStreamOnFileW");
extern char shcore_export_anchor_25 __asm__("SHCreateThread");
extern char shcore_export_anchor_26 __asm__("SHCreateThreadRef");
extern char shcore_export_anchor_27 __asm__("SHDeleteEmptyKeyA");
extern char shcore_export_anchor_28 __asm__("SHDeleteEmptyKeyW");
extern char shcore_export_anchor_29 __asm__("SHDeleteKeyA");
extern char shcore_export_anchor_30 __asm__("SHDeleteKeyW");
extern char shcore_export_anchor_31 __asm__("SHDeleteValueA");
extern char shcore_export_anchor_32 __asm__("SHDeleteValueW");
extern char shcore_export_anchor_33 __asm__("SHEnumKeyExA");
extern char shcore_export_anchor_34 __asm__("SHEnumKeyExW");
extern char shcore_export_anchor_35 __asm__("SHEnumValueA");
extern char shcore_export_anchor_36 __asm__("SHEnumValueW");
extern char shcore_export_anchor_37 __asm__("SHGetThreadRef");
extern char shcore_export_anchor_38 __asm__("SHGetValueA");
extern char shcore_export_anchor_39 __asm__("SHGetValueW");
extern char shcore_export_anchor_40 __asm__("SHOpenRegStream2A");
extern char shcore_export_anchor_41 __asm__("SHOpenRegStream2W");
extern char shcore_export_anchor_42 __asm__("SHOpenRegStreamA");
extern char shcore_export_anchor_43 __asm__("SHOpenRegStreamW");
extern char shcore_export_anchor_44 __asm__("SHQueryInfoKeyA");
extern char shcore_export_anchor_45 __asm__("SHQueryInfoKeyW");
extern char shcore_export_anchor_46 __asm__("SHQueryValueExA");
extern char shcore_export_anchor_47 __asm__("SHQueryValueExW");
extern char shcore_export_anchor_48 __asm__("SHRegDuplicateHKey");
extern char shcore_export_anchor_49 __asm__("SHRegGetIntW");
extern char shcore_export_anchor_50 __asm__("SHRegGetPathA");
extern char shcore_export_anchor_51 __asm__("SHRegGetPathW");
extern char shcore_export_anchor_52 __asm__("SHRegSetPathA");
extern char shcore_export_anchor_53 __asm__("SHRegSetPathW");
extern char shcore_export_anchor_54 __asm__("SHReleaseThreadRef");
extern char shcore_export_anchor_55 __asm__("SHSetThreadRef");
extern char shcore_export_anchor_56 __asm__("SHSetValueA");
extern char shcore_export_anchor_57 __asm__("SHSetValueW");
extern char shcore_export_anchor_58 __asm__("SHStrDupA");
extern char shcore_export_anchor_59 __asm__("SHStrDupW");
extern char shcore_export_anchor_60 __asm__("SHUnicodeToAnsi");
extern char shcore_export_anchor_61 __asm__("SHUnicodeToUnicode");
extern char shcore_export_anchor_62 __asm__("SetCurrentProcessExplicitAppUserModelID");
extern char shcore_export_anchor_63 __asm__("SetProcessReference");
extern char shcore_export_anchor_64 __asm__("SubscribeFeatureStateChangeNotification");
extern char shcore_export_anchor_65 __asm__("_IStream_Read");
extern char shcore_export_anchor_66 __asm__("_IStream_Write");
__attribute__((used)) static const void *const shcore_export_anchors[] = {
    &shcore_export_anchor_0,
    &shcore_export_anchor_1,
    &shcore_export_anchor_2,
    &shcore_export_anchor_3,
    &shcore_export_anchor_4,
    &shcore_export_anchor_5,
    &shcore_export_anchor_6,
    &shcore_export_anchor_7,
    &shcore_export_anchor_8,
    &shcore_export_anchor_9,
    &shcore_export_anchor_10,
    &shcore_export_anchor_11,
    &shcore_export_anchor_12,
    &shcore_export_anchor_13,
    &shcore_export_anchor_14,
    &shcore_export_anchor_15,
    &shcore_export_anchor_16,
    &shcore_export_anchor_17,
    &shcore_export_anchor_18,
    &shcore_export_anchor_19,
    &shcore_export_anchor_20,
    &shcore_export_anchor_21,
    &shcore_export_anchor_22,
    &shcore_export_anchor_23,
    &shcore_export_anchor_24,
    &shcore_export_anchor_25,
    &shcore_export_anchor_26,
    &shcore_export_anchor_27,
    &shcore_export_anchor_28,
    &shcore_export_anchor_29,
    &shcore_export_anchor_30,
    &shcore_export_anchor_31,
    &shcore_export_anchor_32,
    &shcore_export_anchor_33,
    &shcore_export_anchor_34,
    &shcore_export_anchor_35,
    &shcore_export_anchor_36,
    &shcore_export_anchor_37,
    &shcore_export_anchor_38,
    &shcore_export_anchor_39,
    &shcore_export_anchor_40,
    &shcore_export_anchor_41,
    &shcore_export_anchor_42,
    &shcore_export_anchor_43,
    &shcore_export_anchor_44,
    &shcore_export_anchor_45,
    &shcore_export_anchor_46,
    &shcore_export_anchor_47,
    &shcore_export_anchor_48,
    &shcore_export_anchor_49,
    &shcore_export_anchor_50,
    &shcore_export_anchor_51,
    &shcore_export_anchor_52,
    &shcore_export_anchor_53,
    &shcore_export_anchor_54,
    &shcore_export_anchor_55,
    &shcore_export_anchor_56,
    &shcore_export_anchor_57,
    &shcore_export_anchor_58,
    &shcore_export_anchor_59,
    &shcore_export_anchor_60,
    &shcore_export_anchor_61,
    &shcore_export_anchor_62,
    &shcore_export_anchor_63,
    &shcore_export_anchor_64,
    &shcore_export_anchor_65,
    &shcore_export_anchor_66,
    0,
};
#endif
