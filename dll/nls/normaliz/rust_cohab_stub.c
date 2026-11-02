/* normaliz is Rust (sdk/lib/rust/normaliz-rust); a CMake target needs one TU. */
typedef int normaliz_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char normaliz_export_anchor_0 __asm__("_IdnToAscii@20");
extern char normaliz_export_anchor_1 __asm__("_IdnToNameprepUnicode@20");
extern char normaliz_export_anchor_2 __asm__("_IdnToUnicode@20");
extern char normaliz_export_anchor_3 __asm__("_IsNormalizedString@12");
extern char normaliz_export_anchor_4 __asm__("_NormalizeString@20");
__attribute__((used)) static const void *const normaliz_export_anchors[] = {
    &normaliz_export_anchor_0,
    &normaliz_export_anchor_1,
    &normaliz_export_anchor_2,
    &normaliz_export_anchor_3,
    &normaliz_export_anchor_4,
    0,
};
#elif defined(__x86_64__)
extern char normaliz_export_anchor_0 __asm__("IdnToAscii");
extern char normaliz_export_anchor_1 __asm__("IdnToNameprepUnicode");
extern char normaliz_export_anchor_2 __asm__("IdnToUnicode");
extern char normaliz_export_anchor_3 __asm__("IsNormalizedString");
extern char normaliz_export_anchor_4 __asm__("NormalizeString");
__attribute__((used)) static const void *const normaliz_export_anchors[] = {
    &normaliz_export_anchor_0,
    &normaliz_export_anchor_1,
    &normaliz_export_anchor_2,
    &normaliz_export_anchor_3,
    &normaliz_export_anchor_4,
    0,
};
#elif defined(__aarch64__)
extern char normaliz_export_anchor_0 __asm__("IdnToAscii");
extern char normaliz_export_anchor_1 __asm__("IdnToNameprepUnicode");
extern char normaliz_export_anchor_2 __asm__("IdnToUnicode");
extern char normaliz_export_anchor_3 __asm__("IsNormalizedString");
extern char normaliz_export_anchor_4 __asm__("NormalizeString");
__attribute__((used)) static const void *const normaliz_export_anchors[] = {
    &normaliz_export_anchor_0,
    &normaliz_export_anchor_1,
    &normaliz_export_anchor_2,
    &normaliz_export_anchor_3,
    &normaliz_export_anchor_4,
    0,
};
#endif
