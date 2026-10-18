/* msctfime is Rust (sdk/lib/rust/msctfime-rust); a CMake target needs one TU. */
typedef int msctfime_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char msctfime_export_anchor_0 __asm__("_CtfImeCreateInputContext@4");
extern char msctfime_export_anchor_1 __asm__("_CtfImeCreateThreadMgr@0");
extern char msctfime_export_anchor_2 __asm__("_CtfImeDestroyInputContext@4");
extern char msctfime_export_anchor_3 __asm__("_CtfImeDestroyThreadMgr@0");
extern char msctfime_export_anchor_4 __asm__("_CtfImeDispatchDefImeMessage@16");
extern char msctfime_export_anchor_5 __asm__("_CtfImeEscapeEx@16");
extern char msctfime_export_anchor_6 __asm__("_CtfImeGetGuidAtom@12");
extern char msctfime_export_anchor_7 __asm__("_CtfImeInquireExW@16");
extern char msctfime_export_anchor_8 __asm__("_CtfImeIsGuidMapEnable@4");
extern char msctfime_export_anchor_9 __asm__("_CtfImeIsIME@4");
extern char msctfime_export_anchor_10 __asm__("_CtfImeProcessCicHotkey@12");
extern char msctfime_export_anchor_11 __asm__("_CtfImeSelectEx@12");
extern char msctfime_export_anchor_12 __asm__("_CtfImeSetActiveContextAlways@16");
extern char msctfime_export_anchor_13 __asm__("_CtfImeThreadDetach@0");
extern char msctfime_export_anchor_14 __asm__("_ImeConfigure@16");
extern char msctfime_export_anchor_15 __asm__("_ImeConversionList@20");
extern char msctfime_export_anchor_16 __asm__("_ImeDestroy@4");
extern char msctfime_export_anchor_17 __asm__("_ImeEnumRegisterWord@20");
extern char msctfime_export_anchor_18 __asm__("_ImeEscape@12");
extern char msctfime_export_anchor_19 __asm__("_ImeGetRegisterWordStyle@8");
extern char msctfime_export_anchor_20 __asm__("_ImeInquire@12");
extern char msctfime_export_anchor_21 __asm__("_ImeProcessKey@16");
extern char msctfime_export_anchor_22 __asm__("_ImeRegisterWord@12");
extern char msctfime_export_anchor_23 __asm__("_ImeSelect@8");
extern char msctfime_export_anchor_24 __asm__("_ImeSetActiveContext@8");
extern char msctfime_export_anchor_25 __asm__("_ImeSetCompositionString@24");
extern char msctfime_export_anchor_26 __asm__("_ImeToAsciiEx@24");
extern char msctfime_export_anchor_27 __asm__("_ImeUnregisterWord@12");
extern char msctfime_export_anchor_28 __asm__("_NotifyIME@16");
extern char msctfime_export_anchor_29 __asm__("_UIWndProc@16");
__attribute__((used)) static const void *const msctfime_export_anchors[] = {
    &msctfime_export_anchor_0,
    &msctfime_export_anchor_1,
    &msctfime_export_anchor_2,
    &msctfime_export_anchor_3,
    &msctfime_export_anchor_4,
    &msctfime_export_anchor_5,
    &msctfime_export_anchor_6,
    &msctfime_export_anchor_7,
    &msctfime_export_anchor_8,
    &msctfime_export_anchor_9,
    &msctfime_export_anchor_10,
    &msctfime_export_anchor_11,
    &msctfime_export_anchor_12,
    &msctfime_export_anchor_13,
    &msctfime_export_anchor_14,
    &msctfime_export_anchor_15,
    &msctfime_export_anchor_16,
    &msctfime_export_anchor_17,
    &msctfime_export_anchor_18,
    &msctfime_export_anchor_19,
    &msctfime_export_anchor_20,
    &msctfime_export_anchor_21,
    &msctfime_export_anchor_22,
    &msctfime_export_anchor_23,
    &msctfime_export_anchor_24,
    &msctfime_export_anchor_25,
    &msctfime_export_anchor_26,
    &msctfime_export_anchor_27,
    &msctfime_export_anchor_28,
    &msctfime_export_anchor_29,
    0,
};
#elif defined(__x86_64__)
extern char msctfime_export_anchor_0 __asm__("CtfImeCreateInputContext");
extern char msctfime_export_anchor_1 __asm__("CtfImeCreateThreadMgr");
extern char msctfime_export_anchor_2 __asm__("CtfImeDestroyInputContext");
extern char msctfime_export_anchor_3 __asm__("CtfImeDestroyThreadMgr");
extern char msctfime_export_anchor_4 __asm__("CtfImeDispatchDefImeMessage");
extern char msctfime_export_anchor_5 __asm__("CtfImeEscapeEx");
extern char msctfime_export_anchor_6 __asm__("CtfImeGetGuidAtom");
extern char msctfime_export_anchor_7 __asm__("CtfImeInquireExW");
extern char msctfime_export_anchor_8 __asm__("CtfImeIsGuidMapEnable");
extern char msctfime_export_anchor_9 __asm__("CtfImeIsIME");
extern char msctfime_export_anchor_10 __asm__("CtfImeProcessCicHotkey");
extern char msctfime_export_anchor_11 __asm__("CtfImeSelectEx");
extern char msctfime_export_anchor_12 __asm__("CtfImeSetActiveContextAlways");
extern char msctfime_export_anchor_13 __asm__("CtfImeThreadDetach");
extern char msctfime_export_anchor_14 __asm__("ImeConfigure");
extern char msctfime_export_anchor_15 __asm__("ImeConversionList");
extern char msctfime_export_anchor_16 __asm__("ImeDestroy");
extern char msctfime_export_anchor_17 __asm__("ImeEnumRegisterWord");
extern char msctfime_export_anchor_18 __asm__("ImeEscape");
extern char msctfime_export_anchor_19 __asm__("ImeGetRegisterWordStyle");
extern char msctfime_export_anchor_20 __asm__("ImeInquire");
extern char msctfime_export_anchor_21 __asm__("ImeProcessKey");
extern char msctfime_export_anchor_22 __asm__("ImeRegisterWord");
extern char msctfime_export_anchor_23 __asm__("ImeSelect");
extern char msctfime_export_anchor_24 __asm__("ImeSetActiveContext");
extern char msctfime_export_anchor_25 __asm__("ImeSetCompositionString");
extern char msctfime_export_anchor_26 __asm__("ImeToAsciiEx");
extern char msctfime_export_anchor_27 __asm__("ImeUnregisterWord");
extern char msctfime_export_anchor_28 __asm__("NotifyIME");
extern char msctfime_export_anchor_29 __asm__("UIWndProc");
__attribute__((used)) static const void *const msctfime_export_anchors[] = {
    &msctfime_export_anchor_0,
    &msctfime_export_anchor_1,
    &msctfime_export_anchor_2,
    &msctfime_export_anchor_3,
    &msctfime_export_anchor_4,
    &msctfime_export_anchor_5,
    &msctfime_export_anchor_6,
    &msctfime_export_anchor_7,
    &msctfime_export_anchor_8,
    &msctfime_export_anchor_9,
    &msctfime_export_anchor_10,
    &msctfime_export_anchor_11,
    &msctfime_export_anchor_12,
    &msctfime_export_anchor_13,
    &msctfime_export_anchor_14,
    &msctfime_export_anchor_15,
    &msctfime_export_anchor_16,
    &msctfime_export_anchor_17,
    &msctfime_export_anchor_18,
    &msctfime_export_anchor_19,
    &msctfime_export_anchor_20,
    &msctfime_export_anchor_21,
    &msctfime_export_anchor_22,
    &msctfime_export_anchor_23,
    &msctfime_export_anchor_24,
    &msctfime_export_anchor_25,
    &msctfime_export_anchor_26,
    &msctfime_export_anchor_27,
    &msctfime_export_anchor_28,
    &msctfime_export_anchor_29,
    0,
};
#elif defined(__aarch64__)
extern char msctfime_export_anchor_0 __asm__("CtfImeCreateInputContext");
extern char msctfime_export_anchor_1 __asm__("CtfImeCreateThreadMgr");
extern char msctfime_export_anchor_2 __asm__("CtfImeDestroyInputContext");
extern char msctfime_export_anchor_3 __asm__("CtfImeDestroyThreadMgr");
extern char msctfime_export_anchor_4 __asm__("CtfImeDispatchDefImeMessage");
extern char msctfime_export_anchor_5 __asm__("CtfImeEscapeEx");
extern char msctfime_export_anchor_6 __asm__("CtfImeGetGuidAtom");
extern char msctfime_export_anchor_7 __asm__("CtfImeInquireExW");
extern char msctfime_export_anchor_8 __asm__("CtfImeIsGuidMapEnable");
extern char msctfime_export_anchor_9 __asm__("CtfImeIsIME");
extern char msctfime_export_anchor_10 __asm__("CtfImeProcessCicHotkey");
extern char msctfime_export_anchor_11 __asm__("CtfImeSelectEx");
extern char msctfime_export_anchor_12 __asm__("CtfImeSetActiveContextAlways");
extern char msctfime_export_anchor_13 __asm__("CtfImeThreadDetach");
extern char msctfime_export_anchor_14 __asm__("ImeConfigure");
extern char msctfime_export_anchor_15 __asm__("ImeConversionList");
extern char msctfime_export_anchor_16 __asm__("ImeDestroy");
extern char msctfime_export_anchor_17 __asm__("ImeEnumRegisterWord");
extern char msctfime_export_anchor_18 __asm__("ImeEscape");
extern char msctfime_export_anchor_19 __asm__("ImeGetRegisterWordStyle");
extern char msctfime_export_anchor_20 __asm__("ImeInquire");
extern char msctfime_export_anchor_21 __asm__("ImeProcessKey");
extern char msctfime_export_anchor_22 __asm__("ImeRegisterWord");
extern char msctfime_export_anchor_23 __asm__("ImeSelect");
extern char msctfime_export_anchor_24 __asm__("ImeSetActiveContext");
extern char msctfime_export_anchor_25 __asm__("ImeSetCompositionString");
extern char msctfime_export_anchor_26 __asm__("ImeToAsciiEx");
extern char msctfime_export_anchor_27 __asm__("ImeUnregisterWord");
extern char msctfime_export_anchor_28 __asm__("NotifyIME");
extern char msctfime_export_anchor_29 __asm__("UIWndProc");
__attribute__((used)) static const void *const msctfime_export_anchors[] = {
    &msctfime_export_anchor_0,
    &msctfime_export_anchor_1,
    &msctfime_export_anchor_2,
    &msctfime_export_anchor_3,
    &msctfime_export_anchor_4,
    &msctfime_export_anchor_5,
    &msctfime_export_anchor_6,
    &msctfime_export_anchor_7,
    &msctfime_export_anchor_8,
    &msctfime_export_anchor_9,
    &msctfime_export_anchor_10,
    &msctfime_export_anchor_11,
    &msctfime_export_anchor_12,
    &msctfime_export_anchor_13,
    &msctfime_export_anchor_14,
    &msctfime_export_anchor_15,
    &msctfime_export_anchor_16,
    &msctfime_export_anchor_17,
    &msctfime_export_anchor_18,
    &msctfime_export_anchor_19,
    &msctfime_export_anchor_20,
    &msctfime_export_anchor_21,
    &msctfime_export_anchor_22,
    &msctfime_export_anchor_23,
    &msctfime_export_anchor_24,
    &msctfime_export_anchor_25,
    &msctfime_export_anchor_26,
    &msctfime_export_anchor_27,
    &msctfime_export_anchor_28,
    &msctfime_export_anchor_29,
    0,
};
#endif
