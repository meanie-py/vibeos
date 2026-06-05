/*
 * PROJECT:     ReactOS win32k (Rust cohabitation)
 * PURPOSE:     Exported (cdecl) wrappers around FORCEINLINE / macro / arch-divergent
 *              eng helpers that gdi/ntgdi/path.rs needs but which have no callable
 *              symbol (FLOATOBJ_* are stdcall asm on i386 / float macros elsewhere;
 *              XFORMOBJ_*, MatrixS2XForm, Coord*, RECTL_vSetRect, GDI_HANDLE_GET_TYPE,
 *              DC_pszlViewportExt, IntIsEffectiveWidePen are FORCEINLINE/macros).
 *              Reusing the C definitions here avoids reimplementing soft-float /
 *              matrix / coordinate math in Rust. Thin, plain-cdecl, one call each.
 */

#include <win32k.h>
#include <safe_user_io.h>

/* ---- FLOATOBJ (stdcall asm on i386, FLOAT macros on amd64/arm64) ---- */
VOID w32r_FLOATOBJ_Add(FLOATOBJ *pf, FLOATOBJ *pf1)      { FLOATOBJ_Add(pf, pf1); }
VOID w32r_FLOATOBJ_AddLong(FLOATOBJ *pf, LONG l)         { FLOATOBJ_AddLong(pf, l); }
VOID w32r_FLOATOBJ_Div(FLOATOBJ *pf, FLOATOBJ *pf1)      { FLOATOBJ_Div(pf, pf1); }
VOID w32r_FLOATOBJ_MulLong(FLOATOBJ *pf, LONG l)         { FLOATOBJ_MulLong(pf, l); }
VOID w32r_FLOATOBJ_SetFloat(FLOATOBJ *pf, FLOATL f)      { FLOATOBJ_SetFloat(pf, f); }
BOOL w32r_FLOATOBJ_bConvertToLong(FLOATOBJ *pf, PLONG pl){ return FLOATOBJ_bConvertToLong(pf, pl); }

/* ---- XFORMOBJ / matrix (FORCEINLINE) ---- */
VOID w32r_XFORMOBJ_vInit(XFORMOBJ *pxo, MATRIX *pmx)     { XFORMOBJ_vInit(pxo, pmx); }
ULONG w32r_XFORMOBJ_iGetXform(XFORMOBJ *pxo, XFORML *pxform) { return XFORMOBJ_iGetXform(pxo, pxform); }
BOOL w32r_FLOATOBJ_Equal1(FLOATOBJ *pf)                  { return FLOATOBJ_Equal1(pf); }
VOID w32r_FLOATOBJ_Set1(FLOATOBJ *pf)                    { FLOATOBJ_SetLong(pf, 1); }
BOOL w32r_XFORMOBJ_bApplyXform(XFORMOBJ *pxo, ULONG iMode, ULONG cPoints, PVOID pvIn, PVOID pvOut)
{ return XFORMOBJ_bApplyXform(pxo, iMode, cPoints, pvIn, pvOut); }
VOID w32r_MatrixS2XForm(XFORML *pxform, MATRIX *pmx)     { MatrixS2XForm(pxform, pmx); }
VOID w32r_XFormToMatrix(MATRIX *pmx, XFORML *pxform)     { XFormToMatrix(pmx, pxform); }

/* ---- coordinate transforms (FORCEINLINE / macro) ---- */
VOID w32r_CoordLPtoDP(PDC pdc, PPOINTL ppt)             { CoordLPtoDP(pdc, ppt); }
BOOLEAN w32r_IntDPtoLP(DC *pdc, PPOINTL ppt, UINT count) { return IntDPtoLP(pdc, ppt, count); }
BOOL w32r_IntLPtoDP(DC *pdc, LPPOINT ppt, INT count)    { return IntLPtoDP(pdc, ppt, count); }

/* ---- misc FORCEINLINE / macro ---- */
VOID w32r_RECTL_vSetRect(RECTL *prcl, LONG l, LONG t, LONG r, LONG b) { RECTL_vSetRect(prcl, l, t, r, b); }
PSIZEL w32r_DC_pszlViewportExt(PDC pdc)                 { return DC_pszlViewportExt(pdc); }
ULONG w32r_GDI_HANDLE_GET_TYPE(HGDIOBJ h)              { return (ULONG)GDI_HANDLE_GET_TYPE(h); }
BOOL w32r_IntIsEffectiveWidePen(PBRUSH pbrLine)        { return (BOOL)IntIsEffectiveWidePen(pbrLine); }

/* ---- real fns wrapped for uniform cdecl (avoid ABI guesswork) ---- */
NTSTATUS w32r_SafeProbeForWrite(PVOID Address, SIZE_T Length, ULONG Alignment)
{ return SafeProbeForWrite(Address, Length, Alignment); }
ULONG w32r_ftGdiGetGlyphOutline(PDC pdc, WCHAR wch, UINT format, LPGLYPHMETRICS pgm,
                                ULONG cjBuf, PVOID pvBuf, const MAT2 *pmat2, BOOL bIgnoreRotation)
{ return ftGdiGetGlyphOutline(pdc, wch, format, pgm, cjBuf, pvBuf, pmat2, bIgnoreRotation); }

/* EOF */

/* ---- second batch (more helpers path.rs calls) ---- */
VOID w32r_FLOATOBJ_DivLong(FLOATOBJ *pf, LONG l)        { FLOATOBJ_DivLong(pf, l); }
FLOATL w32r_FLOATOBJ_GetFloat(FLOATOBJ *pf)            { return FLOATOBJ_GetFloat(pf); }
LONG w32r_FLOATOBJ_GetLong(FLOATOBJ *pf)               { return FLOATOBJ_GetLong(pf); }
VOID w32r_FLOATOBJ_Mul(FLOATOBJ *pf, FLOATOBJ *pf2)    { FLOATOBJ_Mul(pf, pf2); }
BOOL w32r_FLOATOBJ_Equal(FLOATOBJ *pf, FLOATOBJ *pf2)  { return FLOATOBJ_Equal(pf, pf2); }
VOID w32r_FLOATOBJ_SetLong(FLOATOBJ *pf, LONG l)        { FLOATOBJ_SetLong(pf, l); }
VOID w32r_FLOATOBJ_Sub(FLOATOBJ *pf, FLOATOBJ *pf2)     { FLOATOBJ_Sub(pf, pf2); }
VOID w32r_INTERNAL_LPTODP(DC *dc, LPPOINT points, UINT count) { INTERNAL_LPTODP(dc, points, count); }

/* gef0/1/2/16 are `static const FLOATOBJ` (i386 struct / amd64+arm64 float) — C
 * can address them on any arch; Rust can't take &of an amd64 `const`. */
FLOATOBJ* w32r_pgef(int n)
{
    switch (n) { case 1: return (FLOATOBJ*)&gef1; case 2: return (FLOATOBJ*)&gef2;
                 case 16: return (FLOATOBJ*)&gef16; default: return (FLOATOBJ*)&gef0; }
}

LONG w32r_RECTL_lGetWidth(RECTL *prcl)  { return RECTL_lGetWidth(prcl); }
LONG w32r_RECTL_lGetHeight(RECTL *prcl) { return RECTL_lGetHeight(prcl); }

/* KeSave/RestoreFloatingPointState: real export on x86, FORCEINLINE on arm64. */
NTSTATUS w32r_KeSaveFloatingPointState(PVOID p)    { return KeSaveFloatingPointState((PKFLOATING_SAVE)p); }
VOID     w32r_KeRestoreFloatingPointState(PVOID p) { KeRestoreFloatingPointState((PKFLOATING_SAVE)p); }
