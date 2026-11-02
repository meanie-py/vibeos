/* dxapi is Rust (sdk/lib/rust/dxapi-rust); a CMake target needs one TU. */
typedef int dxapi_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char dxapi_export_anchor_0 __asm__("_DxApi@20");
extern char dxapi_export_anchor_1 __asm__("_DxApiGetVersion@0");
extern char dxapi_export_anchor_2 __asm__("_DxApiInitialize@32");
extern char dxapi_export_anchor_3 __asm__("_DxAutoflipUpdate@20");
extern char dxapi_export_anchor_4 __asm__("_DxEnableIRQ@8");
extern char dxapi_export_anchor_5 __asm__("_DxLoseObject@8");
extern char dxapi_export_anchor_6 __asm__("_DxUpdateCapture@12");
__attribute__((used)) static const void *const dxapi_export_anchors[] = {
    &dxapi_export_anchor_0,
    &dxapi_export_anchor_1,
    &dxapi_export_anchor_2,
    &dxapi_export_anchor_3,
    &dxapi_export_anchor_4,
    &dxapi_export_anchor_5,
    &dxapi_export_anchor_6,
    0,
};
#elif defined(__x86_64__)
extern char dxapi_export_anchor_0 __asm__("DxApi");
extern char dxapi_export_anchor_1 __asm__("DxApiGetVersion");
extern char dxapi_export_anchor_2 __asm__("DxApiInitialize");
extern char dxapi_export_anchor_3 __asm__("DxAutoflipUpdate");
extern char dxapi_export_anchor_4 __asm__("DxEnableIRQ");
extern char dxapi_export_anchor_5 __asm__("DxLoseObject");
extern char dxapi_export_anchor_6 __asm__("DxUpdateCapture");
__attribute__((used)) static const void *const dxapi_export_anchors[] = {
    &dxapi_export_anchor_0,
    &dxapi_export_anchor_1,
    &dxapi_export_anchor_2,
    &dxapi_export_anchor_3,
    &dxapi_export_anchor_4,
    &dxapi_export_anchor_5,
    &dxapi_export_anchor_6,
    0,
};
#elif defined(__aarch64__)
extern char dxapi_export_anchor_0 __asm__("DxApi");
extern char dxapi_export_anchor_1 __asm__("DxApiGetVersion");
extern char dxapi_export_anchor_2 __asm__("DxApiInitialize");
extern char dxapi_export_anchor_3 __asm__("DxAutoflipUpdate");
extern char dxapi_export_anchor_4 __asm__("DxEnableIRQ");
extern char dxapi_export_anchor_5 __asm__("DxLoseObject");
extern char dxapi_export_anchor_6 __asm__("DxUpdateCapture");
__attribute__((used)) static const void *const dxapi_export_anchors[] = {
    &dxapi_export_anchor_0,
    &dxapi_export_anchor_1,
    &dxapi_export_anchor_2,
    &dxapi_export_anchor_3,
    &dxapi_export_anchor_4,
    &dxapi_export_anchor_5,
    &dxapi_export_anchor_6,
    0,
};
#endif
