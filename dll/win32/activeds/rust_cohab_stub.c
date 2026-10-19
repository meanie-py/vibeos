/* activeds is Rust (sdk/lib/rust/activeds-rust); a CMake target needs one TU. */
typedef int activeds_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char activeds_export_anchor_0 __asm__("_ADsBuildEnumerator@8");
extern char activeds_export_anchor_1 __asm__("_ADsBuildVarArrayInt@12");
extern char activeds_export_anchor_2 __asm__("_ADsBuildVarArrayStr@12");
extern char activeds_export_anchor_3 __asm__("_ADsDecodeBinaryData@12");
extern char activeds_export_anchor_4 __asm__("_ADsEncodeBinaryData@12");
extern char activeds_export_anchor_5 __asm__("_ADsEnumerateNext@16");
extern char activeds_export_anchor_6 __asm__("_ADsFreeEnumerator@4");
extern char activeds_export_anchor_7 __asm__("_ADsGetLastError@20");
extern char activeds_export_anchor_8 __asm__("_ADsGetObject@12");
extern char activeds_export_anchor_9 __asm__("_ADsOpenObject@24");
extern char activeds_export_anchor_10 __asm__("_ADsSetLastError@12");
extern char activeds_export_anchor_11 __asm__("_AdsFreeAdsValues@8");
extern char activeds_export_anchor_12 __asm__("_AdsTypeToPropVariant2");
extern char activeds_export_anchor_13 __asm__("_AdsTypeToPropVariant@12");
extern char activeds_export_anchor_14 __asm__("_AllocADsMem@4");
extern char activeds_export_anchor_15 __asm__("_AllocADsStr@4");
extern char activeds_export_anchor_16 __asm__("_BinarySDToSecurityDescriptor@24");
extern char activeds_export_anchor_17 __asm__("_ConvertSecDescriptorToVariant");
extern char activeds_export_anchor_18 __asm__("_ConvertSecurityDescriptorToSecDes");
extern char activeds_export_anchor_19 __asm__("_DllCanUnloadNow@0");
extern char activeds_export_anchor_20 __asm__("_DllGetClassObject@12");
extern char activeds_export_anchor_21 __asm__("_DllRegisterServer@0");
extern char activeds_export_anchor_22 __asm__("_DllUnregisterServer@0");
extern char activeds_export_anchor_23 __asm__("_FreeADsMem@4");
extern char activeds_export_anchor_24 __asm__("_FreeADsStr@4");
extern char activeds_export_anchor_25 __asm__("_PropVariantToAdsType2");
extern char activeds_export_anchor_26 __asm__("_PropVariantToAdsType@16");
extern char activeds_export_anchor_27 __asm__("_ReallocADsMem@12");
extern char activeds_export_anchor_28 __asm__("_ReallocADsStr@8");
extern char activeds_export_anchor_29 __asm__("_SecurityDescriptorToBinarySD@40");
__attribute__((used)) static const void *const activeds_export_anchors[] = {
    &activeds_export_anchor_0,
    &activeds_export_anchor_1,
    &activeds_export_anchor_2,
    &activeds_export_anchor_3,
    &activeds_export_anchor_4,
    &activeds_export_anchor_5,
    &activeds_export_anchor_6,
    &activeds_export_anchor_7,
    &activeds_export_anchor_8,
    &activeds_export_anchor_9,
    &activeds_export_anchor_10,
    &activeds_export_anchor_11,
    &activeds_export_anchor_12,
    &activeds_export_anchor_13,
    &activeds_export_anchor_14,
    &activeds_export_anchor_15,
    &activeds_export_anchor_16,
    &activeds_export_anchor_17,
    &activeds_export_anchor_18,
    &activeds_export_anchor_19,
    &activeds_export_anchor_20,
    &activeds_export_anchor_21,
    &activeds_export_anchor_22,
    &activeds_export_anchor_23,
    &activeds_export_anchor_24,
    &activeds_export_anchor_25,
    &activeds_export_anchor_26,
    &activeds_export_anchor_27,
    &activeds_export_anchor_28,
    &activeds_export_anchor_29,
    0,
};
#elif defined(__x86_64__)
extern char activeds_export_anchor_0 __asm__("ADsBuildEnumerator");
extern char activeds_export_anchor_1 __asm__("ADsBuildVarArrayInt");
extern char activeds_export_anchor_2 __asm__("ADsBuildVarArrayStr");
extern char activeds_export_anchor_3 __asm__("ADsDecodeBinaryData");
extern char activeds_export_anchor_4 __asm__("ADsEncodeBinaryData");
extern char activeds_export_anchor_5 __asm__("ADsEnumerateNext");
extern char activeds_export_anchor_6 __asm__("ADsFreeEnumerator");
extern char activeds_export_anchor_7 __asm__("ADsGetLastError");
extern char activeds_export_anchor_8 __asm__("ADsGetObject");
extern char activeds_export_anchor_9 __asm__("ADsOpenObject");
extern char activeds_export_anchor_10 __asm__("ADsSetLastError");
extern char activeds_export_anchor_11 __asm__("AdsFreeAdsValues");
extern char activeds_export_anchor_12 __asm__("AdsTypeToPropVariant");
extern char activeds_export_anchor_13 __asm__("AdsTypeToPropVariant2");
extern char activeds_export_anchor_14 __asm__("AllocADsMem");
extern char activeds_export_anchor_15 __asm__("AllocADsStr");
extern char activeds_export_anchor_16 __asm__("BinarySDToSecurityDescriptor");
extern char activeds_export_anchor_17 __asm__("ConvertSecDescriptorToVariant");
extern char activeds_export_anchor_18 __asm__("ConvertSecurityDescriptorToSecDes");
extern char activeds_export_anchor_19 __asm__("DllCanUnloadNow");
extern char activeds_export_anchor_20 __asm__("DllGetClassObject");
extern char activeds_export_anchor_21 __asm__("DllRegisterServer");
extern char activeds_export_anchor_22 __asm__("DllUnregisterServer");
extern char activeds_export_anchor_23 __asm__("FreeADsMem");
extern char activeds_export_anchor_24 __asm__("FreeADsStr");
extern char activeds_export_anchor_25 __asm__("PropVariantToAdsType");
extern char activeds_export_anchor_26 __asm__("PropVariantToAdsType2");
extern char activeds_export_anchor_27 __asm__("ReallocADsMem");
extern char activeds_export_anchor_28 __asm__("ReallocADsStr");
extern char activeds_export_anchor_29 __asm__("SecurityDescriptorToBinarySD");
__attribute__((used)) static const void *const activeds_export_anchors[] = {
    &activeds_export_anchor_0,
    &activeds_export_anchor_1,
    &activeds_export_anchor_2,
    &activeds_export_anchor_3,
    &activeds_export_anchor_4,
    &activeds_export_anchor_5,
    &activeds_export_anchor_6,
    &activeds_export_anchor_7,
    &activeds_export_anchor_8,
    &activeds_export_anchor_9,
    &activeds_export_anchor_10,
    &activeds_export_anchor_11,
    &activeds_export_anchor_12,
    &activeds_export_anchor_13,
    &activeds_export_anchor_14,
    &activeds_export_anchor_15,
    &activeds_export_anchor_16,
    &activeds_export_anchor_17,
    &activeds_export_anchor_18,
    &activeds_export_anchor_19,
    &activeds_export_anchor_20,
    &activeds_export_anchor_21,
    &activeds_export_anchor_22,
    &activeds_export_anchor_23,
    &activeds_export_anchor_24,
    &activeds_export_anchor_25,
    &activeds_export_anchor_26,
    &activeds_export_anchor_27,
    &activeds_export_anchor_28,
    &activeds_export_anchor_29,
    0,
};
#elif defined(__aarch64__)
extern char activeds_export_anchor_0 __asm__("ADsBuildEnumerator");
extern char activeds_export_anchor_1 __asm__("ADsBuildVarArrayInt");
extern char activeds_export_anchor_2 __asm__("ADsBuildVarArrayStr");
extern char activeds_export_anchor_3 __asm__("ADsDecodeBinaryData");
extern char activeds_export_anchor_4 __asm__("ADsEncodeBinaryData");
extern char activeds_export_anchor_5 __asm__("ADsEnumerateNext");
extern char activeds_export_anchor_6 __asm__("ADsFreeEnumerator");
extern char activeds_export_anchor_7 __asm__("ADsGetLastError");
extern char activeds_export_anchor_8 __asm__("ADsGetObject");
extern char activeds_export_anchor_9 __asm__("ADsOpenObject");
extern char activeds_export_anchor_10 __asm__("ADsSetLastError");
extern char activeds_export_anchor_11 __asm__("AdsFreeAdsValues");
extern char activeds_export_anchor_12 __asm__("AdsTypeToPropVariant");
extern char activeds_export_anchor_13 __asm__("AdsTypeToPropVariant2");
extern char activeds_export_anchor_14 __asm__("AllocADsMem");
extern char activeds_export_anchor_15 __asm__("AllocADsStr");
extern char activeds_export_anchor_16 __asm__("BinarySDToSecurityDescriptor");
extern char activeds_export_anchor_17 __asm__("ConvertSecDescriptorToVariant");
extern char activeds_export_anchor_18 __asm__("ConvertSecurityDescriptorToSecDes");
extern char activeds_export_anchor_19 __asm__("DllCanUnloadNow");
extern char activeds_export_anchor_20 __asm__("DllGetClassObject");
extern char activeds_export_anchor_21 __asm__("DllRegisterServer");
extern char activeds_export_anchor_22 __asm__("DllUnregisterServer");
extern char activeds_export_anchor_23 __asm__("FreeADsMem");
extern char activeds_export_anchor_24 __asm__("FreeADsStr");
extern char activeds_export_anchor_25 __asm__("PropVariantToAdsType");
extern char activeds_export_anchor_26 __asm__("PropVariantToAdsType2");
extern char activeds_export_anchor_27 __asm__("ReallocADsMem");
extern char activeds_export_anchor_28 __asm__("ReallocADsStr");
extern char activeds_export_anchor_29 __asm__("SecurityDescriptorToBinarySD");
__attribute__((used)) static const void *const activeds_export_anchors[] = {
    &activeds_export_anchor_0,
    &activeds_export_anchor_1,
    &activeds_export_anchor_2,
    &activeds_export_anchor_3,
    &activeds_export_anchor_4,
    &activeds_export_anchor_5,
    &activeds_export_anchor_6,
    &activeds_export_anchor_7,
    &activeds_export_anchor_8,
    &activeds_export_anchor_9,
    &activeds_export_anchor_10,
    &activeds_export_anchor_11,
    &activeds_export_anchor_12,
    &activeds_export_anchor_13,
    &activeds_export_anchor_14,
    &activeds_export_anchor_15,
    &activeds_export_anchor_16,
    &activeds_export_anchor_17,
    &activeds_export_anchor_18,
    &activeds_export_anchor_19,
    &activeds_export_anchor_20,
    &activeds_export_anchor_21,
    &activeds_export_anchor_22,
    &activeds_export_anchor_23,
    &activeds_export_anchor_24,
    &activeds_export_anchor_25,
    &activeds_export_anchor_26,
    &activeds_export_anchor_27,
    &activeds_export_anchor_28,
    &activeds_export_anchor_29,
    0,
};
#endif
