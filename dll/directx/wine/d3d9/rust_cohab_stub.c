/* d3d9 is Rust (sdk/lib/rust/d3d9-rust); a CMake target needs one TU. */
typedef int d3d9_rust_cohab_placeholder;
/* Every export the .spec requires, referenced from this OBJECT so the module's own
   archive defines it before an import library can (rustconv/cohab_stub.py). */
#if defined(__i386__)
extern char d3d9_export_anchor_0 __asm__("_D3DPERF_BeginEvent@8");
extern char d3d9_export_anchor_1 __asm__("_D3DPERF_EndEvent@0");
extern char d3d9_export_anchor_2 __asm__("_D3DPERF_GetStatus@0");
extern char d3d9_export_anchor_3 __asm__("_D3DPERF_QueryRepeatFrame@0");
extern char d3d9_export_anchor_4 __asm__("_D3DPERF_SetMarker@8");
extern char d3d9_export_anchor_5 __asm__("_D3DPERF_SetOptions@4");
extern char d3d9_export_anchor_6 __asm__("_D3DPERF_SetRegion@8");
extern char d3d9_export_anchor_7 __asm__("_DebugSetMute@0");
extern char d3d9_export_anchor_8 __asm__("_Direct3DCreate9@4");
extern char d3d9_export_anchor_9 __asm__("_Direct3DCreate9Ex@8");
extern char d3d9_export_anchor_10 __asm__("_Direct3DCreate9On12@12");
extern char d3d9_export_anchor_11 __asm__("_Direct3DShaderValidatorCreate9@0");
__attribute__((used)) static const void *const d3d9_export_anchors[] = {
    &d3d9_export_anchor_0,
    &d3d9_export_anchor_1,
    &d3d9_export_anchor_2,
    &d3d9_export_anchor_3,
    &d3d9_export_anchor_4,
    &d3d9_export_anchor_5,
    &d3d9_export_anchor_6,
    &d3d9_export_anchor_7,
    &d3d9_export_anchor_8,
    &d3d9_export_anchor_9,
    &d3d9_export_anchor_10,
    &d3d9_export_anchor_11,
    0,
};
#elif defined(__x86_64__)
extern char d3d9_export_anchor_0 __asm__("D3DPERF_BeginEvent");
extern char d3d9_export_anchor_1 __asm__("D3DPERF_EndEvent");
extern char d3d9_export_anchor_2 __asm__("D3DPERF_GetStatus");
extern char d3d9_export_anchor_3 __asm__("D3DPERF_QueryRepeatFrame");
extern char d3d9_export_anchor_4 __asm__("D3DPERF_SetMarker");
extern char d3d9_export_anchor_5 __asm__("D3DPERF_SetOptions");
extern char d3d9_export_anchor_6 __asm__("D3DPERF_SetRegion");
extern char d3d9_export_anchor_7 __asm__("DebugSetMute");
extern char d3d9_export_anchor_8 __asm__("Direct3DCreate9");
extern char d3d9_export_anchor_9 __asm__("Direct3DCreate9Ex");
extern char d3d9_export_anchor_10 __asm__("Direct3DCreate9On12");
extern char d3d9_export_anchor_11 __asm__("Direct3DShaderValidatorCreate9");
__attribute__((used)) static const void *const d3d9_export_anchors[] = {
    &d3d9_export_anchor_0,
    &d3d9_export_anchor_1,
    &d3d9_export_anchor_2,
    &d3d9_export_anchor_3,
    &d3d9_export_anchor_4,
    &d3d9_export_anchor_5,
    &d3d9_export_anchor_6,
    &d3d9_export_anchor_7,
    &d3d9_export_anchor_8,
    &d3d9_export_anchor_9,
    &d3d9_export_anchor_10,
    &d3d9_export_anchor_11,
    0,
};
#elif defined(__aarch64__)
extern char d3d9_export_anchor_0 __asm__("D3DPERF_BeginEvent");
extern char d3d9_export_anchor_1 __asm__("D3DPERF_EndEvent");
extern char d3d9_export_anchor_2 __asm__("D3DPERF_GetStatus");
extern char d3d9_export_anchor_3 __asm__("D3DPERF_QueryRepeatFrame");
extern char d3d9_export_anchor_4 __asm__("D3DPERF_SetMarker");
extern char d3d9_export_anchor_5 __asm__("D3DPERF_SetOptions");
extern char d3d9_export_anchor_6 __asm__("D3DPERF_SetRegion");
extern char d3d9_export_anchor_7 __asm__("DebugSetMute");
extern char d3d9_export_anchor_8 __asm__("Direct3DCreate9");
extern char d3d9_export_anchor_9 __asm__("Direct3DCreate9Ex");
extern char d3d9_export_anchor_10 __asm__("Direct3DCreate9On12");
extern char d3d9_export_anchor_11 __asm__("Direct3DShaderValidatorCreate9");
__attribute__((used)) static const void *const d3d9_export_anchors[] = {
    &d3d9_export_anchor_0,
    &d3d9_export_anchor_1,
    &d3d9_export_anchor_2,
    &d3d9_export_anchor_3,
    &d3d9_export_anchor_4,
    &d3d9_export_anchor_5,
    &d3d9_export_anchor_6,
    &d3d9_export_anchor_7,
    &d3d9_export_anchor_8,
    &d3d9_export_anchor_9,
    &d3d9_export_anchor_10,
    &d3d9_export_anchor_11,
    0,
};
#endif
