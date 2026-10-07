/*
 * The MUI language tables: localized UI text, DATA only. usetup's code is Rust
 * (sdk/lib/rust/usetup-rust, `rustconv port`), including mui.c's ResourceList index,
 * which names these tables as extern statics. They are the lang/<xx-YY>.h definitions
 * muilanguages.h pulls in, under the same LANGUAGE_* guards, and nothing else: including
 * muilanguages.h itself would define ResourceList a second time.
 *
 * Kept as C deliberately for now (~70,000 lines of string tables across 30 languages; a
 * c2rust rendering would be several times that as u16 arrays and adds nothing over the C).
 * Owed: move them to a resource the Rust reads, tracked in rust-port-status.md.
 */
#include "usetup.h"

#include <reactos/buildno.h>

// #ifdef LANGUAGE_AF_ZA
    // #include "lang/af-ZA.h"
// #endif
// #ifdef LANGUAGE_AR_AE
    // #include "lang/ar-AE.h"
// #endif
// #ifdef LANGUAGE_AR_BH
    // #include "lang/ar-BH.h"
// #endif
// #ifdef LANGUAGE_AR_DZ
    // #include "lang/ar-DZ.h"
// #endif
// #ifdef LANGUAGE_AR_EG
    // #include "lang/ar-EG.h"
// #endif
// #ifdef LANGUAGE_AR_IQ
    // #include "lang/ar-IQ.h"
// #endif
// #ifdef LANGUAGE_AR_JO
    // #include "lang/ar-JO.h"
// #endif
// #ifdef LANGUAGE_AR_KW
    // #include "lang/ar-KW.h"
// #endif
// #ifdef LANGUAGE_AR_LB
    // #include "lang/ar-LB.h"
// #endif
// #ifdef LANGUAGE_AR_LY
    // #include "lang/ar-LY.h"
// #endif
// #ifdef LANGUAGE_AR_MA
    // #include "lang/ar-MA.h"
// #endif
// #ifdef LANGUAGE_AR_OM
    // #include "lang/ar-OM.h"
// #endif
// #ifdef LANGUAGE_AR_QA
    // #include "lang/ar-QA.h"
// #endif
// #ifdef LANGUAGE_AR_SA
    // #include "lang/ar-SA.h"
// #endif
// #ifdef LANGUAGE_AR_SY
    // #include "lang/ar-SY.h"
// #endif
// #ifdef LANGUAGE_AR_TN
    // #include "lang/ar-TN.h"
// #endif
// #ifdef LANGUAGE_AR_YE
    // #include "lang/ar-YE.h"
// #endif
// #ifdef LANGUAGE_AZ_AZ
    // #include "lang/az-AZ.h"
// #endif
// #ifdef LANGUAGE_BE_BY
    // #include "lang/be-BY.h"
// #endif
#ifdef LANGUAGE_BG_BG
    #include "lang/bg-BG.h"
#endif
#ifdef LANGUAGE_BN_BD
    #include "lang/bn-BD.h"
#endif
// #ifdef LANGUAGE_BN_IN
    // #include "lang/bn-IN.h"
// #endif
// #ifdef LANGUAGE_CA_ES
    // #include "lang/ca-ES.h"
// #endif
#ifdef LANGUAGE_CS_CZ
    #include "lang/cs-CZ.h"
#endif
#ifdef LANGUAGE_DA_DK
    #include "lang/da-DK.h"
#endif
// #ifdef LANGUAGE_DE_AT
    // #include "lang/de-AT.h"
// #endif
// #ifdef LANGUAGE_DE_CH
    // #include "lang/de-CH.h"
// #endif
#ifdef LANGUAGE_DE_DE
    #include "lang/de-DE.h"
#endif
// #ifdef LANGUAGE_DE_LI
    // #include "lang/de-LI.h"
// #endif
// #ifdef LANGUAGE_DE_LU
    // #include "lang/de-LU.h"
// #endif
// #ifdef LANGUAGE_DV_MV
    // #include "lang/dv-MV.h"
// #endif
#ifdef LANGUAGE_EL_GR
    #include "lang/el-GR.h"
#endif
// #ifdef LANGUAGE_EN_AU
    // #include "lang/en-AU.h"
// #endif
// #ifdef LANGUAGE_EN_BZ
    // #include "lang/en-BZ.h"
// #endif
// #ifdef LANGUAGE_EN_CA
    // #include "lang/en-CA.h"
// #endif
// #ifdef LANGUAGE_EN_CB
    // #include "lang/en-CB.h"
// #endif
// #ifdef LANGUAGE_EN_GB
    // #include "lang/en-GB.h"
// #endif
// #ifdef LANGUAGE_EN_IE
    // #include "lang/en-IE.h"
// #endif
// #ifdef LANGUAGE_EN_JM
    // #include "lang/en-JM.h"
// #endif
// #ifdef LANGUAGE_EN_NZ
    // #include "lang/en-NZ.h"
// #endif
// #ifdef LANGUAGE_EN_PH
    // #include "lang/en-PH.h"
// #endif
// #ifdef LANGUAGE_EN_TT
    // #include "lang/en-TT.h"
// #endif
#ifdef LANGUAGE_EN_US
    #include "lang/en-US.h"
#endif
// #ifdef LANGUAGE_EN_ZA
    // #include "lang/en-ZA.h"
// #endif
// #ifdef LANGUAGE_EN_ZW
    // #include "lang/en-ZW.h"
// #endif
// #ifdef LANGUAGE_EO_AA
    // #include "lang/eo-AA.h"
// #endif
// #ifdef LANGUAGE_ES_AR
    // #include "lang/es-AR.h"
// #endif
// #ifdef LANGUAGE_ES_BO
    // #include "lang/es-BO.h"
// #endif
// #ifdef LANGUAGE_ES_CL
    // #include "lang/es-CL.h"
// #endif
// #ifdef LANGUAGE_ES_CO
    // #include "lang/es-CO.h"
// #endif
// #ifdef LANGUAGE_ES_CR
    // #include "lang/es-CR.h"
// #endif
// #ifdef LANGUAGE_ES_DO
    // #include "lang/es-DO.h"
// #endif
// #ifdef LANGUAGE_ES_EC
    // #include "lang/es-EC.h"
// #endif
#ifdef LANGUAGE_ES_ES
    #include "lang/es-ES.h"
#endif
// #ifdef LANGUAGE_ES_GT
    // #include "lang/es-GT.h"
// #endif
// #ifdef LANGUAGE_ES_HN
    // #include "lang/es-HN.h"
// #endif
// #ifdef LANGUAGE_ES_MX
    // #include "lang/es-MX.h"
// #endif
// #ifdef LANGUAGE_ES_NI
    // #include "lang/es-NI.h"
// #endif
// #ifdef LANGUAGE_ES_PA
    // #include "lang/es-PA.h"
// #endif
// #ifdef LANGUAGE_ES_PE
    // #include "lang/es-PE.h"
// #endif
// #ifdef LANGUAGE_ES_PR
    // #include "lang/es-PR.h"
// #endif
// #ifdef LANGUAGE_ES_PY
    // #include "lang/es-PY.h"
// #endif
// #ifdef LANGUAGE_ES_SV
    // #include "lang/es-SV.h"
// #endif
// #ifdef LANGUAGE_ES_UY
    // #include "lang/es-UY.h"
// #endif
// #ifdef LANGUAGE_ES_VE
    // #include "lang/es-VE.h"
// #endif
#ifdef LANGUAGE_ET_EE
    #include "lang/et-EE.h"
#endif
#ifdef LANGUAGE_EU_ES
    #include "lang/eu-ES.h"
#endif
// #ifdef LANGUAGE_FA_IR
    // #include "lang/fa-IR.h"
// #endif
// #ifdef LANGUAGE_FI_FI
    // #include "lang/fi-FI.h"
// #endif
// #ifdef LANGUAGE_FO_FO
    // #include "lang/fo-FO.h"
// #endif
// #ifdef LANGUAGE_FR_BE
    // #include "lang/fr-BE.h"
// #endif
// #ifdef LANGUAGE_FR_CA
    // #include "lang/fr-CA.h"
// #endif
// #ifdef LANGUAGE_FR_CH
    // #include "lang/fr-CH.h"
// #endif
#ifdef LANGUAGE_FR_FR
    #include "lang/fr-FR.h"
#endif
// #ifdef LANGUAGE_FR_LU
    // #include "lang/fr-LU.h"
// #endif
// #ifdef LANGUAGE_FR_MC
    // #include "lang/fr-MC.h"
// #endif
// #ifdef LANGUAGE_GL_ES
    // #include "lang/gl-ES.h"
// #endif
// #ifdef LANGUAGE_GU_IN
    // #include "lang/gu-IN.h"
// #endif
#ifdef LANGUAGE_HE_IL
    #include "lang/he-IL.h"
#endif
// #ifdef LANGUAGE_HI_IN
    // #include "lang/hi-IN.h"
// #endif
#ifdef LANGUAGE_HR_HR
    #include "lang/hr-HR.h"
#endif
#ifdef LANGUAGE_HU_HU
    #include "lang/hu-HU.h"
#endif
// #ifdef LANGUAGE_HY_AM
    // #include "lang/hy-AM.h"
// #endif
#ifdef LANGUAGE_ID_ID
    #include "lang/id-ID.h"
 #endif
// #ifdef LANGUAGE_IS_IS
    // #include "lang/is-IS.h"
// #endif
// #ifdef LANGUAGE_IT_CH
    // #include "lang/it-CH.h"
// #endif
#ifdef LANGUAGE_IT_IT
    #include "lang/it-IT.h"
#endif
#ifdef LANGUAGE_JA_JP
    #include "lang/ja-JP.h"
#endif
// #ifdef LANGUAGE_KA_GE
    // #include "lang/ka-GE.h"
// #endif
// #ifdef LANGUAGE_KK_KZ
    // #include "lang/kk-KZ.h"
// #endif
// #ifdef LANGUAGE_KN_IN
    // #include "lang/kn-IN.h"
// #endif
// #ifdef LANGUAGE_KOK_IN
    // #include "lang/kok-IN.h"
// #endif
// #ifdef LANGUAGE_KO_KR
    // #include "lang/ko-KR.h"
// #endif
// #ifdef LANGUAGE_KY_KG
    // #include "lang/ky-KG.h"
// #endif
#ifdef LANGUAGE_LT_LT
    #include "lang/lt-LT.h"
#endif
// #ifdef LANGUAGE_LV_LV
    // #include "lang/lv-LV.h"
// #endif
// #ifdef LANGUAGE_MK_MK
    // #include "lang/mk-MK.h"
// #endif
// #ifdef LANGUAGE_MN_MN
    // #include "lang/mn-MN.h"
// #endif
// #ifdef LANGUAGE_MR_IN
    // #include "lang/mr-IN.h"
// #endif
// #ifdef LANGUAGE_MS_BN
    // #include "lang/ms-BN.h"
// #endif
#ifdef LANGUAGE_MS_MY
    #include "lang/ms-MY.h"
#endif
// #ifdef LANGUAGE_MY_MM
    // #include "lang/my-MM.h"
// #endif
// #ifdef LANGUAGE_NL_BE
    // #include "lang/nl-BE.h"
// #endif
#ifdef LANGUAGE_NL_NL
    #include "lang/nl-NL.h"
#endif
// #ifdef LANGUAGE_NN_NO
    // #include "lang/nn-NO.h"
// #endif
// #ifdef LANGUAGE_PA_IN
    // #include "lang/pa-IN.h"
// #endif
#ifdef LANGUAGE_PL_PL
    #include "lang/pl-PL.h"
#endif
#ifdef LANGUAGE_PT_BR
    #include "lang/pt-BR.h"
#endif
#ifdef LANGUAGE_PT_PT
    #include "lang/pt-PT.h"
#endif
// #ifdef LANGUAGE_RM_CH
    // #include "lang/rm-CH.h"
// #endif
#ifdef LANGUAGE_RO_RO
    #include "lang/ro-RO.h"
#endif
#ifdef LANGUAGE_RU_RU
    #include "lang/ru-RU.h"
#endif
// #ifdef LANGUAGE_SA_IN
    // #include "lang/sa-IN.h"
// #endif
#ifdef LANGUAGE_SK_SK
    #include "lang/sk-SK.h"
#endif
// #ifdef LANGUAGE_SL_SI
    // #include "lang/sl-SI.h"
// #endif
#ifdef LANGUAGE_SQ_AL
    #include "lang/sq-AL.h"
#endif
// #ifdef LANGUAGE_SR_SP
    // #include "lang/sr-SP.h"
// #endif
// #ifdef LANGUAGE_SV_FI
    // #include "lang/sv-FI.h"
// #endif
#ifdef LANGUAGE_SV_SE
    #include "lang/sv-SE.h"
#endif
// #ifdef LANGUAGE_SW_KE
    // #include "lang/sw-KE.h"
// #endif
// #ifdef LANGUAGE_SYR_SY
    // #include "lang/syr-SY.h"
// #endif
// #ifdef LANGUAGE_TA_IN
    // #include "lang/ta-IN.h"
// #endif
// #ifdef LANGUAGE_TE_IN
    // #include "lang/te-IN.h"
// #endif
// #ifdef LANGUAGE_TH_TH
    // #include "lang/th-TH.h"
// #endif
#ifdef LANGUAGE_TR_TR
    #include "lang/tr-TR.h"
#endif
// #ifdef LANGUAGE_TT_TA
    // #include "lang/tt-TA.h"
// #endif
#ifdef LANGUAGE_UK_UA
    #include "lang/uk-UA.h"
#endif
// #ifdef LANGUAGE_UR_PK
    // #include "lang/ur-PK.h"
// #endif
// #ifdef LANGUAGE_UZ_UZ
    // #include "lang/uz-UZ.h"
// #endif
// #ifdef LANGUAGE_VI_VN
    // #include "lang/vi-VN.h"
// #endif
// #ifdef LANGUAGE_WA_BE
    // #include "lang/wa-BE.h"
// #endif
// #ifdef LANGUAGE_ZH_CN
    // #include "lang/zh-CN.h"
// #endif
// #ifdef LANGUAGE_ZH_HK
    // #include "lang/zh-HK.h"
// #endif
// #ifdef LANGUAGE_ZH_MO
    // #include "lang/zh-MO.h"
// #endif
// #ifdef LANGUAGE_ZH_SG
    // #include "lang/zh-SG.h"
// #endif
// #ifdef LANGUAGE_ZH_TW
    // #include "lang/zh-TW.h"
// #endif

