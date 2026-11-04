/* d3d8 is Rust (sdk/lib/rust/d3d8-rust); a CMake target needs one TU. */
typedef int d3d8_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char d3d8_export_anchor_0 __asm__("_D3D8GetSWInfo@0");
extern char d3d8_export_anchor_1 __asm__("_DebugSetMute@0");
extern char d3d8_export_anchor_2 __asm__("_Direct3DCreate8@4");
extern char d3d8_export_anchor_3 __asm__("_ValidatePixelShader@16");
extern char d3d8_export_anchor_4 __asm__("_ValidateVertexShader@20");
__attribute__((used)) static const void *const d3d8_export_anchors[] = {
    &d3d8_export_anchor_0,
    &d3d8_export_anchor_1,
    &d3d8_export_anchor_2,
    &d3d8_export_anchor_3,
    &d3d8_export_anchor_4,
    0,
};
#elif defined(__x86_64__)
extern char d3d8_export_anchor_0 __asm__("D3D8GetSWInfo");
extern char d3d8_export_anchor_1 __asm__("DebugSetMute");
extern char d3d8_export_anchor_2 __asm__("Direct3DCreate8");
extern char d3d8_export_anchor_3 __asm__("ValidatePixelShader");
extern char d3d8_export_anchor_4 __asm__("ValidateVertexShader");
__attribute__((used)) static const void *const d3d8_export_anchors[] = {
    &d3d8_export_anchor_0,
    &d3d8_export_anchor_1,
    &d3d8_export_anchor_2,
    &d3d8_export_anchor_3,
    &d3d8_export_anchor_4,
    0,
};
#elif defined(__aarch64__)
extern char d3d8_export_anchor_0 __asm__("D3D8GetSWInfo");
extern char d3d8_export_anchor_1 __asm__("DebugSetMute");
extern char d3d8_export_anchor_2 __asm__("Direct3DCreate8");
extern char d3d8_export_anchor_3 __asm__("ValidatePixelShader");
extern char d3d8_export_anchor_4 __asm__("ValidateVertexShader");
__attribute__((used)) static const void *const d3d8_export_anchors[] = {
    &d3d8_export_anchor_0,
    &d3d8_export_anchor_1,
    &d3d8_export_anchor_2,
    &d3d8_export_anchor_3,
    &d3d8_export_anchor_4,
    0,
};
#endif
