/* propsys is Rust (sdk/lib/rust/propsys-rust); a CMake target needs one TU. */
typedef int propsys_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char propsys_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char propsys_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char propsys_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char propsys_export_anchor_3 __asm__("_DllUnregisterServer@0");
extern char propsys_export_anchor_4 __asm__("_InitPropVariantFromBuffer@12");
extern char propsys_export_anchor_5 __asm__("_InitPropVariantFromCLSID@8");
extern char propsys_export_anchor_6 __asm__("_InitPropVariantFromGUIDAsString@8");
extern char propsys_export_anchor_7 __asm__("_InitPropVariantFromStringVector@12");
extern char propsys_export_anchor_8 __asm__("_InitVariantFromBuffer@12");
extern char propsys_export_anchor_9 __asm__("_InitVariantFromFileTime@8");
extern char propsys_export_anchor_10 __asm__("_InitVariantFromGUIDAsString@8");
extern char propsys_export_anchor_11 __asm__("_PSCreateMemoryPropertyStore@8");
extern char propsys_export_anchor_12 __asm__("_PSCreatePropertyStoreFromObject@16");
extern char propsys_export_anchor_13 __asm__("_PSGetPropertyDescription@12");
extern char propsys_export_anchor_14 __asm__("_PSGetPropertyDescriptionListFromString@12");
extern char propsys_export_anchor_15 __asm__("_PSGetPropertyKeyFromName@8");
extern char propsys_export_anchor_16 __asm__("_PSGetPropertySystem@8");
extern char propsys_export_anchor_17 __asm__("_PSPropertyKeyFromString@8");
extern char propsys_export_anchor_18 __asm__("_PSRefreshPropertySchema@0");
extern char propsys_export_anchor_19 __asm__("_PSRegisterPropertySchema@4");
extern char propsys_export_anchor_20 __asm__("_PSStringFromPropertyKey@12");
extern char propsys_export_anchor_21 __asm__("_PSUnregisterPropertySchema@4");
extern char propsys_export_anchor_22 __asm__("_PropVariantChangeType@16");
extern char propsys_export_anchor_23 __asm__("_PropVariantCompareEx@16");
extern char propsys_export_anchor_24 __asm__("_PropVariantToBSTR@8");
extern char propsys_export_anchor_25 __asm__("_PropVariantToBoolean@8");
extern char propsys_export_anchor_26 __asm__("_PropVariantToBuffer@12");
extern char propsys_export_anchor_27 __asm__("_PropVariantToDouble@8");
extern char propsys_export_anchor_28 __asm__("_PropVariantToGUID@8");
extern char propsys_export_anchor_29 __asm__("_PropVariantToInt16@8");
extern char propsys_export_anchor_30 __asm__("_PropVariantToInt32@8");
extern char propsys_export_anchor_31 __asm__("_PropVariantToInt64@8");
extern char propsys_export_anchor_32 __asm__("_PropVariantToString@12");
extern char propsys_export_anchor_33 __asm__("_PropVariantToStringAlloc@8");
extern char propsys_export_anchor_34 __asm__("_PropVariantToStringWithDefault@8");
extern char propsys_export_anchor_35 __asm__("_PropVariantToUInt16@8");
extern char propsys_export_anchor_36 __asm__("_PropVariantToUInt32@8");
extern char propsys_export_anchor_37 __asm__("_PropVariantToUInt32WithDefault@8");
extern char propsys_export_anchor_38 __asm__("_PropVariantToUInt64@8");
extern char propsys_export_anchor_39 __asm__("_PropVariantToVariant@8");
extern char propsys_export_anchor_40 __asm__("_VariantCompare@8");
extern char propsys_export_anchor_41 __asm__("_VariantToGUID@8");
extern char propsys_export_anchor_42 __asm__("_VariantToPropVariant@8");
extern char propsys_export_anchor_43 __asm__("_VariantToString@12");
extern char propsys_export_anchor_44 __asm__("_VariantToStringWithDefault@8");
__attribute__((used)) static const void *const propsys_export_anchors[] = {
    &propsys_export_anchor_0,
    &propsys_export_anchor_1,
    &propsys_export_anchor_2,
    &propsys_export_anchor_3,
    &propsys_export_anchor_4,
    &propsys_export_anchor_5,
    &propsys_export_anchor_6,
    &propsys_export_anchor_7,
    &propsys_export_anchor_8,
    &propsys_export_anchor_9,
    &propsys_export_anchor_10,
    &propsys_export_anchor_11,
    &propsys_export_anchor_12,
    &propsys_export_anchor_13,
    &propsys_export_anchor_14,
    &propsys_export_anchor_15,
    &propsys_export_anchor_16,
    &propsys_export_anchor_17,
    &propsys_export_anchor_18,
    &propsys_export_anchor_19,
    &propsys_export_anchor_20,
    &propsys_export_anchor_21,
    &propsys_export_anchor_22,
    &propsys_export_anchor_23,
    &propsys_export_anchor_24,
    &propsys_export_anchor_25,
    &propsys_export_anchor_26,
    &propsys_export_anchor_27,
    &propsys_export_anchor_28,
    &propsys_export_anchor_29,
    &propsys_export_anchor_30,
    &propsys_export_anchor_31,
    &propsys_export_anchor_32,
    &propsys_export_anchor_33,
    &propsys_export_anchor_34,
    &propsys_export_anchor_35,
    &propsys_export_anchor_36,
    &propsys_export_anchor_37,
    &propsys_export_anchor_38,
    &propsys_export_anchor_39,
    &propsys_export_anchor_40,
    &propsys_export_anchor_41,
    &propsys_export_anchor_42,
    &propsys_export_anchor_43,
    &propsys_export_anchor_44,
    0,
};
#elif defined(__x86_64__)
extern char propsys_export_anchor_0 __asm__("DllCanUnloadNow");
extern char propsys_export_anchor_1 __asm__("DllGetClassObject");
extern char propsys_export_anchor_2 __asm__("DllRegisterServer");
extern char propsys_export_anchor_3 __asm__("DllUnregisterServer");
extern char propsys_export_anchor_4 __asm__("InitPropVariantFromBuffer");
extern char propsys_export_anchor_5 __asm__("InitPropVariantFromCLSID");
extern char propsys_export_anchor_6 __asm__("InitPropVariantFromGUIDAsString");
extern char propsys_export_anchor_7 __asm__("InitPropVariantFromStringVector");
extern char propsys_export_anchor_8 __asm__("InitVariantFromBuffer");
extern char propsys_export_anchor_9 __asm__("InitVariantFromFileTime");
extern char propsys_export_anchor_10 __asm__("InitVariantFromGUIDAsString");
extern char propsys_export_anchor_11 __asm__("PSCreateMemoryPropertyStore");
extern char propsys_export_anchor_12 __asm__("PSCreatePropertyStoreFromObject");
extern char propsys_export_anchor_13 __asm__("PSGetPropertyDescription");
extern char propsys_export_anchor_14 __asm__("PSGetPropertyDescriptionListFromString");
extern char propsys_export_anchor_15 __asm__("PSGetPropertyKeyFromName");
extern char propsys_export_anchor_16 __asm__("PSGetPropertySystem");
extern char propsys_export_anchor_17 __asm__("PSPropertyKeyFromString");
extern char propsys_export_anchor_18 __asm__("PSRefreshPropertySchema");
extern char propsys_export_anchor_19 __asm__("PSRegisterPropertySchema");
extern char propsys_export_anchor_20 __asm__("PSStringFromPropertyKey");
extern char propsys_export_anchor_21 __asm__("PSUnregisterPropertySchema");
extern char propsys_export_anchor_22 __asm__("PropVariantChangeType");
extern char propsys_export_anchor_23 __asm__("PropVariantCompareEx");
extern char propsys_export_anchor_24 __asm__("PropVariantToBSTR");
extern char propsys_export_anchor_25 __asm__("PropVariantToBoolean");
extern char propsys_export_anchor_26 __asm__("PropVariantToBuffer");
extern char propsys_export_anchor_27 __asm__("PropVariantToDouble");
extern char propsys_export_anchor_28 __asm__("PropVariantToGUID");
extern char propsys_export_anchor_29 __asm__("PropVariantToInt16");
extern char propsys_export_anchor_30 __asm__("PropVariantToInt32");
extern char propsys_export_anchor_31 __asm__("PropVariantToInt64");
extern char propsys_export_anchor_32 __asm__("PropVariantToString");
extern char propsys_export_anchor_33 __asm__("PropVariantToStringAlloc");
extern char propsys_export_anchor_34 __asm__("PropVariantToStringWithDefault");
extern char propsys_export_anchor_35 __asm__("PropVariantToUInt16");
extern char propsys_export_anchor_36 __asm__("PropVariantToUInt32");
extern char propsys_export_anchor_37 __asm__("PropVariantToUInt32WithDefault");
extern char propsys_export_anchor_38 __asm__("PropVariantToUInt64");
extern char propsys_export_anchor_39 __asm__("PropVariantToVariant");
extern char propsys_export_anchor_40 __asm__("VariantCompare");
extern char propsys_export_anchor_41 __asm__("VariantToGUID");
extern char propsys_export_anchor_42 __asm__("VariantToPropVariant");
extern char propsys_export_anchor_43 __asm__("VariantToString");
extern char propsys_export_anchor_44 __asm__("VariantToStringWithDefault");
__attribute__((used)) static const void *const propsys_export_anchors[] = {
    &propsys_export_anchor_0,
    &propsys_export_anchor_1,
    &propsys_export_anchor_2,
    &propsys_export_anchor_3,
    &propsys_export_anchor_4,
    &propsys_export_anchor_5,
    &propsys_export_anchor_6,
    &propsys_export_anchor_7,
    &propsys_export_anchor_8,
    &propsys_export_anchor_9,
    &propsys_export_anchor_10,
    &propsys_export_anchor_11,
    &propsys_export_anchor_12,
    &propsys_export_anchor_13,
    &propsys_export_anchor_14,
    &propsys_export_anchor_15,
    &propsys_export_anchor_16,
    &propsys_export_anchor_17,
    &propsys_export_anchor_18,
    &propsys_export_anchor_19,
    &propsys_export_anchor_20,
    &propsys_export_anchor_21,
    &propsys_export_anchor_22,
    &propsys_export_anchor_23,
    &propsys_export_anchor_24,
    &propsys_export_anchor_25,
    &propsys_export_anchor_26,
    &propsys_export_anchor_27,
    &propsys_export_anchor_28,
    &propsys_export_anchor_29,
    &propsys_export_anchor_30,
    &propsys_export_anchor_31,
    &propsys_export_anchor_32,
    &propsys_export_anchor_33,
    &propsys_export_anchor_34,
    &propsys_export_anchor_35,
    &propsys_export_anchor_36,
    &propsys_export_anchor_37,
    &propsys_export_anchor_38,
    &propsys_export_anchor_39,
    &propsys_export_anchor_40,
    &propsys_export_anchor_41,
    &propsys_export_anchor_42,
    &propsys_export_anchor_43,
    &propsys_export_anchor_44,
    0,
};
#elif defined(__aarch64__)
extern char propsys_export_anchor_0 __asm__("DllCanUnloadNow");
extern char propsys_export_anchor_1 __asm__("DllGetClassObject");
extern char propsys_export_anchor_2 __asm__("DllRegisterServer");
extern char propsys_export_anchor_3 __asm__("DllUnregisterServer");
extern char propsys_export_anchor_4 __asm__("InitPropVariantFromBuffer");
extern char propsys_export_anchor_5 __asm__("InitPropVariantFromCLSID");
extern char propsys_export_anchor_6 __asm__("InitPropVariantFromGUIDAsString");
extern char propsys_export_anchor_7 __asm__("InitPropVariantFromStringVector");
extern char propsys_export_anchor_8 __asm__("InitVariantFromBuffer");
extern char propsys_export_anchor_9 __asm__("InitVariantFromFileTime");
extern char propsys_export_anchor_10 __asm__("InitVariantFromGUIDAsString");
extern char propsys_export_anchor_11 __asm__("PSCreateMemoryPropertyStore");
extern char propsys_export_anchor_12 __asm__("PSCreatePropertyStoreFromObject");
extern char propsys_export_anchor_13 __asm__("PSGetPropertyDescription");
extern char propsys_export_anchor_14 __asm__("PSGetPropertyDescriptionListFromString");
extern char propsys_export_anchor_15 __asm__("PSGetPropertyKeyFromName");
extern char propsys_export_anchor_16 __asm__("PSGetPropertySystem");
extern char propsys_export_anchor_17 __asm__("PSPropertyKeyFromString");
extern char propsys_export_anchor_18 __asm__("PSRefreshPropertySchema");
extern char propsys_export_anchor_19 __asm__("PSRegisterPropertySchema");
extern char propsys_export_anchor_20 __asm__("PSStringFromPropertyKey");
extern char propsys_export_anchor_21 __asm__("PSUnregisterPropertySchema");
extern char propsys_export_anchor_22 __asm__("PropVariantChangeType");
extern char propsys_export_anchor_23 __asm__("PropVariantCompareEx");
extern char propsys_export_anchor_24 __asm__("PropVariantToBSTR");
extern char propsys_export_anchor_25 __asm__("PropVariantToBoolean");
extern char propsys_export_anchor_26 __asm__("PropVariantToBuffer");
extern char propsys_export_anchor_27 __asm__("PropVariantToDouble");
extern char propsys_export_anchor_28 __asm__("PropVariantToGUID");
extern char propsys_export_anchor_29 __asm__("PropVariantToInt16");
extern char propsys_export_anchor_30 __asm__("PropVariantToInt32");
extern char propsys_export_anchor_31 __asm__("PropVariantToInt64");
extern char propsys_export_anchor_32 __asm__("PropVariantToString");
extern char propsys_export_anchor_33 __asm__("PropVariantToStringAlloc");
extern char propsys_export_anchor_34 __asm__("PropVariantToStringWithDefault");
extern char propsys_export_anchor_35 __asm__("PropVariantToUInt16");
extern char propsys_export_anchor_36 __asm__("PropVariantToUInt32");
extern char propsys_export_anchor_37 __asm__("PropVariantToUInt32WithDefault");
extern char propsys_export_anchor_38 __asm__("PropVariantToUInt64");
extern char propsys_export_anchor_39 __asm__("PropVariantToVariant");
extern char propsys_export_anchor_40 __asm__("VariantCompare");
extern char propsys_export_anchor_41 __asm__("VariantToGUID");
extern char propsys_export_anchor_42 __asm__("VariantToPropVariant");
extern char propsys_export_anchor_43 __asm__("VariantToString");
extern char propsys_export_anchor_44 __asm__("VariantToStringWithDefault");
__attribute__((used)) static const void *const propsys_export_anchors[] = {
    &propsys_export_anchor_0,
    &propsys_export_anchor_1,
    &propsys_export_anchor_2,
    &propsys_export_anchor_3,
    &propsys_export_anchor_4,
    &propsys_export_anchor_5,
    &propsys_export_anchor_6,
    &propsys_export_anchor_7,
    &propsys_export_anchor_8,
    &propsys_export_anchor_9,
    &propsys_export_anchor_10,
    &propsys_export_anchor_11,
    &propsys_export_anchor_12,
    &propsys_export_anchor_13,
    &propsys_export_anchor_14,
    &propsys_export_anchor_15,
    &propsys_export_anchor_16,
    &propsys_export_anchor_17,
    &propsys_export_anchor_18,
    &propsys_export_anchor_19,
    &propsys_export_anchor_20,
    &propsys_export_anchor_21,
    &propsys_export_anchor_22,
    &propsys_export_anchor_23,
    &propsys_export_anchor_24,
    &propsys_export_anchor_25,
    &propsys_export_anchor_26,
    &propsys_export_anchor_27,
    &propsys_export_anchor_28,
    &propsys_export_anchor_29,
    &propsys_export_anchor_30,
    &propsys_export_anchor_31,
    &propsys_export_anchor_32,
    &propsys_export_anchor_33,
    &propsys_export_anchor_34,
    &propsys_export_anchor_35,
    &propsys_export_anchor_36,
    &propsys_export_anchor_37,
    &propsys_export_anchor_38,
    &propsys_export_anchor_39,
    &propsys_export_anchor_40,
    &propsys_export_anchor_41,
    &propsys_export_anchor_42,
    &propsys_export_anchor_43,
    &propsys_export_anchor_44,
    0,
};
#endif
