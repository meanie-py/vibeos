/* hlink is Rust (sdk/lib/rust/hlink-rust); a CMake target needs one TU. */
typedef int hlink_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char hlink_export_anchor_0 __asm__("_DllCanUnloadNow@0");
extern char hlink_export_anchor_1 __asm__("_DllGetClassObject@12");
extern char hlink_export_anchor_2 __asm__("_DllRegisterServer@0");
extern char hlink_export_anchor_3 __asm__("_DllUnregisterServer@0");
extern char hlink_export_anchor_4 __asm__("_HlinkClone@20");
extern char hlink_export_anchor_5 __asm__("_HlinkCreateBrowseContext@12");
extern char hlink_export_anchor_6 __asm__("_HlinkCreateExtensionServices@28");
extern char hlink_export_anchor_7 __asm__("_HlinkCreateFromData@24");
extern char hlink_export_anchor_8 __asm__("_HlinkCreateFromMoniker@32");
extern char hlink_export_anchor_9 __asm__("_HlinkCreateFromString@32");
extern char hlink_export_anchor_10 __asm__("_HlinkGetSpecialReference@8");
extern char hlink_export_anchor_11 __asm__("_HlinkIsShortcut@4");
extern char hlink_export_anchor_12 __asm__("_HlinkNavigate@24");
extern char hlink_export_anchor_13 __asm__("_HlinkNavigateToStringReference@36");
extern char hlink_export_anchor_14 __asm__("_HlinkOnNavigate@28");
extern char hlink_export_anchor_15 __asm__("_HlinkParseDisplayName@20");
extern char hlink_export_anchor_16 __asm__("_HlinkQueryCreateFromData@4");
extern char hlink_export_anchor_17 __asm__("_HlinkResolveMonikerForData@28");
extern char hlink_export_anchor_18 __asm__("_HlinkTranslateURL@12");
extern char hlink_export_anchor_19 __asm__("_HlinkUpdateStackItem@24");
__attribute__((used)) static const void *const hlink_export_anchors[] = {
    &hlink_export_anchor_0,
    &hlink_export_anchor_1,
    &hlink_export_anchor_2,
    &hlink_export_anchor_3,
    &hlink_export_anchor_4,
    &hlink_export_anchor_5,
    &hlink_export_anchor_6,
    &hlink_export_anchor_7,
    &hlink_export_anchor_8,
    &hlink_export_anchor_9,
    &hlink_export_anchor_10,
    &hlink_export_anchor_11,
    &hlink_export_anchor_12,
    &hlink_export_anchor_13,
    &hlink_export_anchor_14,
    &hlink_export_anchor_15,
    &hlink_export_anchor_16,
    &hlink_export_anchor_17,
    &hlink_export_anchor_18,
    &hlink_export_anchor_19,
    0,
};
#elif defined(__x86_64__)
extern char hlink_export_anchor_0 __asm__("DllCanUnloadNow");
extern char hlink_export_anchor_1 __asm__("DllGetClassObject");
extern char hlink_export_anchor_2 __asm__("DllRegisterServer");
extern char hlink_export_anchor_3 __asm__("DllUnregisterServer");
extern char hlink_export_anchor_4 __asm__("HlinkClone");
extern char hlink_export_anchor_5 __asm__("HlinkCreateBrowseContext");
extern char hlink_export_anchor_6 __asm__("HlinkCreateExtensionServices");
extern char hlink_export_anchor_7 __asm__("HlinkCreateFromData");
extern char hlink_export_anchor_8 __asm__("HlinkCreateFromMoniker");
extern char hlink_export_anchor_9 __asm__("HlinkCreateFromString");
extern char hlink_export_anchor_10 __asm__("HlinkGetSpecialReference");
extern char hlink_export_anchor_11 __asm__("HlinkIsShortcut");
extern char hlink_export_anchor_12 __asm__("HlinkNavigate");
extern char hlink_export_anchor_13 __asm__("HlinkNavigateToStringReference");
extern char hlink_export_anchor_14 __asm__("HlinkOnNavigate");
extern char hlink_export_anchor_15 __asm__("HlinkParseDisplayName");
extern char hlink_export_anchor_16 __asm__("HlinkQueryCreateFromData");
extern char hlink_export_anchor_17 __asm__("HlinkResolveMonikerForData");
extern char hlink_export_anchor_18 __asm__("HlinkTranslateURL");
extern char hlink_export_anchor_19 __asm__("HlinkUpdateStackItem");
__attribute__((used)) static const void *const hlink_export_anchors[] = {
    &hlink_export_anchor_0,
    &hlink_export_anchor_1,
    &hlink_export_anchor_2,
    &hlink_export_anchor_3,
    &hlink_export_anchor_4,
    &hlink_export_anchor_5,
    &hlink_export_anchor_6,
    &hlink_export_anchor_7,
    &hlink_export_anchor_8,
    &hlink_export_anchor_9,
    &hlink_export_anchor_10,
    &hlink_export_anchor_11,
    &hlink_export_anchor_12,
    &hlink_export_anchor_13,
    &hlink_export_anchor_14,
    &hlink_export_anchor_15,
    &hlink_export_anchor_16,
    &hlink_export_anchor_17,
    &hlink_export_anchor_18,
    &hlink_export_anchor_19,
    0,
};
#elif defined(__aarch64__)
extern char hlink_export_anchor_0 __asm__("DllCanUnloadNow");
extern char hlink_export_anchor_1 __asm__("DllGetClassObject");
extern char hlink_export_anchor_2 __asm__("DllRegisterServer");
extern char hlink_export_anchor_3 __asm__("DllUnregisterServer");
extern char hlink_export_anchor_4 __asm__("HlinkClone");
extern char hlink_export_anchor_5 __asm__("HlinkCreateBrowseContext");
extern char hlink_export_anchor_6 __asm__("HlinkCreateExtensionServices");
extern char hlink_export_anchor_7 __asm__("HlinkCreateFromData");
extern char hlink_export_anchor_8 __asm__("HlinkCreateFromMoniker");
extern char hlink_export_anchor_9 __asm__("HlinkCreateFromString");
extern char hlink_export_anchor_10 __asm__("HlinkGetSpecialReference");
extern char hlink_export_anchor_11 __asm__("HlinkIsShortcut");
extern char hlink_export_anchor_12 __asm__("HlinkNavigate");
extern char hlink_export_anchor_13 __asm__("HlinkNavigateToStringReference");
extern char hlink_export_anchor_14 __asm__("HlinkOnNavigate");
extern char hlink_export_anchor_15 __asm__("HlinkParseDisplayName");
extern char hlink_export_anchor_16 __asm__("HlinkQueryCreateFromData");
extern char hlink_export_anchor_17 __asm__("HlinkResolveMonikerForData");
extern char hlink_export_anchor_18 __asm__("HlinkTranslateURL");
extern char hlink_export_anchor_19 __asm__("HlinkUpdateStackItem");
__attribute__((used)) static const void *const hlink_export_anchors[] = {
    &hlink_export_anchor_0,
    &hlink_export_anchor_1,
    &hlink_export_anchor_2,
    &hlink_export_anchor_3,
    &hlink_export_anchor_4,
    &hlink_export_anchor_5,
    &hlink_export_anchor_6,
    &hlink_export_anchor_7,
    &hlink_export_anchor_8,
    &hlink_export_anchor_9,
    &hlink_export_anchor_10,
    &hlink_export_anchor_11,
    &hlink_export_anchor_12,
    &hlink_export_anchor_13,
    &hlink_export_anchor_14,
    &hlink_export_anchor_15,
    &hlink_export_anchor_16,
    &hlink_export_anchor_17,
    &hlink_export_anchor_18,
    &hlink_export_anchor_19,
    0,
};
#endif
