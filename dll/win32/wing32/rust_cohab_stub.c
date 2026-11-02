/* wing32 is Rust (sdk/lib/rust/wing32-rust); a CMake target needs one TU. */
typedef int wing32_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char wing32_export_anchor_0 __asm__("_WinGBitBlt@32");
extern char wing32_export_anchor_1 __asm__("_WinGCreateBitmap@12");
extern char wing32_export_anchor_2 __asm__("_WinGCreateDC@0");
extern char wing32_export_anchor_3 __asm__("_WinGCreateHalftoneBrush@12");
extern char wing32_export_anchor_4 __asm__("_WinGCreateHalftonePalette@0");
extern char wing32_export_anchor_5 __asm__("_WinGGetDIBColorTable@16");
extern char wing32_export_anchor_6 __asm__("_WinGGetDIBPointer@8");
extern char wing32_export_anchor_7 __asm__("_WinGRecommendDIBFormat@4");
extern char wing32_export_anchor_8 __asm__("_WinGSetDIBColorTable@16");
extern char wing32_export_anchor_9 __asm__("_WinGStretchBlt@40");
__attribute__((used)) static const void *const wing32_export_anchors[] = {
    &wing32_export_anchor_0,
    &wing32_export_anchor_1,
    &wing32_export_anchor_2,
    &wing32_export_anchor_3,
    &wing32_export_anchor_4,
    &wing32_export_anchor_5,
    &wing32_export_anchor_6,
    &wing32_export_anchor_7,
    &wing32_export_anchor_8,
    &wing32_export_anchor_9,
    0,
};
#elif defined(__x86_64__)
extern char wing32_export_anchor_0 __asm__("WinGBitBlt");
extern char wing32_export_anchor_1 __asm__("WinGCreateBitmap");
extern char wing32_export_anchor_2 __asm__("WinGCreateDC");
extern char wing32_export_anchor_3 __asm__("WinGCreateHalftoneBrush");
extern char wing32_export_anchor_4 __asm__("WinGCreateHalftonePalette");
extern char wing32_export_anchor_5 __asm__("WinGGetDIBColorTable");
extern char wing32_export_anchor_6 __asm__("WinGGetDIBPointer");
extern char wing32_export_anchor_7 __asm__("WinGRecommendDIBFormat");
extern char wing32_export_anchor_8 __asm__("WinGSetDIBColorTable");
extern char wing32_export_anchor_9 __asm__("WinGStretchBlt");
__attribute__((used)) static const void *const wing32_export_anchors[] = {
    &wing32_export_anchor_0,
    &wing32_export_anchor_1,
    &wing32_export_anchor_2,
    &wing32_export_anchor_3,
    &wing32_export_anchor_4,
    &wing32_export_anchor_5,
    &wing32_export_anchor_6,
    &wing32_export_anchor_7,
    &wing32_export_anchor_8,
    &wing32_export_anchor_9,
    0,
};
#elif defined(__aarch64__)
extern char wing32_export_anchor_0 __asm__("WinGBitBlt");
extern char wing32_export_anchor_1 __asm__("WinGCreateBitmap");
extern char wing32_export_anchor_2 __asm__("WinGCreateDC");
extern char wing32_export_anchor_3 __asm__("WinGCreateHalftoneBrush");
extern char wing32_export_anchor_4 __asm__("WinGCreateHalftonePalette");
extern char wing32_export_anchor_5 __asm__("WinGGetDIBColorTable");
extern char wing32_export_anchor_6 __asm__("WinGGetDIBPointer");
extern char wing32_export_anchor_7 __asm__("WinGRecommendDIBFormat");
extern char wing32_export_anchor_8 __asm__("WinGSetDIBColorTable");
extern char wing32_export_anchor_9 __asm__("WinGStretchBlt");
__attribute__((used)) static const void *const wing32_export_anchors[] = {
    &wing32_export_anchor_0,
    &wing32_export_anchor_1,
    &wing32_export_anchor_2,
    &wing32_export_anchor_3,
    &wing32_export_anchor_4,
    &wing32_export_anchor_5,
    &wing32_export_anchor_6,
    &wing32_export_anchor_7,
    &wing32_export_anchor_8,
    &wing32_export_anchor_9,
    0,
};
#endif
