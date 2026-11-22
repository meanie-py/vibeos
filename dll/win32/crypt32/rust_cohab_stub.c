/* crypt32 is Rust (sdk/lib/rust/crypt32-rust); a CMake target needs one TU. */
typedef int crypt32_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char crypt32_export_anchor_0 __asm__("_CertAddCRLContextToStore@16");
extern char crypt32_export_anchor_1 __asm__("_CertAddCTLContextToStore@16");
extern char crypt32_export_anchor_2 __asm__("_CertAddCertificateContextToStore@16");
extern char crypt32_export_anchor_3 __asm__("_CertAddCertificateLinkToStore@16");
extern char crypt32_export_anchor_4 __asm__("_CertAddEncodedCRLToStore@24");
extern char crypt32_export_anchor_5 __asm__("_CertAddEncodedCTLToStore@24");
extern char crypt32_export_anchor_6 __asm__("_CertAddEncodedCertificateToStore@24");
extern char crypt32_export_anchor_7 __asm__("_CertAddEncodedCertificateToSystemStoreA@12");
extern char crypt32_export_anchor_8 __asm__("_CertAddEncodedCertificateToSystemStoreW@12");
extern char crypt32_export_anchor_9 __asm__("_CertAddEnhancedKeyUsageIdentifier@8");
extern char crypt32_export_anchor_10 __asm__("_CertAddSerializedElementToStore@32");
extern char crypt32_export_anchor_11 __asm__("_CertAddStoreToCollection@16");
extern char crypt32_export_anchor_12 __asm__("_CertAlgIdToOID@4");
extern char crypt32_export_anchor_13 __asm__("_CertCloseStore@8");
extern char crypt32_export_anchor_14 __asm__("_CertCompareCertificate@12");
extern char crypt32_export_anchor_15 __asm__("_CertCompareCertificateName@12");
extern char crypt32_export_anchor_16 __asm__("_CertCompareIntegerBlob@8");
extern char crypt32_export_anchor_17 __asm__("_CertComparePublicKeyInfo@12");
extern char crypt32_export_anchor_18 __asm__("_CertControlStore@16");
extern char crypt32_export_anchor_19 __asm__("_CertCreateCRLContext@12");
extern char crypt32_export_anchor_20 __asm__("_CertCreateCTLContext@12");
extern char crypt32_export_anchor_21 __asm__("_CertCreateCertificateChainEngine@8");
extern char crypt32_export_anchor_22 __asm__("_CertCreateCertificateContext@12");
extern char crypt32_export_anchor_23 __asm__("_CertCreateContext@24");
extern char crypt32_export_anchor_24 __asm__("_CertCreateSelfSignCertificate@32");
extern char crypt32_export_anchor_25 __asm__("_CertDeleteCRLFromStore@4");
extern char crypt32_export_anchor_26 __asm__("_CertDeleteCTLFromStore@4");
extern char crypt32_export_anchor_27 __asm__("_CertDeleteCertificateFromStore@4");
extern char crypt32_export_anchor_28 __asm__("_CertDuplicateCRLContext@4");
extern char crypt32_export_anchor_29 __asm__("_CertDuplicateCTLContext@4");
extern char crypt32_export_anchor_30 __asm__("_CertDuplicateCertificateChain@4");
extern char crypt32_export_anchor_31 __asm__("_CertDuplicateCertificateContext@4");
extern char crypt32_export_anchor_32 __asm__("_CertDuplicateStore@4");
extern char crypt32_export_anchor_33 __asm__("_CertEnumCRLContextProperties@8");
extern char crypt32_export_anchor_34 __asm__("_CertEnumCRLsInStore@8");
extern char crypt32_export_anchor_35 __asm__("_CertEnumCTLContextProperties@8");
extern char crypt32_export_anchor_36 __asm__("_CertEnumCTLsInStore@8");
extern char crypt32_export_anchor_37 __asm__("_CertEnumCertificateContextProperties@8");
extern char crypt32_export_anchor_38 __asm__("_CertEnumCertificatesInStore@8");
extern char crypt32_export_anchor_39 __asm__("_CertEnumPhysicalStore@16");
extern char crypt32_export_anchor_40 __asm__("_CertEnumSystemStore@16");
extern char crypt32_export_anchor_41 __asm__("_CertEnumSystemStoreLocation@12");
extern char crypt32_export_anchor_42 __asm__("_CertFindAttribute@12");
extern char crypt32_export_anchor_43 __asm__("_CertFindCRLInStore@24");
extern char crypt32_export_anchor_44 __asm__("_CertFindCTLInStore@24");
extern char crypt32_export_anchor_45 __asm__("_CertFindCertificateInCRL@20");
extern char crypt32_export_anchor_46 __asm__("_CertFindCertificateInStore@24");
extern char crypt32_export_anchor_47 __asm__("_CertFindChainInStore@24");
extern char crypt32_export_anchor_48 __asm__("_CertFindExtension@12");
extern char crypt32_export_anchor_49 __asm__("_CertFindRDNAttr@8");
extern char crypt32_export_anchor_50 __asm__("_CertFreeCRLContext@4");
extern char crypt32_export_anchor_51 __asm__("_CertFreeCTLContext@4");
extern char crypt32_export_anchor_52 __asm__("_CertFreeCertificateChain@4");
extern char crypt32_export_anchor_53 __asm__("_CertFreeCertificateChainEngine@4");
extern char crypt32_export_anchor_54 __asm__("_CertFreeCertificateContext@4");
extern char crypt32_export_anchor_55 __asm__("_CertGetCRLContextProperty@16");
extern char crypt32_export_anchor_56 __asm__("_CertGetCRLFromStore@16");
extern char crypt32_export_anchor_57 __asm__("_CertGetCTLContextProperty@16");
extern char crypt32_export_anchor_58 __asm__("_CertGetCertificateChain@32");
extern char crypt32_export_anchor_59 __asm__("_CertGetCertificateContextProperty@16");
extern char crypt32_export_anchor_60 __asm__("_CertGetEnhancedKeyUsage@16");
extern char crypt32_export_anchor_61 __asm__("_CertGetIntendedKeyUsage@16");
extern char crypt32_export_anchor_62 __asm__("_CertGetIssuerCertificateFromStore@16");
extern char crypt32_export_anchor_63 __asm__("_CertGetNameStringA@24");
extern char crypt32_export_anchor_64 __asm__("_CertGetNameStringW@24");
extern char crypt32_export_anchor_65 __asm__("_CertGetPublicKeyLength@8");
extern char crypt32_export_anchor_66 __asm__("_CertGetStoreProperty@16");
extern char crypt32_export_anchor_67 __asm__("_CertGetSubjectCertificateFromStore@12");
extern char crypt32_export_anchor_68 __asm__("_CertGetValidUsages@20");
extern char crypt32_export_anchor_69 __asm__("_CertIsRDNAttrsInCertificateName@16");
extern char crypt32_export_anchor_70 __asm__("_CertIsValidCRLForCertificate@16");
extern char crypt32_export_anchor_71 __asm__("_CertNameToStrA@20");
extern char crypt32_export_anchor_72 __asm__("_CertNameToStrW@20");
extern char crypt32_export_anchor_73 __asm__("_CertOIDToAlgId@4");
extern char crypt32_export_anchor_74 __asm__("_CertOpenStore@20");
extern char crypt32_export_anchor_75 __asm__("_CertOpenSystemStoreA@8");
extern char crypt32_export_anchor_76 __asm__("_CertOpenSystemStoreW@8");
extern char crypt32_export_anchor_77 __asm__("_CertRDNValueToStrA@16");
extern char crypt32_export_anchor_78 __asm__("_CertRDNValueToStrW@16");
extern char crypt32_export_anchor_79 __asm__("_CertRegisterPhysicalStore@20");
extern char crypt32_export_anchor_80 __asm__("_CertRegisterSystemStore@16");
extern char crypt32_export_anchor_81 __asm__("_CertRemoveEnhancedKeyUsageIdentifier@8");
extern char crypt32_export_anchor_82 __asm__("_CertRemoveStoreFromCollection@8");
extern char crypt32_export_anchor_83 __asm__("_CertSaveStore@24");
extern char crypt32_export_anchor_84 __asm__("_CertSerializeCRLStoreElement@16");
extern char crypt32_export_anchor_85 __asm__("_CertSerializeCTLStoreElement@16");
extern char crypt32_export_anchor_86 __asm__("_CertSerializeCertificateStoreElement@16");
extern char crypt32_export_anchor_87 __asm__("_CertSetCRLContextProperty@16");
extern char crypt32_export_anchor_88 __asm__("_CertSetCTLContextProperty@16");
extern char crypt32_export_anchor_89 __asm__("_CertSetCertificateContextProperty@16");
extern char crypt32_export_anchor_90 __asm__("_CertSetEnhancedKeyUsage@8");
extern char crypt32_export_anchor_91 __asm__("_CertSetStoreProperty@16");
extern char crypt32_export_anchor_92 __asm__("_CertStrToNameA@28");
extern char crypt32_export_anchor_93 __asm__("_CertStrToNameW@28");
extern char crypt32_export_anchor_94 __asm__("_CertUnregisterPhysicalStore@12");
extern char crypt32_export_anchor_95 __asm__("_CertUnregisterSystemStore@8");
extern char crypt32_export_anchor_96 __asm__("_CertVerifyCRLRevocation@16");
extern char crypt32_export_anchor_97 __asm__("_CertVerifyCRLTimeValidity@8");
extern char crypt32_export_anchor_98 __asm__("_CertVerifyCTLUsage@28");
extern char crypt32_export_anchor_99 __asm__("_CertVerifyCertificateChainPolicy@16");
extern char crypt32_export_anchor_100 __asm__("_CertVerifyRevocation@28");
extern char crypt32_export_anchor_101 __asm__("_CertVerifySubjectCertificateContext@12");
extern char crypt32_export_anchor_102 __asm__("_CertVerifyTimeValidity@8");
extern char crypt32_export_anchor_103 __asm__("_CertVerifyValidityNesting@8");
extern char crypt32_export_anchor_104 __asm__("_CryptAcquireCertificatePrivateKey@24");
extern char crypt32_export_anchor_105 __asm__("_CryptBinaryToStringA@20");
extern char crypt32_export_anchor_106 __asm__("_CryptBinaryToStringW@20");
extern char crypt32_export_anchor_107 __asm__("_CryptDecodeObject@28");
extern char crypt32_export_anchor_108 __asm__("_CryptDecodeObjectEx@32");
extern char crypt32_export_anchor_109 __asm__("_CryptEncodeObject@20");
extern char crypt32_export_anchor_110 __asm__("_CryptEncodeObjectEx@28");
extern char crypt32_export_anchor_111 __asm__("_CryptEncryptMessage@28");
extern char crypt32_export_anchor_112 __asm__("_CryptEnumOIDInfo@16");
extern char crypt32_export_anchor_113 __asm__("_CryptExportPublicKeyInfo@20");
extern char crypt32_export_anchor_114 __asm__("_CryptExportPublicKeyInfoEx@32");
extern char crypt32_export_anchor_115 __asm__("_CryptFindCertificateKeyProvInfo@12");
extern char crypt32_export_anchor_116 __asm__("_CryptFindLocalizedName@4");
extern char crypt32_export_anchor_117 __asm__("_CryptFindOIDInfo@12");
extern char crypt32_export_anchor_118 __asm__("_CryptFormatObject@36");
extern char crypt32_export_anchor_119 __asm__("_CryptFreeOIDFunctionAddress@8");
extern char crypt32_export_anchor_120 __asm__("_CryptGetDefaultOIDDllList@16");
extern char crypt32_export_anchor_121 __asm__("_CryptGetDefaultOIDFunctionAddress@24");
extern char crypt32_export_anchor_122 __asm__("_CryptGetMessageCertificates@20");
extern char crypt32_export_anchor_123 __asm__("_CryptGetMessageSignerCount@12");
extern char crypt32_export_anchor_124 __asm__("_CryptGetOIDFunctionAddress@24");
extern char crypt32_export_anchor_125 __asm__("_CryptGetOIDFunctionValue@28");
extern char crypt32_export_anchor_126 __asm__("_CryptHashCertificate2@28");
extern char crypt32_export_anchor_127 __asm__("_CryptHashCertificate@28");
extern char crypt32_export_anchor_128 __asm__("_CryptHashMessage@36");
extern char crypt32_export_anchor_129 __asm__("_CryptHashPublicKeyInfo@28");
extern char crypt32_export_anchor_130 __asm__("_CryptHashToBeSigned@24");
extern char crypt32_export_anchor_131 __asm__("_CryptImportPublicKeyInfo@16");
extern char crypt32_export_anchor_132 __asm__("_CryptImportPublicKeyInfoEx2@20");
extern char crypt32_export_anchor_133 __asm__("_CryptImportPublicKeyInfoEx@28");
extern char crypt32_export_anchor_134 __asm__("_CryptInitOIDFunctionSet@8");
extern char crypt32_export_anchor_135 __asm__("_CryptInstallOIDFunctionAddress@24");
extern char crypt32_export_anchor_136 __asm__("_CryptMemAlloc@4");
extern char crypt32_export_anchor_137 __asm__("_CryptMemFree@4");
extern char crypt32_export_anchor_138 __asm__("_CryptMemRealloc@8");
extern char crypt32_export_anchor_139 __asm__("_CryptMsgClose@4");
extern char crypt32_export_anchor_140 __asm__("_CryptMsgControl@16");
extern char crypt32_export_anchor_141 __asm__("_CryptMsgDuplicate@4");
extern char crypt32_export_anchor_142 __asm__("_CryptMsgEncodeAndSignCTL@24");
extern char crypt32_export_anchor_143 __asm__("_CryptMsgGetAndVerifySigner@24");
extern char crypt32_export_anchor_144 __asm__("_CryptMsgGetParam@20");
extern char crypt32_export_anchor_145 __asm__("_CryptMsgOpenToDecode@24");
extern char crypt32_export_anchor_146 __asm__("_CryptMsgOpenToEncode@24");
extern char crypt32_export_anchor_147 __asm__("_CryptMsgSignCTL@28");
extern char crypt32_export_anchor_148 __asm__("_CryptMsgUpdate@16");
extern char crypt32_export_anchor_149 __asm__("_CryptMsgVerifyCountersignatureEncoded@28");
extern char crypt32_export_anchor_150 __asm__("_CryptMsgVerifyCountersignatureEncodedEx@40");
extern char crypt32_export_anchor_151 __asm__("_CryptProtectData@28");
extern char crypt32_export_anchor_152 __asm__("_CryptProtectMemory@12");
extern char crypt32_export_anchor_153 __asm__("_CryptQueryObject@44");
extern char crypt32_export_anchor_154 __asm__("_CryptRegisterDefaultOIDFunction@16");
extern char crypt32_export_anchor_155 __asm__("_CryptRegisterOIDFunction@20");
extern char crypt32_export_anchor_156 __asm__("_CryptRegisterOIDInfo@8");
extern char crypt32_export_anchor_157 __asm__("_CryptSIPAddProvider@4");
extern char crypt32_export_anchor_158 __asm__("_CryptSIPCreateIndirectData@12");
extern char crypt32_export_anchor_159 __asm__("_CryptSIPGetSignedDataMsg@20");
extern char crypt32_export_anchor_160 __asm__("_CryptSIPLoad@12");
extern char crypt32_export_anchor_161 __asm__("_CryptSIPPutSignedDataMsg@20");
extern char crypt32_export_anchor_162 __asm__("_CryptSIPRemoveProvider@4");
extern char crypt32_export_anchor_163 __asm__("_CryptSIPRemoveSignedDataMsg@8");
extern char crypt32_export_anchor_164 __asm__("_CryptSIPRetrieveSubjectGuid@12");
extern char crypt32_export_anchor_165 __asm__("_CryptSIPRetrieveSubjectGuidForCatalogFile@12");
extern char crypt32_export_anchor_166 __asm__("_CryptSIPVerifyIndirectData@8");
extern char crypt32_export_anchor_167 __asm__("_CryptSetKeyIdentifierProperty@24");
extern char crypt32_export_anchor_168 __asm__("_CryptSetOIDFunctionValue@28");
extern char crypt32_export_anchor_169 __asm__("_CryptSignAndEncodeCertificate@36");
extern char crypt32_export_anchor_170 __asm__("_CryptSignCertificate@36");
extern char crypt32_export_anchor_171 __asm__("_CryptSignMessage@28");
extern char crypt32_export_anchor_172 __asm__("_CryptStringToBinaryA@28");
extern char crypt32_export_anchor_173 __asm__("_CryptStringToBinaryW@28");
extern char crypt32_export_anchor_174 __asm__("_CryptUnprotectData@28");
extern char crypt32_export_anchor_175 __asm__("_CryptUnprotectMemory@12");
extern char crypt32_export_anchor_176 __asm__("_CryptUnregisterDefaultOIDFunction@12");
extern char crypt32_export_anchor_177 __asm__("_CryptUnregisterOIDFunction@12");
extern char crypt32_export_anchor_178 __asm__("_CryptUnregisterOIDInfo@4");
extern char crypt32_export_anchor_179 __asm__("_CryptVerifyCertificateSignature@20");
extern char crypt32_export_anchor_180 __asm__("_CryptVerifyCertificateSignatureEx@32");
extern char crypt32_export_anchor_181 __asm__("_CryptVerifyDetachedMessageHash@32");
extern char crypt32_export_anchor_182 __asm__("_CryptVerifyDetachedMessageSignature@32");
extern char crypt32_export_anchor_183 __asm__("_CryptVerifyMessageHash@28");
extern char crypt32_export_anchor_184 __asm__("_CryptVerifyMessageSignature@28");
extern char crypt32_export_anchor_185 __asm__("_I_CertUpdateStore@16");
extern char crypt32_export_anchor_186 __asm__("_I_CryptAllocTls@0");
extern char crypt32_export_anchor_187 __asm__("_I_CryptCreateLruCache@8");
extern char crypt32_export_anchor_188 __asm__("_I_CryptCreateLruEntry@12");
extern char crypt32_export_anchor_189 __asm__("_I_CryptDetachTls@4");
extern char crypt32_export_anchor_190 __asm__("_I_CryptFindLruEntry@8");
extern char crypt32_export_anchor_191 __asm__("_I_CryptFindLruEntryData@12");
extern char crypt32_export_anchor_192 __asm__("_I_CryptFlushLruCache@12");
extern char crypt32_export_anchor_193 __asm__("_I_CryptFreeLruCache@12");
extern char crypt32_export_anchor_194 __asm__("_I_CryptFreeTls@8");
extern char crypt32_export_anchor_195 __asm__("_I_CryptGetAsn1Decoder@4");
extern char crypt32_export_anchor_196 __asm__("_I_CryptGetAsn1Encoder@4");
extern char crypt32_export_anchor_197 __asm__("_I_CryptGetDefaultCryptProv@4");
extern char crypt32_export_anchor_198 __asm__("_I_CryptGetOssGlobal@4");
extern char crypt32_export_anchor_199 __asm__("_I_CryptGetTls@4");
extern char crypt32_export_anchor_200 __asm__("_I_CryptInstallAsn1Module@12");
extern char crypt32_export_anchor_201 __asm__("_I_CryptInstallOssGlobal@12");
extern char crypt32_export_anchor_202 __asm__("_I_CryptReadTrustedPublisherDWORDValueFromRegistry@8");
extern char crypt32_export_anchor_203 __asm__("_I_CryptSetTls@8");
extern char crypt32_export_anchor_204 __asm__("_I_CryptUninstallAsn1Module@4");
extern char crypt32_export_anchor_205 __asm__("_PFXExportCertStore@16");
extern char crypt32_export_anchor_206 __asm__("_PFXExportCertStoreEx@20");
extern char crypt32_export_anchor_207 __asm__("_PFXImportCertStore@12");
extern char crypt32_export_anchor_208 __asm__("_PFXIsPFXBlob@4");
extern char crypt32_export_anchor_209 __asm__("_PFXVerifyPassword@12");
__attribute__((used)) static const void *const crypt32_export_anchors[] = {
    &crypt32_export_anchor_0,
    &crypt32_export_anchor_1,
    &crypt32_export_anchor_2,
    &crypt32_export_anchor_3,
    &crypt32_export_anchor_4,
    &crypt32_export_anchor_5,
    &crypt32_export_anchor_6,
    &crypt32_export_anchor_7,
    &crypt32_export_anchor_8,
    &crypt32_export_anchor_9,
    &crypt32_export_anchor_10,
    &crypt32_export_anchor_11,
    &crypt32_export_anchor_12,
    &crypt32_export_anchor_13,
    &crypt32_export_anchor_14,
    &crypt32_export_anchor_15,
    &crypt32_export_anchor_16,
    &crypt32_export_anchor_17,
    &crypt32_export_anchor_18,
    &crypt32_export_anchor_19,
    &crypt32_export_anchor_20,
    &crypt32_export_anchor_21,
    &crypt32_export_anchor_22,
    &crypt32_export_anchor_23,
    &crypt32_export_anchor_24,
    &crypt32_export_anchor_25,
    &crypt32_export_anchor_26,
    &crypt32_export_anchor_27,
    &crypt32_export_anchor_28,
    &crypt32_export_anchor_29,
    &crypt32_export_anchor_30,
    &crypt32_export_anchor_31,
    &crypt32_export_anchor_32,
    &crypt32_export_anchor_33,
    &crypt32_export_anchor_34,
    &crypt32_export_anchor_35,
    &crypt32_export_anchor_36,
    &crypt32_export_anchor_37,
    &crypt32_export_anchor_38,
    &crypt32_export_anchor_39,
    &crypt32_export_anchor_40,
    &crypt32_export_anchor_41,
    &crypt32_export_anchor_42,
    &crypt32_export_anchor_43,
    &crypt32_export_anchor_44,
    &crypt32_export_anchor_45,
    &crypt32_export_anchor_46,
    &crypt32_export_anchor_47,
    &crypt32_export_anchor_48,
    &crypt32_export_anchor_49,
    &crypt32_export_anchor_50,
    &crypt32_export_anchor_51,
    &crypt32_export_anchor_52,
    &crypt32_export_anchor_53,
    &crypt32_export_anchor_54,
    &crypt32_export_anchor_55,
    &crypt32_export_anchor_56,
    &crypt32_export_anchor_57,
    &crypt32_export_anchor_58,
    &crypt32_export_anchor_59,
    &crypt32_export_anchor_60,
    &crypt32_export_anchor_61,
    &crypt32_export_anchor_62,
    &crypt32_export_anchor_63,
    &crypt32_export_anchor_64,
    &crypt32_export_anchor_65,
    &crypt32_export_anchor_66,
    &crypt32_export_anchor_67,
    &crypt32_export_anchor_68,
    &crypt32_export_anchor_69,
    &crypt32_export_anchor_70,
    &crypt32_export_anchor_71,
    &crypt32_export_anchor_72,
    &crypt32_export_anchor_73,
    &crypt32_export_anchor_74,
    &crypt32_export_anchor_75,
    &crypt32_export_anchor_76,
    &crypt32_export_anchor_77,
    &crypt32_export_anchor_78,
    &crypt32_export_anchor_79,
    &crypt32_export_anchor_80,
    &crypt32_export_anchor_81,
    &crypt32_export_anchor_82,
    &crypt32_export_anchor_83,
    &crypt32_export_anchor_84,
    &crypt32_export_anchor_85,
    &crypt32_export_anchor_86,
    &crypt32_export_anchor_87,
    &crypt32_export_anchor_88,
    &crypt32_export_anchor_89,
    &crypt32_export_anchor_90,
    &crypt32_export_anchor_91,
    &crypt32_export_anchor_92,
    &crypt32_export_anchor_93,
    &crypt32_export_anchor_94,
    &crypt32_export_anchor_95,
    &crypt32_export_anchor_96,
    &crypt32_export_anchor_97,
    &crypt32_export_anchor_98,
    &crypt32_export_anchor_99,
    &crypt32_export_anchor_100,
    &crypt32_export_anchor_101,
    &crypt32_export_anchor_102,
    &crypt32_export_anchor_103,
    &crypt32_export_anchor_104,
    &crypt32_export_anchor_105,
    &crypt32_export_anchor_106,
    &crypt32_export_anchor_107,
    &crypt32_export_anchor_108,
    &crypt32_export_anchor_109,
    &crypt32_export_anchor_110,
    &crypt32_export_anchor_111,
    &crypt32_export_anchor_112,
    &crypt32_export_anchor_113,
    &crypt32_export_anchor_114,
    &crypt32_export_anchor_115,
    &crypt32_export_anchor_116,
    &crypt32_export_anchor_117,
    &crypt32_export_anchor_118,
    &crypt32_export_anchor_119,
    &crypt32_export_anchor_120,
    &crypt32_export_anchor_121,
    &crypt32_export_anchor_122,
    &crypt32_export_anchor_123,
    &crypt32_export_anchor_124,
    &crypt32_export_anchor_125,
    &crypt32_export_anchor_126,
    &crypt32_export_anchor_127,
    &crypt32_export_anchor_128,
    &crypt32_export_anchor_129,
    &crypt32_export_anchor_130,
    &crypt32_export_anchor_131,
    &crypt32_export_anchor_132,
    &crypt32_export_anchor_133,
    &crypt32_export_anchor_134,
    &crypt32_export_anchor_135,
    &crypt32_export_anchor_136,
    &crypt32_export_anchor_137,
    &crypt32_export_anchor_138,
    &crypt32_export_anchor_139,
    &crypt32_export_anchor_140,
    &crypt32_export_anchor_141,
    &crypt32_export_anchor_142,
    &crypt32_export_anchor_143,
    &crypt32_export_anchor_144,
    &crypt32_export_anchor_145,
    &crypt32_export_anchor_146,
    &crypt32_export_anchor_147,
    &crypt32_export_anchor_148,
    &crypt32_export_anchor_149,
    &crypt32_export_anchor_150,
    &crypt32_export_anchor_151,
    &crypt32_export_anchor_152,
    &crypt32_export_anchor_153,
    &crypt32_export_anchor_154,
    &crypt32_export_anchor_155,
    &crypt32_export_anchor_156,
    &crypt32_export_anchor_157,
    &crypt32_export_anchor_158,
    &crypt32_export_anchor_159,
    &crypt32_export_anchor_160,
    &crypt32_export_anchor_161,
    &crypt32_export_anchor_162,
    &crypt32_export_anchor_163,
    &crypt32_export_anchor_164,
    &crypt32_export_anchor_165,
    &crypt32_export_anchor_166,
    &crypt32_export_anchor_167,
    &crypt32_export_anchor_168,
    &crypt32_export_anchor_169,
    &crypt32_export_anchor_170,
    &crypt32_export_anchor_171,
    &crypt32_export_anchor_172,
    &crypt32_export_anchor_173,
    &crypt32_export_anchor_174,
    &crypt32_export_anchor_175,
    &crypt32_export_anchor_176,
    &crypt32_export_anchor_177,
    &crypt32_export_anchor_178,
    &crypt32_export_anchor_179,
    &crypt32_export_anchor_180,
    &crypt32_export_anchor_181,
    &crypt32_export_anchor_182,
    &crypt32_export_anchor_183,
    &crypt32_export_anchor_184,
    &crypt32_export_anchor_185,
    &crypt32_export_anchor_186,
    &crypt32_export_anchor_187,
    &crypt32_export_anchor_188,
    &crypt32_export_anchor_189,
    &crypt32_export_anchor_190,
    &crypt32_export_anchor_191,
    &crypt32_export_anchor_192,
    &crypt32_export_anchor_193,
    &crypt32_export_anchor_194,
    &crypt32_export_anchor_195,
    &crypt32_export_anchor_196,
    &crypt32_export_anchor_197,
    &crypt32_export_anchor_198,
    &crypt32_export_anchor_199,
    &crypt32_export_anchor_200,
    &crypt32_export_anchor_201,
    &crypt32_export_anchor_202,
    &crypt32_export_anchor_203,
    &crypt32_export_anchor_204,
    &crypt32_export_anchor_205,
    &crypt32_export_anchor_206,
    &crypt32_export_anchor_207,
    &crypt32_export_anchor_208,
    &crypt32_export_anchor_209,
    0,
};
#elif defined(__x86_64__)
extern char crypt32_export_anchor_0 __asm__("CertAddCRLContextToStore");
extern char crypt32_export_anchor_1 __asm__("CertAddCTLContextToStore");
extern char crypt32_export_anchor_2 __asm__("CertAddCertificateContextToStore");
extern char crypt32_export_anchor_3 __asm__("CertAddCertificateLinkToStore");
extern char crypt32_export_anchor_4 __asm__("CertAddEncodedCRLToStore");
extern char crypt32_export_anchor_5 __asm__("CertAddEncodedCTLToStore");
extern char crypt32_export_anchor_6 __asm__("CertAddEncodedCertificateToStore");
extern char crypt32_export_anchor_7 __asm__("CertAddEncodedCertificateToSystemStoreA");
extern char crypt32_export_anchor_8 __asm__("CertAddEncodedCertificateToSystemStoreW");
extern char crypt32_export_anchor_9 __asm__("CertAddEnhancedKeyUsageIdentifier");
extern char crypt32_export_anchor_10 __asm__("CertAddSerializedElementToStore");
extern char crypt32_export_anchor_11 __asm__("CertAddStoreToCollection");
extern char crypt32_export_anchor_12 __asm__("CertAlgIdToOID");
extern char crypt32_export_anchor_13 __asm__("CertCloseStore");
extern char crypt32_export_anchor_14 __asm__("CertCompareCertificate");
extern char crypt32_export_anchor_15 __asm__("CertCompareCertificateName");
extern char crypt32_export_anchor_16 __asm__("CertCompareIntegerBlob");
extern char crypt32_export_anchor_17 __asm__("CertComparePublicKeyInfo");
extern char crypt32_export_anchor_18 __asm__("CertControlStore");
extern char crypt32_export_anchor_19 __asm__("CertCreateCRLContext");
extern char crypt32_export_anchor_20 __asm__("CertCreateCTLContext");
extern char crypt32_export_anchor_21 __asm__("CertCreateCertificateChainEngine");
extern char crypt32_export_anchor_22 __asm__("CertCreateCertificateContext");
extern char crypt32_export_anchor_23 __asm__("CertCreateContext");
extern char crypt32_export_anchor_24 __asm__("CertCreateSelfSignCertificate");
extern char crypt32_export_anchor_25 __asm__("CertDeleteCRLFromStore");
extern char crypt32_export_anchor_26 __asm__("CertDeleteCTLFromStore");
extern char crypt32_export_anchor_27 __asm__("CertDeleteCertificateFromStore");
extern char crypt32_export_anchor_28 __asm__("CertDuplicateCRLContext");
extern char crypt32_export_anchor_29 __asm__("CertDuplicateCTLContext");
extern char crypt32_export_anchor_30 __asm__("CertDuplicateCertificateChain");
extern char crypt32_export_anchor_31 __asm__("CertDuplicateCertificateContext");
extern char crypt32_export_anchor_32 __asm__("CertDuplicateStore");
extern char crypt32_export_anchor_33 __asm__("CertEnumCRLContextProperties");
extern char crypt32_export_anchor_34 __asm__("CertEnumCRLsInStore");
extern char crypt32_export_anchor_35 __asm__("CertEnumCTLContextProperties");
extern char crypt32_export_anchor_36 __asm__("CertEnumCTLsInStore");
extern char crypt32_export_anchor_37 __asm__("CertEnumCertificateContextProperties");
extern char crypt32_export_anchor_38 __asm__("CertEnumCertificatesInStore");
extern char crypt32_export_anchor_39 __asm__("CertEnumPhysicalStore");
extern char crypt32_export_anchor_40 __asm__("CertEnumSystemStore");
extern char crypt32_export_anchor_41 __asm__("CertEnumSystemStoreLocation");
extern char crypt32_export_anchor_42 __asm__("CertFindAttribute");
extern char crypt32_export_anchor_43 __asm__("CertFindCRLInStore");
extern char crypt32_export_anchor_44 __asm__("CertFindCTLInStore");
extern char crypt32_export_anchor_45 __asm__("CertFindCertificateInCRL");
extern char crypt32_export_anchor_46 __asm__("CertFindCertificateInStore");
extern char crypt32_export_anchor_47 __asm__("CertFindChainInStore");
extern char crypt32_export_anchor_48 __asm__("CertFindExtension");
extern char crypt32_export_anchor_49 __asm__("CertFindRDNAttr");
extern char crypt32_export_anchor_50 __asm__("CertFreeCRLContext");
extern char crypt32_export_anchor_51 __asm__("CertFreeCTLContext");
extern char crypt32_export_anchor_52 __asm__("CertFreeCertificateChain");
extern char crypt32_export_anchor_53 __asm__("CertFreeCertificateChainEngine");
extern char crypt32_export_anchor_54 __asm__("CertFreeCertificateContext");
extern char crypt32_export_anchor_55 __asm__("CertGetCRLContextProperty");
extern char crypt32_export_anchor_56 __asm__("CertGetCRLFromStore");
extern char crypt32_export_anchor_57 __asm__("CertGetCTLContextProperty");
extern char crypt32_export_anchor_58 __asm__("CertGetCertificateChain");
extern char crypt32_export_anchor_59 __asm__("CertGetCertificateContextProperty");
extern char crypt32_export_anchor_60 __asm__("CertGetEnhancedKeyUsage");
extern char crypt32_export_anchor_61 __asm__("CertGetIntendedKeyUsage");
extern char crypt32_export_anchor_62 __asm__("CertGetIssuerCertificateFromStore");
extern char crypt32_export_anchor_63 __asm__("CertGetNameStringA");
extern char crypt32_export_anchor_64 __asm__("CertGetNameStringW");
extern char crypt32_export_anchor_65 __asm__("CertGetPublicKeyLength");
extern char crypt32_export_anchor_66 __asm__("CertGetStoreProperty");
extern char crypt32_export_anchor_67 __asm__("CertGetSubjectCertificateFromStore");
extern char crypt32_export_anchor_68 __asm__("CertGetValidUsages");
extern char crypt32_export_anchor_69 __asm__("CertIsRDNAttrsInCertificateName");
extern char crypt32_export_anchor_70 __asm__("CertIsValidCRLForCertificate");
extern char crypt32_export_anchor_71 __asm__("CertNameToStrA");
extern char crypt32_export_anchor_72 __asm__("CertNameToStrW");
extern char crypt32_export_anchor_73 __asm__("CertOIDToAlgId");
extern char crypt32_export_anchor_74 __asm__("CertOpenStore");
extern char crypt32_export_anchor_75 __asm__("CertOpenSystemStoreA");
extern char crypt32_export_anchor_76 __asm__("CertOpenSystemStoreW");
extern char crypt32_export_anchor_77 __asm__("CertRDNValueToStrA");
extern char crypt32_export_anchor_78 __asm__("CertRDNValueToStrW");
extern char crypt32_export_anchor_79 __asm__("CertRegisterPhysicalStore");
extern char crypt32_export_anchor_80 __asm__("CertRegisterSystemStore");
extern char crypt32_export_anchor_81 __asm__("CertRemoveEnhancedKeyUsageIdentifier");
extern char crypt32_export_anchor_82 __asm__("CertRemoveStoreFromCollection");
extern char crypt32_export_anchor_83 __asm__("CertSaveStore");
extern char crypt32_export_anchor_84 __asm__("CertSerializeCRLStoreElement");
extern char crypt32_export_anchor_85 __asm__("CertSerializeCTLStoreElement");
extern char crypt32_export_anchor_86 __asm__("CertSerializeCertificateStoreElement");
extern char crypt32_export_anchor_87 __asm__("CertSetCRLContextProperty");
extern char crypt32_export_anchor_88 __asm__("CertSetCTLContextProperty");
extern char crypt32_export_anchor_89 __asm__("CertSetCertificateContextProperty");
extern char crypt32_export_anchor_90 __asm__("CertSetEnhancedKeyUsage");
extern char crypt32_export_anchor_91 __asm__("CertSetStoreProperty");
extern char crypt32_export_anchor_92 __asm__("CertStrToNameA");
extern char crypt32_export_anchor_93 __asm__("CertStrToNameW");
extern char crypt32_export_anchor_94 __asm__("CertUnregisterPhysicalStore");
extern char crypt32_export_anchor_95 __asm__("CertUnregisterSystemStore");
extern char crypt32_export_anchor_96 __asm__("CertVerifyCRLRevocation");
extern char crypt32_export_anchor_97 __asm__("CertVerifyCRLTimeValidity");
extern char crypt32_export_anchor_98 __asm__("CertVerifyCTLUsage");
extern char crypt32_export_anchor_99 __asm__("CertVerifyCertificateChainPolicy");
extern char crypt32_export_anchor_100 __asm__("CertVerifyRevocation");
extern char crypt32_export_anchor_101 __asm__("CertVerifySubjectCertificateContext");
extern char crypt32_export_anchor_102 __asm__("CertVerifyTimeValidity");
extern char crypt32_export_anchor_103 __asm__("CertVerifyValidityNesting");
extern char crypt32_export_anchor_104 __asm__("CryptAcquireCertificatePrivateKey");
extern char crypt32_export_anchor_105 __asm__("CryptBinaryToStringA");
extern char crypt32_export_anchor_106 __asm__("CryptBinaryToStringW");
extern char crypt32_export_anchor_107 __asm__("CryptDecodeObject");
extern char crypt32_export_anchor_108 __asm__("CryptDecodeObjectEx");
extern char crypt32_export_anchor_109 __asm__("CryptEncodeObject");
extern char crypt32_export_anchor_110 __asm__("CryptEncodeObjectEx");
extern char crypt32_export_anchor_111 __asm__("CryptEncryptMessage");
extern char crypt32_export_anchor_112 __asm__("CryptEnumOIDInfo");
extern char crypt32_export_anchor_113 __asm__("CryptExportPublicKeyInfo");
extern char crypt32_export_anchor_114 __asm__("CryptExportPublicKeyInfoEx");
extern char crypt32_export_anchor_115 __asm__("CryptFindCertificateKeyProvInfo");
extern char crypt32_export_anchor_116 __asm__("CryptFindLocalizedName");
extern char crypt32_export_anchor_117 __asm__("CryptFindOIDInfo");
extern char crypt32_export_anchor_118 __asm__("CryptFormatObject");
extern char crypt32_export_anchor_119 __asm__("CryptFreeOIDFunctionAddress");
extern char crypt32_export_anchor_120 __asm__("CryptGetDefaultOIDDllList");
extern char crypt32_export_anchor_121 __asm__("CryptGetDefaultOIDFunctionAddress");
extern char crypt32_export_anchor_122 __asm__("CryptGetMessageCertificates");
extern char crypt32_export_anchor_123 __asm__("CryptGetMessageSignerCount");
extern char crypt32_export_anchor_124 __asm__("CryptGetOIDFunctionAddress");
extern char crypt32_export_anchor_125 __asm__("CryptGetOIDFunctionValue");
extern char crypt32_export_anchor_126 __asm__("CryptHashCertificate");
extern char crypt32_export_anchor_127 __asm__("CryptHashCertificate2");
extern char crypt32_export_anchor_128 __asm__("CryptHashMessage");
extern char crypt32_export_anchor_129 __asm__("CryptHashPublicKeyInfo");
extern char crypt32_export_anchor_130 __asm__("CryptHashToBeSigned");
extern char crypt32_export_anchor_131 __asm__("CryptImportPublicKeyInfo");
extern char crypt32_export_anchor_132 __asm__("CryptImportPublicKeyInfoEx");
extern char crypt32_export_anchor_133 __asm__("CryptImportPublicKeyInfoEx2");
extern char crypt32_export_anchor_134 __asm__("CryptInitOIDFunctionSet");
extern char crypt32_export_anchor_135 __asm__("CryptInstallOIDFunctionAddress");
extern char crypt32_export_anchor_136 __asm__("CryptMemAlloc");
extern char crypt32_export_anchor_137 __asm__("CryptMemFree");
extern char crypt32_export_anchor_138 __asm__("CryptMemRealloc");
extern char crypt32_export_anchor_139 __asm__("CryptMsgClose");
extern char crypt32_export_anchor_140 __asm__("CryptMsgControl");
extern char crypt32_export_anchor_141 __asm__("CryptMsgDuplicate");
extern char crypt32_export_anchor_142 __asm__("CryptMsgEncodeAndSignCTL");
extern char crypt32_export_anchor_143 __asm__("CryptMsgGetAndVerifySigner");
extern char crypt32_export_anchor_144 __asm__("CryptMsgGetParam");
extern char crypt32_export_anchor_145 __asm__("CryptMsgOpenToDecode");
extern char crypt32_export_anchor_146 __asm__("CryptMsgOpenToEncode");
extern char crypt32_export_anchor_147 __asm__("CryptMsgSignCTL");
extern char crypt32_export_anchor_148 __asm__("CryptMsgUpdate");
extern char crypt32_export_anchor_149 __asm__("CryptMsgVerifyCountersignatureEncoded");
extern char crypt32_export_anchor_150 __asm__("CryptMsgVerifyCountersignatureEncodedEx");
extern char crypt32_export_anchor_151 __asm__("CryptProtectData");
extern char crypt32_export_anchor_152 __asm__("CryptProtectMemory");
extern char crypt32_export_anchor_153 __asm__("CryptQueryObject");
extern char crypt32_export_anchor_154 __asm__("CryptRegisterDefaultOIDFunction");
extern char crypt32_export_anchor_155 __asm__("CryptRegisterOIDFunction");
extern char crypt32_export_anchor_156 __asm__("CryptRegisterOIDInfo");
extern char crypt32_export_anchor_157 __asm__("CryptSIPAddProvider");
extern char crypt32_export_anchor_158 __asm__("CryptSIPCreateIndirectData");
extern char crypt32_export_anchor_159 __asm__("CryptSIPGetSignedDataMsg");
extern char crypt32_export_anchor_160 __asm__("CryptSIPLoad");
extern char crypt32_export_anchor_161 __asm__("CryptSIPPutSignedDataMsg");
extern char crypt32_export_anchor_162 __asm__("CryptSIPRemoveProvider");
extern char crypt32_export_anchor_163 __asm__("CryptSIPRemoveSignedDataMsg");
extern char crypt32_export_anchor_164 __asm__("CryptSIPRetrieveSubjectGuid");
extern char crypt32_export_anchor_165 __asm__("CryptSIPRetrieveSubjectGuidForCatalogFile");
extern char crypt32_export_anchor_166 __asm__("CryptSIPVerifyIndirectData");
extern char crypt32_export_anchor_167 __asm__("CryptSetKeyIdentifierProperty");
extern char crypt32_export_anchor_168 __asm__("CryptSetOIDFunctionValue");
extern char crypt32_export_anchor_169 __asm__("CryptSignAndEncodeCertificate");
extern char crypt32_export_anchor_170 __asm__("CryptSignCertificate");
extern char crypt32_export_anchor_171 __asm__("CryptSignMessage");
extern char crypt32_export_anchor_172 __asm__("CryptStringToBinaryA");
extern char crypt32_export_anchor_173 __asm__("CryptStringToBinaryW");
extern char crypt32_export_anchor_174 __asm__("CryptUnprotectData");
extern char crypt32_export_anchor_175 __asm__("CryptUnprotectMemory");
extern char crypt32_export_anchor_176 __asm__("CryptUnregisterDefaultOIDFunction");
extern char crypt32_export_anchor_177 __asm__("CryptUnregisterOIDFunction");
extern char crypt32_export_anchor_178 __asm__("CryptUnregisterOIDInfo");
extern char crypt32_export_anchor_179 __asm__("CryptVerifyCertificateSignature");
extern char crypt32_export_anchor_180 __asm__("CryptVerifyCertificateSignatureEx");
extern char crypt32_export_anchor_181 __asm__("CryptVerifyDetachedMessageHash");
extern char crypt32_export_anchor_182 __asm__("CryptVerifyDetachedMessageSignature");
extern char crypt32_export_anchor_183 __asm__("CryptVerifyMessageHash");
extern char crypt32_export_anchor_184 __asm__("CryptVerifyMessageSignature");
extern char crypt32_export_anchor_185 __asm__("I_CertUpdateStore");
extern char crypt32_export_anchor_186 __asm__("I_CryptAllocTls");
extern char crypt32_export_anchor_187 __asm__("I_CryptCreateLruCache");
extern char crypt32_export_anchor_188 __asm__("I_CryptCreateLruEntry");
extern char crypt32_export_anchor_189 __asm__("I_CryptDetachTls");
extern char crypt32_export_anchor_190 __asm__("I_CryptFindLruEntry");
extern char crypt32_export_anchor_191 __asm__("I_CryptFindLruEntryData");
extern char crypt32_export_anchor_192 __asm__("I_CryptFlushLruCache");
extern char crypt32_export_anchor_193 __asm__("I_CryptFreeLruCache");
extern char crypt32_export_anchor_194 __asm__("I_CryptFreeTls");
extern char crypt32_export_anchor_195 __asm__("I_CryptGetAsn1Decoder");
extern char crypt32_export_anchor_196 __asm__("I_CryptGetAsn1Encoder");
extern char crypt32_export_anchor_197 __asm__("I_CryptGetDefaultCryptProv");
extern char crypt32_export_anchor_198 __asm__("I_CryptGetOssGlobal");
extern char crypt32_export_anchor_199 __asm__("I_CryptGetTls");
extern char crypt32_export_anchor_200 __asm__("I_CryptInstallAsn1Module");
extern char crypt32_export_anchor_201 __asm__("I_CryptInstallOssGlobal");
extern char crypt32_export_anchor_202 __asm__("I_CryptReadTrustedPublisherDWORDValueFromRegistry");
extern char crypt32_export_anchor_203 __asm__("I_CryptSetTls");
extern char crypt32_export_anchor_204 __asm__("I_CryptUninstallAsn1Module");
extern char crypt32_export_anchor_205 __asm__("PFXExportCertStore");
extern char crypt32_export_anchor_206 __asm__("PFXExportCertStoreEx");
extern char crypt32_export_anchor_207 __asm__("PFXImportCertStore");
extern char crypt32_export_anchor_208 __asm__("PFXIsPFXBlob");
extern char crypt32_export_anchor_209 __asm__("PFXVerifyPassword");
__attribute__((used)) static const void *const crypt32_export_anchors[] = {
    &crypt32_export_anchor_0,
    &crypt32_export_anchor_1,
    &crypt32_export_anchor_2,
    &crypt32_export_anchor_3,
    &crypt32_export_anchor_4,
    &crypt32_export_anchor_5,
    &crypt32_export_anchor_6,
    &crypt32_export_anchor_7,
    &crypt32_export_anchor_8,
    &crypt32_export_anchor_9,
    &crypt32_export_anchor_10,
    &crypt32_export_anchor_11,
    &crypt32_export_anchor_12,
    &crypt32_export_anchor_13,
    &crypt32_export_anchor_14,
    &crypt32_export_anchor_15,
    &crypt32_export_anchor_16,
    &crypt32_export_anchor_17,
    &crypt32_export_anchor_18,
    &crypt32_export_anchor_19,
    &crypt32_export_anchor_20,
    &crypt32_export_anchor_21,
    &crypt32_export_anchor_22,
    &crypt32_export_anchor_23,
    &crypt32_export_anchor_24,
    &crypt32_export_anchor_25,
    &crypt32_export_anchor_26,
    &crypt32_export_anchor_27,
    &crypt32_export_anchor_28,
    &crypt32_export_anchor_29,
    &crypt32_export_anchor_30,
    &crypt32_export_anchor_31,
    &crypt32_export_anchor_32,
    &crypt32_export_anchor_33,
    &crypt32_export_anchor_34,
    &crypt32_export_anchor_35,
    &crypt32_export_anchor_36,
    &crypt32_export_anchor_37,
    &crypt32_export_anchor_38,
    &crypt32_export_anchor_39,
    &crypt32_export_anchor_40,
    &crypt32_export_anchor_41,
    &crypt32_export_anchor_42,
    &crypt32_export_anchor_43,
    &crypt32_export_anchor_44,
    &crypt32_export_anchor_45,
    &crypt32_export_anchor_46,
    &crypt32_export_anchor_47,
    &crypt32_export_anchor_48,
    &crypt32_export_anchor_49,
    &crypt32_export_anchor_50,
    &crypt32_export_anchor_51,
    &crypt32_export_anchor_52,
    &crypt32_export_anchor_53,
    &crypt32_export_anchor_54,
    &crypt32_export_anchor_55,
    &crypt32_export_anchor_56,
    &crypt32_export_anchor_57,
    &crypt32_export_anchor_58,
    &crypt32_export_anchor_59,
    &crypt32_export_anchor_60,
    &crypt32_export_anchor_61,
    &crypt32_export_anchor_62,
    &crypt32_export_anchor_63,
    &crypt32_export_anchor_64,
    &crypt32_export_anchor_65,
    &crypt32_export_anchor_66,
    &crypt32_export_anchor_67,
    &crypt32_export_anchor_68,
    &crypt32_export_anchor_69,
    &crypt32_export_anchor_70,
    &crypt32_export_anchor_71,
    &crypt32_export_anchor_72,
    &crypt32_export_anchor_73,
    &crypt32_export_anchor_74,
    &crypt32_export_anchor_75,
    &crypt32_export_anchor_76,
    &crypt32_export_anchor_77,
    &crypt32_export_anchor_78,
    &crypt32_export_anchor_79,
    &crypt32_export_anchor_80,
    &crypt32_export_anchor_81,
    &crypt32_export_anchor_82,
    &crypt32_export_anchor_83,
    &crypt32_export_anchor_84,
    &crypt32_export_anchor_85,
    &crypt32_export_anchor_86,
    &crypt32_export_anchor_87,
    &crypt32_export_anchor_88,
    &crypt32_export_anchor_89,
    &crypt32_export_anchor_90,
    &crypt32_export_anchor_91,
    &crypt32_export_anchor_92,
    &crypt32_export_anchor_93,
    &crypt32_export_anchor_94,
    &crypt32_export_anchor_95,
    &crypt32_export_anchor_96,
    &crypt32_export_anchor_97,
    &crypt32_export_anchor_98,
    &crypt32_export_anchor_99,
    &crypt32_export_anchor_100,
    &crypt32_export_anchor_101,
    &crypt32_export_anchor_102,
    &crypt32_export_anchor_103,
    &crypt32_export_anchor_104,
    &crypt32_export_anchor_105,
    &crypt32_export_anchor_106,
    &crypt32_export_anchor_107,
    &crypt32_export_anchor_108,
    &crypt32_export_anchor_109,
    &crypt32_export_anchor_110,
    &crypt32_export_anchor_111,
    &crypt32_export_anchor_112,
    &crypt32_export_anchor_113,
    &crypt32_export_anchor_114,
    &crypt32_export_anchor_115,
    &crypt32_export_anchor_116,
    &crypt32_export_anchor_117,
    &crypt32_export_anchor_118,
    &crypt32_export_anchor_119,
    &crypt32_export_anchor_120,
    &crypt32_export_anchor_121,
    &crypt32_export_anchor_122,
    &crypt32_export_anchor_123,
    &crypt32_export_anchor_124,
    &crypt32_export_anchor_125,
    &crypt32_export_anchor_126,
    &crypt32_export_anchor_127,
    &crypt32_export_anchor_128,
    &crypt32_export_anchor_129,
    &crypt32_export_anchor_130,
    &crypt32_export_anchor_131,
    &crypt32_export_anchor_132,
    &crypt32_export_anchor_133,
    &crypt32_export_anchor_134,
    &crypt32_export_anchor_135,
    &crypt32_export_anchor_136,
    &crypt32_export_anchor_137,
    &crypt32_export_anchor_138,
    &crypt32_export_anchor_139,
    &crypt32_export_anchor_140,
    &crypt32_export_anchor_141,
    &crypt32_export_anchor_142,
    &crypt32_export_anchor_143,
    &crypt32_export_anchor_144,
    &crypt32_export_anchor_145,
    &crypt32_export_anchor_146,
    &crypt32_export_anchor_147,
    &crypt32_export_anchor_148,
    &crypt32_export_anchor_149,
    &crypt32_export_anchor_150,
    &crypt32_export_anchor_151,
    &crypt32_export_anchor_152,
    &crypt32_export_anchor_153,
    &crypt32_export_anchor_154,
    &crypt32_export_anchor_155,
    &crypt32_export_anchor_156,
    &crypt32_export_anchor_157,
    &crypt32_export_anchor_158,
    &crypt32_export_anchor_159,
    &crypt32_export_anchor_160,
    &crypt32_export_anchor_161,
    &crypt32_export_anchor_162,
    &crypt32_export_anchor_163,
    &crypt32_export_anchor_164,
    &crypt32_export_anchor_165,
    &crypt32_export_anchor_166,
    &crypt32_export_anchor_167,
    &crypt32_export_anchor_168,
    &crypt32_export_anchor_169,
    &crypt32_export_anchor_170,
    &crypt32_export_anchor_171,
    &crypt32_export_anchor_172,
    &crypt32_export_anchor_173,
    &crypt32_export_anchor_174,
    &crypt32_export_anchor_175,
    &crypt32_export_anchor_176,
    &crypt32_export_anchor_177,
    &crypt32_export_anchor_178,
    &crypt32_export_anchor_179,
    &crypt32_export_anchor_180,
    &crypt32_export_anchor_181,
    &crypt32_export_anchor_182,
    &crypt32_export_anchor_183,
    &crypt32_export_anchor_184,
    &crypt32_export_anchor_185,
    &crypt32_export_anchor_186,
    &crypt32_export_anchor_187,
    &crypt32_export_anchor_188,
    &crypt32_export_anchor_189,
    &crypt32_export_anchor_190,
    &crypt32_export_anchor_191,
    &crypt32_export_anchor_192,
    &crypt32_export_anchor_193,
    &crypt32_export_anchor_194,
    &crypt32_export_anchor_195,
    &crypt32_export_anchor_196,
    &crypt32_export_anchor_197,
    &crypt32_export_anchor_198,
    &crypt32_export_anchor_199,
    &crypt32_export_anchor_200,
    &crypt32_export_anchor_201,
    &crypt32_export_anchor_202,
    &crypt32_export_anchor_203,
    &crypt32_export_anchor_204,
    &crypt32_export_anchor_205,
    &crypt32_export_anchor_206,
    &crypt32_export_anchor_207,
    &crypt32_export_anchor_208,
    &crypt32_export_anchor_209,
    0,
};
#elif defined(__aarch64__)
extern char crypt32_export_anchor_0 __asm__("CertAddCRLContextToStore");
extern char crypt32_export_anchor_1 __asm__("CertAddCTLContextToStore");
extern char crypt32_export_anchor_2 __asm__("CertAddCertificateContextToStore");
extern char crypt32_export_anchor_3 __asm__("CertAddCertificateLinkToStore");
extern char crypt32_export_anchor_4 __asm__("CertAddEncodedCRLToStore");
extern char crypt32_export_anchor_5 __asm__("CertAddEncodedCTLToStore");
extern char crypt32_export_anchor_6 __asm__("CertAddEncodedCertificateToStore");
extern char crypt32_export_anchor_7 __asm__("CertAddEncodedCertificateToSystemStoreA");
extern char crypt32_export_anchor_8 __asm__("CertAddEncodedCertificateToSystemStoreW");
extern char crypt32_export_anchor_9 __asm__("CertAddEnhancedKeyUsageIdentifier");
extern char crypt32_export_anchor_10 __asm__("CertAddSerializedElementToStore");
extern char crypt32_export_anchor_11 __asm__("CertAddStoreToCollection");
extern char crypt32_export_anchor_12 __asm__("CertAlgIdToOID");
extern char crypt32_export_anchor_13 __asm__("CertCloseStore");
extern char crypt32_export_anchor_14 __asm__("CertCompareCertificate");
extern char crypt32_export_anchor_15 __asm__("CertCompareCertificateName");
extern char crypt32_export_anchor_16 __asm__("CertCompareIntegerBlob");
extern char crypt32_export_anchor_17 __asm__("CertComparePublicKeyInfo");
extern char crypt32_export_anchor_18 __asm__("CertControlStore");
extern char crypt32_export_anchor_19 __asm__("CertCreateCRLContext");
extern char crypt32_export_anchor_20 __asm__("CertCreateCTLContext");
extern char crypt32_export_anchor_21 __asm__("CertCreateCertificateChainEngine");
extern char crypt32_export_anchor_22 __asm__("CertCreateCertificateContext");
extern char crypt32_export_anchor_23 __asm__("CertCreateContext");
extern char crypt32_export_anchor_24 __asm__("CertCreateSelfSignCertificate");
extern char crypt32_export_anchor_25 __asm__("CertDeleteCRLFromStore");
extern char crypt32_export_anchor_26 __asm__("CertDeleteCTLFromStore");
extern char crypt32_export_anchor_27 __asm__("CertDeleteCertificateFromStore");
extern char crypt32_export_anchor_28 __asm__("CertDuplicateCRLContext");
extern char crypt32_export_anchor_29 __asm__("CertDuplicateCTLContext");
extern char crypt32_export_anchor_30 __asm__("CertDuplicateCertificateChain");
extern char crypt32_export_anchor_31 __asm__("CertDuplicateCertificateContext");
extern char crypt32_export_anchor_32 __asm__("CertDuplicateStore");
extern char crypt32_export_anchor_33 __asm__("CertEnumCRLContextProperties");
extern char crypt32_export_anchor_34 __asm__("CertEnumCRLsInStore");
extern char crypt32_export_anchor_35 __asm__("CertEnumCTLContextProperties");
extern char crypt32_export_anchor_36 __asm__("CertEnumCTLsInStore");
extern char crypt32_export_anchor_37 __asm__("CertEnumCertificateContextProperties");
extern char crypt32_export_anchor_38 __asm__("CertEnumCertificatesInStore");
extern char crypt32_export_anchor_39 __asm__("CertEnumPhysicalStore");
extern char crypt32_export_anchor_40 __asm__("CertEnumSystemStore");
extern char crypt32_export_anchor_41 __asm__("CertEnumSystemStoreLocation");
extern char crypt32_export_anchor_42 __asm__("CertFindAttribute");
extern char crypt32_export_anchor_43 __asm__("CertFindCRLInStore");
extern char crypt32_export_anchor_44 __asm__("CertFindCTLInStore");
extern char crypt32_export_anchor_45 __asm__("CertFindCertificateInCRL");
extern char crypt32_export_anchor_46 __asm__("CertFindCertificateInStore");
extern char crypt32_export_anchor_47 __asm__("CertFindChainInStore");
extern char crypt32_export_anchor_48 __asm__("CertFindExtension");
extern char crypt32_export_anchor_49 __asm__("CertFindRDNAttr");
extern char crypt32_export_anchor_50 __asm__("CertFreeCRLContext");
extern char crypt32_export_anchor_51 __asm__("CertFreeCTLContext");
extern char crypt32_export_anchor_52 __asm__("CertFreeCertificateChain");
extern char crypt32_export_anchor_53 __asm__("CertFreeCertificateChainEngine");
extern char crypt32_export_anchor_54 __asm__("CertFreeCertificateContext");
extern char crypt32_export_anchor_55 __asm__("CertGetCRLContextProperty");
extern char crypt32_export_anchor_56 __asm__("CertGetCRLFromStore");
extern char crypt32_export_anchor_57 __asm__("CertGetCTLContextProperty");
extern char crypt32_export_anchor_58 __asm__("CertGetCertificateChain");
extern char crypt32_export_anchor_59 __asm__("CertGetCertificateContextProperty");
extern char crypt32_export_anchor_60 __asm__("CertGetEnhancedKeyUsage");
extern char crypt32_export_anchor_61 __asm__("CertGetIntendedKeyUsage");
extern char crypt32_export_anchor_62 __asm__("CertGetIssuerCertificateFromStore");
extern char crypt32_export_anchor_63 __asm__("CertGetNameStringA");
extern char crypt32_export_anchor_64 __asm__("CertGetNameStringW");
extern char crypt32_export_anchor_65 __asm__("CertGetPublicKeyLength");
extern char crypt32_export_anchor_66 __asm__("CertGetStoreProperty");
extern char crypt32_export_anchor_67 __asm__("CertGetSubjectCertificateFromStore");
extern char crypt32_export_anchor_68 __asm__("CertGetValidUsages");
extern char crypt32_export_anchor_69 __asm__("CertIsRDNAttrsInCertificateName");
extern char crypt32_export_anchor_70 __asm__("CertIsValidCRLForCertificate");
extern char crypt32_export_anchor_71 __asm__("CertNameToStrA");
extern char crypt32_export_anchor_72 __asm__("CertNameToStrW");
extern char crypt32_export_anchor_73 __asm__("CertOIDToAlgId");
extern char crypt32_export_anchor_74 __asm__("CertOpenStore");
extern char crypt32_export_anchor_75 __asm__("CertOpenSystemStoreA");
extern char crypt32_export_anchor_76 __asm__("CertOpenSystemStoreW");
extern char crypt32_export_anchor_77 __asm__("CertRDNValueToStrA");
extern char crypt32_export_anchor_78 __asm__("CertRDNValueToStrW");
extern char crypt32_export_anchor_79 __asm__("CertRegisterPhysicalStore");
extern char crypt32_export_anchor_80 __asm__("CertRegisterSystemStore");
extern char crypt32_export_anchor_81 __asm__("CertRemoveEnhancedKeyUsageIdentifier");
extern char crypt32_export_anchor_82 __asm__("CertRemoveStoreFromCollection");
extern char crypt32_export_anchor_83 __asm__("CertSaveStore");
extern char crypt32_export_anchor_84 __asm__("CertSerializeCRLStoreElement");
extern char crypt32_export_anchor_85 __asm__("CertSerializeCTLStoreElement");
extern char crypt32_export_anchor_86 __asm__("CertSerializeCertificateStoreElement");
extern char crypt32_export_anchor_87 __asm__("CertSetCRLContextProperty");
extern char crypt32_export_anchor_88 __asm__("CertSetCTLContextProperty");
extern char crypt32_export_anchor_89 __asm__("CertSetCertificateContextProperty");
extern char crypt32_export_anchor_90 __asm__("CertSetEnhancedKeyUsage");
extern char crypt32_export_anchor_91 __asm__("CertSetStoreProperty");
extern char crypt32_export_anchor_92 __asm__("CertStrToNameA");
extern char crypt32_export_anchor_93 __asm__("CertStrToNameW");
extern char crypt32_export_anchor_94 __asm__("CertUnregisterPhysicalStore");
extern char crypt32_export_anchor_95 __asm__("CertUnregisterSystemStore");
extern char crypt32_export_anchor_96 __asm__("CertVerifyCRLRevocation");
extern char crypt32_export_anchor_97 __asm__("CertVerifyCRLTimeValidity");
extern char crypt32_export_anchor_98 __asm__("CertVerifyCTLUsage");
extern char crypt32_export_anchor_99 __asm__("CertVerifyCertificateChainPolicy");
extern char crypt32_export_anchor_100 __asm__("CertVerifyRevocation");
extern char crypt32_export_anchor_101 __asm__("CertVerifySubjectCertificateContext");
extern char crypt32_export_anchor_102 __asm__("CertVerifyTimeValidity");
extern char crypt32_export_anchor_103 __asm__("CertVerifyValidityNesting");
extern char crypt32_export_anchor_104 __asm__("CryptAcquireCertificatePrivateKey");
extern char crypt32_export_anchor_105 __asm__("CryptBinaryToStringA");
extern char crypt32_export_anchor_106 __asm__("CryptBinaryToStringW");
extern char crypt32_export_anchor_107 __asm__("CryptDecodeObject");
extern char crypt32_export_anchor_108 __asm__("CryptDecodeObjectEx");
extern char crypt32_export_anchor_109 __asm__("CryptEncodeObject");
extern char crypt32_export_anchor_110 __asm__("CryptEncodeObjectEx");
extern char crypt32_export_anchor_111 __asm__("CryptEncryptMessage");
extern char crypt32_export_anchor_112 __asm__("CryptEnumOIDInfo");
extern char crypt32_export_anchor_113 __asm__("CryptExportPublicKeyInfo");
extern char crypt32_export_anchor_114 __asm__("CryptExportPublicKeyInfoEx");
extern char crypt32_export_anchor_115 __asm__("CryptFindCertificateKeyProvInfo");
extern char crypt32_export_anchor_116 __asm__("CryptFindLocalizedName");
extern char crypt32_export_anchor_117 __asm__("CryptFindOIDInfo");
extern char crypt32_export_anchor_118 __asm__("CryptFormatObject");
extern char crypt32_export_anchor_119 __asm__("CryptFreeOIDFunctionAddress");
extern char crypt32_export_anchor_120 __asm__("CryptGetDefaultOIDDllList");
extern char crypt32_export_anchor_121 __asm__("CryptGetDefaultOIDFunctionAddress");
extern char crypt32_export_anchor_122 __asm__("CryptGetMessageCertificates");
extern char crypt32_export_anchor_123 __asm__("CryptGetMessageSignerCount");
extern char crypt32_export_anchor_124 __asm__("CryptGetOIDFunctionAddress");
extern char crypt32_export_anchor_125 __asm__("CryptGetOIDFunctionValue");
extern char crypt32_export_anchor_126 __asm__("CryptHashCertificate");
extern char crypt32_export_anchor_127 __asm__("CryptHashCertificate2");
extern char crypt32_export_anchor_128 __asm__("CryptHashMessage");
extern char crypt32_export_anchor_129 __asm__("CryptHashPublicKeyInfo");
extern char crypt32_export_anchor_130 __asm__("CryptHashToBeSigned");
extern char crypt32_export_anchor_131 __asm__("CryptImportPublicKeyInfo");
extern char crypt32_export_anchor_132 __asm__("CryptImportPublicKeyInfoEx");
extern char crypt32_export_anchor_133 __asm__("CryptImportPublicKeyInfoEx2");
extern char crypt32_export_anchor_134 __asm__("CryptInitOIDFunctionSet");
extern char crypt32_export_anchor_135 __asm__("CryptInstallOIDFunctionAddress");
extern char crypt32_export_anchor_136 __asm__("CryptMemAlloc");
extern char crypt32_export_anchor_137 __asm__("CryptMemFree");
extern char crypt32_export_anchor_138 __asm__("CryptMemRealloc");
extern char crypt32_export_anchor_139 __asm__("CryptMsgClose");
extern char crypt32_export_anchor_140 __asm__("CryptMsgControl");
extern char crypt32_export_anchor_141 __asm__("CryptMsgDuplicate");
extern char crypt32_export_anchor_142 __asm__("CryptMsgEncodeAndSignCTL");
extern char crypt32_export_anchor_143 __asm__("CryptMsgGetAndVerifySigner");
extern char crypt32_export_anchor_144 __asm__("CryptMsgGetParam");
extern char crypt32_export_anchor_145 __asm__("CryptMsgOpenToDecode");
extern char crypt32_export_anchor_146 __asm__("CryptMsgOpenToEncode");
extern char crypt32_export_anchor_147 __asm__("CryptMsgSignCTL");
extern char crypt32_export_anchor_148 __asm__("CryptMsgUpdate");
extern char crypt32_export_anchor_149 __asm__("CryptMsgVerifyCountersignatureEncoded");
extern char crypt32_export_anchor_150 __asm__("CryptMsgVerifyCountersignatureEncodedEx");
extern char crypt32_export_anchor_151 __asm__("CryptProtectData");
extern char crypt32_export_anchor_152 __asm__("CryptProtectMemory");
extern char crypt32_export_anchor_153 __asm__("CryptQueryObject");
extern char crypt32_export_anchor_154 __asm__("CryptRegisterDefaultOIDFunction");
extern char crypt32_export_anchor_155 __asm__("CryptRegisterOIDFunction");
extern char crypt32_export_anchor_156 __asm__("CryptRegisterOIDInfo");
extern char crypt32_export_anchor_157 __asm__("CryptSIPAddProvider");
extern char crypt32_export_anchor_158 __asm__("CryptSIPCreateIndirectData");
extern char crypt32_export_anchor_159 __asm__("CryptSIPGetSignedDataMsg");
extern char crypt32_export_anchor_160 __asm__("CryptSIPLoad");
extern char crypt32_export_anchor_161 __asm__("CryptSIPPutSignedDataMsg");
extern char crypt32_export_anchor_162 __asm__("CryptSIPRemoveProvider");
extern char crypt32_export_anchor_163 __asm__("CryptSIPRemoveSignedDataMsg");
extern char crypt32_export_anchor_164 __asm__("CryptSIPRetrieveSubjectGuid");
extern char crypt32_export_anchor_165 __asm__("CryptSIPRetrieveSubjectGuidForCatalogFile");
extern char crypt32_export_anchor_166 __asm__("CryptSIPVerifyIndirectData");
extern char crypt32_export_anchor_167 __asm__("CryptSetKeyIdentifierProperty");
extern char crypt32_export_anchor_168 __asm__("CryptSetOIDFunctionValue");
extern char crypt32_export_anchor_169 __asm__("CryptSignAndEncodeCertificate");
extern char crypt32_export_anchor_170 __asm__("CryptSignCertificate");
extern char crypt32_export_anchor_171 __asm__("CryptSignMessage");
extern char crypt32_export_anchor_172 __asm__("CryptStringToBinaryA");
extern char crypt32_export_anchor_173 __asm__("CryptStringToBinaryW");
extern char crypt32_export_anchor_174 __asm__("CryptUnprotectData");
extern char crypt32_export_anchor_175 __asm__("CryptUnprotectMemory");
extern char crypt32_export_anchor_176 __asm__("CryptUnregisterDefaultOIDFunction");
extern char crypt32_export_anchor_177 __asm__("CryptUnregisterOIDFunction");
extern char crypt32_export_anchor_178 __asm__("CryptUnregisterOIDInfo");
extern char crypt32_export_anchor_179 __asm__("CryptVerifyCertificateSignature");
extern char crypt32_export_anchor_180 __asm__("CryptVerifyCertificateSignatureEx");
extern char crypt32_export_anchor_181 __asm__("CryptVerifyDetachedMessageHash");
extern char crypt32_export_anchor_182 __asm__("CryptVerifyDetachedMessageSignature");
extern char crypt32_export_anchor_183 __asm__("CryptVerifyMessageHash");
extern char crypt32_export_anchor_184 __asm__("CryptVerifyMessageSignature");
extern char crypt32_export_anchor_185 __asm__("I_CertUpdateStore");
extern char crypt32_export_anchor_186 __asm__("I_CryptAllocTls");
extern char crypt32_export_anchor_187 __asm__("I_CryptCreateLruCache");
extern char crypt32_export_anchor_188 __asm__("I_CryptCreateLruEntry");
extern char crypt32_export_anchor_189 __asm__("I_CryptDetachTls");
extern char crypt32_export_anchor_190 __asm__("I_CryptFindLruEntry");
extern char crypt32_export_anchor_191 __asm__("I_CryptFindLruEntryData");
extern char crypt32_export_anchor_192 __asm__("I_CryptFlushLruCache");
extern char crypt32_export_anchor_193 __asm__("I_CryptFreeLruCache");
extern char crypt32_export_anchor_194 __asm__("I_CryptFreeTls");
extern char crypt32_export_anchor_195 __asm__("I_CryptGetAsn1Decoder");
extern char crypt32_export_anchor_196 __asm__("I_CryptGetAsn1Encoder");
extern char crypt32_export_anchor_197 __asm__("I_CryptGetDefaultCryptProv");
extern char crypt32_export_anchor_198 __asm__("I_CryptGetOssGlobal");
extern char crypt32_export_anchor_199 __asm__("I_CryptGetTls");
extern char crypt32_export_anchor_200 __asm__("I_CryptInstallAsn1Module");
extern char crypt32_export_anchor_201 __asm__("I_CryptInstallOssGlobal");
extern char crypt32_export_anchor_202 __asm__("I_CryptReadTrustedPublisherDWORDValueFromRegistry");
extern char crypt32_export_anchor_203 __asm__("I_CryptSetTls");
extern char crypt32_export_anchor_204 __asm__("I_CryptUninstallAsn1Module");
extern char crypt32_export_anchor_205 __asm__("PFXExportCertStore");
extern char crypt32_export_anchor_206 __asm__("PFXExportCertStoreEx");
extern char crypt32_export_anchor_207 __asm__("PFXImportCertStore");
extern char crypt32_export_anchor_208 __asm__("PFXIsPFXBlob");
extern char crypt32_export_anchor_209 __asm__("PFXVerifyPassword");
__attribute__((used)) static const void *const crypt32_export_anchors[] = {
    &crypt32_export_anchor_0,
    &crypt32_export_anchor_1,
    &crypt32_export_anchor_2,
    &crypt32_export_anchor_3,
    &crypt32_export_anchor_4,
    &crypt32_export_anchor_5,
    &crypt32_export_anchor_6,
    &crypt32_export_anchor_7,
    &crypt32_export_anchor_8,
    &crypt32_export_anchor_9,
    &crypt32_export_anchor_10,
    &crypt32_export_anchor_11,
    &crypt32_export_anchor_12,
    &crypt32_export_anchor_13,
    &crypt32_export_anchor_14,
    &crypt32_export_anchor_15,
    &crypt32_export_anchor_16,
    &crypt32_export_anchor_17,
    &crypt32_export_anchor_18,
    &crypt32_export_anchor_19,
    &crypt32_export_anchor_20,
    &crypt32_export_anchor_21,
    &crypt32_export_anchor_22,
    &crypt32_export_anchor_23,
    &crypt32_export_anchor_24,
    &crypt32_export_anchor_25,
    &crypt32_export_anchor_26,
    &crypt32_export_anchor_27,
    &crypt32_export_anchor_28,
    &crypt32_export_anchor_29,
    &crypt32_export_anchor_30,
    &crypt32_export_anchor_31,
    &crypt32_export_anchor_32,
    &crypt32_export_anchor_33,
    &crypt32_export_anchor_34,
    &crypt32_export_anchor_35,
    &crypt32_export_anchor_36,
    &crypt32_export_anchor_37,
    &crypt32_export_anchor_38,
    &crypt32_export_anchor_39,
    &crypt32_export_anchor_40,
    &crypt32_export_anchor_41,
    &crypt32_export_anchor_42,
    &crypt32_export_anchor_43,
    &crypt32_export_anchor_44,
    &crypt32_export_anchor_45,
    &crypt32_export_anchor_46,
    &crypt32_export_anchor_47,
    &crypt32_export_anchor_48,
    &crypt32_export_anchor_49,
    &crypt32_export_anchor_50,
    &crypt32_export_anchor_51,
    &crypt32_export_anchor_52,
    &crypt32_export_anchor_53,
    &crypt32_export_anchor_54,
    &crypt32_export_anchor_55,
    &crypt32_export_anchor_56,
    &crypt32_export_anchor_57,
    &crypt32_export_anchor_58,
    &crypt32_export_anchor_59,
    &crypt32_export_anchor_60,
    &crypt32_export_anchor_61,
    &crypt32_export_anchor_62,
    &crypt32_export_anchor_63,
    &crypt32_export_anchor_64,
    &crypt32_export_anchor_65,
    &crypt32_export_anchor_66,
    &crypt32_export_anchor_67,
    &crypt32_export_anchor_68,
    &crypt32_export_anchor_69,
    &crypt32_export_anchor_70,
    &crypt32_export_anchor_71,
    &crypt32_export_anchor_72,
    &crypt32_export_anchor_73,
    &crypt32_export_anchor_74,
    &crypt32_export_anchor_75,
    &crypt32_export_anchor_76,
    &crypt32_export_anchor_77,
    &crypt32_export_anchor_78,
    &crypt32_export_anchor_79,
    &crypt32_export_anchor_80,
    &crypt32_export_anchor_81,
    &crypt32_export_anchor_82,
    &crypt32_export_anchor_83,
    &crypt32_export_anchor_84,
    &crypt32_export_anchor_85,
    &crypt32_export_anchor_86,
    &crypt32_export_anchor_87,
    &crypt32_export_anchor_88,
    &crypt32_export_anchor_89,
    &crypt32_export_anchor_90,
    &crypt32_export_anchor_91,
    &crypt32_export_anchor_92,
    &crypt32_export_anchor_93,
    &crypt32_export_anchor_94,
    &crypt32_export_anchor_95,
    &crypt32_export_anchor_96,
    &crypt32_export_anchor_97,
    &crypt32_export_anchor_98,
    &crypt32_export_anchor_99,
    &crypt32_export_anchor_100,
    &crypt32_export_anchor_101,
    &crypt32_export_anchor_102,
    &crypt32_export_anchor_103,
    &crypt32_export_anchor_104,
    &crypt32_export_anchor_105,
    &crypt32_export_anchor_106,
    &crypt32_export_anchor_107,
    &crypt32_export_anchor_108,
    &crypt32_export_anchor_109,
    &crypt32_export_anchor_110,
    &crypt32_export_anchor_111,
    &crypt32_export_anchor_112,
    &crypt32_export_anchor_113,
    &crypt32_export_anchor_114,
    &crypt32_export_anchor_115,
    &crypt32_export_anchor_116,
    &crypt32_export_anchor_117,
    &crypt32_export_anchor_118,
    &crypt32_export_anchor_119,
    &crypt32_export_anchor_120,
    &crypt32_export_anchor_121,
    &crypt32_export_anchor_122,
    &crypt32_export_anchor_123,
    &crypt32_export_anchor_124,
    &crypt32_export_anchor_125,
    &crypt32_export_anchor_126,
    &crypt32_export_anchor_127,
    &crypt32_export_anchor_128,
    &crypt32_export_anchor_129,
    &crypt32_export_anchor_130,
    &crypt32_export_anchor_131,
    &crypt32_export_anchor_132,
    &crypt32_export_anchor_133,
    &crypt32_export_anchor_134,
    &crypt32_export_anchor_135,
    &crypt32_export_anchor_136,
    &crypt32_export_anchor_137,
    &crypt32_export_anchor_138,
    &crypt32_export_anchor_139,
    &crypt32_export_anchor_140,
    &crypt32_export_anchor_141,
    &crypt32_export_anchor_142,
    &crypt32_export_anchor_143,
    &crypt32_export_anchor_144,
    &crypt32_export_anchor_145,
    &crypt32_export_anchor_146,
    &crypt32_export_anchor_147,
    &crypt32_export_anchor_148,
    &crypt32_export_anchor_149,
    &crypt32_export_anchor_150,
    &crypt32_export_anchor_151,
    &crypt32_export_anchor_152,
    &crypt32_export_anchor_153,
    &crypt32_export_anchor_154,
    &crypt32_export_anchor_155,
    &crypt32_export_anchor_156,
    &crypt32_export_anchor_157,
    &crypt32_export_anchor_158,
    &crypt32_export_anchor_159,
    &crypt32_export_anchor_160,
    &crypt32_export_anchor_161,
    &crypt32_export_anchor_162,
    &crypt32_export_anchor_163,
    &crypt32_export_anchor_164,
    &crypt32_export_anchor_165,
    &crypt32_export_anchor_166,
    &crypt32_export_anchor_167,
    &crypt32_export_anchor_168,
    &crypt32_export_anchor_169,
    &crypt32_export_anchor_170,
    &crypt32_export_anchor_171,
    &crypt32_export_anchor_172,
    &crypt32_export_anchor_173,
    &crypt32_export_anchor_174,
    &crypt32_export_anchor_175,
    &crypt32_export_anchor_176,
    &crypt32_export_anchor_177,
    &crypt32_export_anchor_178,
    &crypt32_export_anchor_179,
    &crypt32_export_anchor_180,
    &crypt32_export_anchor_181,
    &crypt32_export_anchor_182,
    &crypt32_export_anchor_183,
    &crypt32_export_anchor_184,
    &crypt32_export_anchor_185,
    &crypt32_export_anchor_186,
    &crypt32_export_anchor_187,
    &crypt32_export_anchor_188,
    &crypt32_export_anchor_189,
    &crypt32_export_anchor_190,
    &crypt32_export_anchor_191,
    &crypt32_export_anchor_192,
    &crypt32_export_anchor_193,
    &crypt32_export_anchor_194,
    &crypt32_export_anchor_195,
    &crypt32_export_anchor_196,
    &crypt32_export_anchor_197,
    &crypt32_export_anchor_198,
    &crypt32_export_anchor_199,
    &crypt32_export_anchor_200,
    &crypt32_export_anchor_201,
    &crypt32_export_anchor_202,
    &crypt32_export_anchor_203,
    &crypt32_export_anchor_204,
    &crypt32_export_anchor_205,
    &crypt32_export_anchor_206,
    &crypt32_export_anchor_207,
    &crypt32_export_anchor_208,
    &crypt32_export_anchor_209,
    0,
};
#endif
