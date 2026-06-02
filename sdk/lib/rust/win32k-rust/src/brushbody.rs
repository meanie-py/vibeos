// Shared BRUSHBODY view for the gdi/ntgdi geometry wave (arc, stockobj, …).
//
// The C `_BRUSH` struct (win32ss/gdi/ntgdi/brush.h) embeds an anonymous
// `BRUSHBODY` right after `BASEOBJECT`. bindgen loses the field names of the
// anonymous member and emits `__bindgen_padding`/no fields, so `_BRUSH` has no
// reachable `iHatch`/`lWidth`/`ulPenStyle`/… The engbrush.rs `BRUSHBODY_View`
// pattern solves this for the brush-only fields; here we reconstruct the FULL
// BRUSHBODY (including the trailing PEN fields) so arc/stockobj can touch
// `lWidth`, `eWidth`, `ulPenStyle`, `iBrushStyle`, etc.
//
// CRITICAL: the layout is built from real win32k_sys types (FLOATOBJ,
// _BRUSH_ATTR, POINTL, pointers, HDEV) so rustc computes the correct per-arch
// offsets. `eWidth: FLOATOBJ` is 8 bytes on i386 (struct) but 4 bytes (FLOAT)
// on amd64/arm64, and every pointer differs 4↔8 — hardcoded offsets would be
// wrong. Mirror brush.h exactly; do not reorder.

use win32k_sys::win32k::*;

#[repr(C)]
pub struct BrushBodyView {
    pub iHatch: ULONG,            // hatch style, not brush style
    pub hbmPattern: HBITMAP,
    pub hbmClient: HBITMAP,
    pub flAttrs: ULONG,

    pub ulBrushUnique: ULONG,
    pub pBrushAttr: *mut _BRUSH_ATTR,
    pub BrushAttr: _BRUSH_ATTR,
    pub ptOrigin: POINTL,         // POINT == POINTL layout
    pub bCacheGrabbed: ULONG,
    pub crBack: COLORREF,
    pub crFore: COLORREF,
    pub ulPalTime: ULONG,
    pub ulSurfTime: ULONG,
    pub pvRBrush: PVOID,
    pub hdev: HDEV,

    // ---- PEN-only fields ----
    pub lWidth: LONG,
    pub eWidth: FLOATOBJ,
    pub ulPenStyle: ULONG,
    pub pStyle: *mut DWORD,
    pub dwStyleCount: ULONG,
    pub jJoin: BYTE,
    pub jEndCap: BYTE,
    pub iBrushStyle: INT,
    pub ulStyleSize: ULONG,
    pub ListHead: LIST_ENTRY,
}

/// View the BRUSHBODY embedded in a `_BRUSH` (immediately after BASEOBJECT).
#[inline(always)]
pub unsafe fn pbrushbody(pbrush: PBRUSH) -> *mut BrushBodyView {
    (pbrush as *mut u8).add(core::mem::size_of::<BASEOBJECT>()) as *mut BrushBodyView
}
