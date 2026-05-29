/*
 * consrv_rust_shim.c
 *
 * Non-inline providers for the C macros / MSVC intrinsics that the Rust
 * consrv translation (sdk/lib/rust/winsrv-rust/src/consrv/**) extern-declares
 * but for which no exported symbol exists once the top-level consrv/*.c files
 * are replaced by their Rust counterparts.
 *
 * Modeled on win32ss/user/winsrv/usersrv/usersrv_rust_strsafe_shim.c.
 *
 * Self-contained (no SDK headers) on purpose: pulling in consrv.h / shlobj.h
 * would re-declare these as macros/inline and collide with the external
 * definitions below.  The ABI types used here are layout-stable.
 *
 * Decoration matters on i386:
 *   - extern "C"      Rust decl  -> undecorated  `_name`
 *   - extern "system" Rust decl  -> stdcall      `_name@N`
 * We provide whichever decoration the Rust references use (see each block).
 */

typedef long             LONG;
typedef long             NTSTATUS;
typedef unsigned short   USHORT;
typedef unsigned short   WCHAR;
typedef unsigned long    ULONG;
typedef int              HRESULT;
typedef void            *PVOID;
typedef void            *LPVOID;
typedef const void      *LPCVOID;

/* mk_wcwidth_cjk lives in frontends/wcwidth.c (still C, cdecl). */
extern int mk_wcwidth_cjk(WCHAR ucs);

/*
 * IS_FULL_WIDTH(wch) — C macro in the original source:
 *     (((USHORT)(wch) >= 0x0080) && (mk_wcwidth_cjk(wch) == 2))
 * The Rust condrv/text.rs + frontends/terminal.rs reference it via an
 * `extern "system"` decl → `_IS_FULL_WIDTH@4` (__stdcall, 1 arg).
 */
unsigned long __stdcall
IS_FULL_WIDTH(WCHAR wch)
{
    return (((USHORT)wch >= 0x0080) && (mk_wcwidth_cjk(wch) == 2)) ? 1UL : 0UL;
}

/*
 * IsListEmpty(ListHead) — NT macro. LIST_ENTRY.Flink is the first field; the
 * list is empty iff Flink points back at the head.  Referenced by the Rust
 * condrv with the PLAIN (cdecl) name `_IsListEmpty`.  Returns BOOLEAN (UCHAR).
 */
unsigned char
IsListEmpty(LPCVOID ListHead)
{
    return (unsigned char)(*(const void *const *)ListHead == ListHead);
}

/*
 * MSVC interlocked intrinsics.  `_Interlocked*` are clang builtins, so we
 * cannot define functions by those literal names; declare under a private C
 * identifier with an asm label that emits the exact symbol referenced.
 *
 * The Rust consrv references both decorations on i386:
 *   - condrv/coninput.rs declares `extern "system"`  -> `__Interlocked*@4`
 *   - frontends/terminal.rs declares `extern "C"`     -> `__InterlockedDecrement`
 */
#if defined(__i386__)
LONG __stdcall shim_ili_stdcall(LONG volatile *p) __asm__("__InterlockedIncrement@4");
LONG __stdcall shim_ild_stdcall(LONG volatile *p) __asm__("__InterlockedDecrement@4");
LONG __cdecl   shim_ild_cdecl  (LONG volatile *p) __asm__("__InterlockedDecrement");

LONG __stdcall shim_ili_stdcall(LONG volatile *p)
{ return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }
LONG __stdcall shim_ild_stdcall(LONG volatile *p)
{ return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
LONG __cdecl   shim_ild_cdecl(LONG volatile *p)
{ return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
#endif /* __i386__ */

/*
 * IShellLinkW / IPersistFile COBJMACROS dispatchers.
 *
 * In the original C, IShellLinkW_GetShowCmd(p, ...) etc. expand inline to
 * `(p)->lpVtbl->Method(p, ...)`.  The Rust LoadShellLinkConsoleInfo
 * translation declares them as `extern "system"` functions → `_Name@N`
 * (__stdcall).  COM methods are themselves __stdcall (STDMETHODCALLTYPE), so
 * we dispatch through the object's vtbl here.
 *
 * vtbl layout (IUnknown first):  0 QueryInterface, 1 AddRef, 2 Release.
 *   IShellLinkW : IUnknown
 *     3 GetPath, 4 GetIDList, 5 SetIDList, 6 GetDescription,
 *     7 SetDescription, 8 GetWorkingDirectory, 9 SetWorkingDirectory,
 *     10 GetArguments, 11 SetArguments, 12 GetHotkey, 13 SetHotkey,
 *     14 GetShowCmd, 15 SetShowCmd, 16 GetIconLocation, ...
 *   IPersistFile : IPersist : IUnknown
 *     3 GetClassID, 4 IsDirty, 5 Load, 6 Save, ...
 */

/* A COM object is `{ const VtblEntry **lpVtbl; ... }`; every method takes the
 * `this` pointer as its first (__stdcall) argument. */
#define COM_VTBL(p)            (*(void ***)(p))
#define COM_METHOD(p, idx)     (COM_VTBL(p)[(idx)])

typedef HRESULT (__stdcall *PFN_QI)(LPVOID, const void *, LPVOID *);
typedef ULONG   (__stdcall *PFN_REL)(LPVOID);
typedef HRESULT (__stdcall *PFN_GETSHOWCMD)(LPVOID, int *);
typedef HRESULT (__stdcall *PFN_GETICONLOC)(LPVOID, WCHAR *, int, int *);
typedef HRESULT (__stdcall *PFN_PFLOAD)(LPVOID, const WCHAR *, ULONG);

HRESULT __stdcall
IShellLinkW_GetShowCmd(LPVOID pshl, int *ShowCmd)
{
    return ((PFN_GETSHOWCMD)COM_METHOD(pshl, 14))(pshl, ShowCmd);
}

HRESULT __stdcall
IShellLinkW_GetIconLocation(LPVOID pshl, WCHAR *IconPath, int cchIconPath, int *IconIndex)
{
    return ((PFN_GETICONLOC)COM_METHOD(pshl, 16))(pshl, IconPath, cchIconPath, IconIndex);
}

ULONG __stdcall
IShellLinkW_Release(LPVOID pshl)
{
    return ((PFN_REL)COM_METHOD(pshl, 2))(pshl);
}

HRESULT __stdcall
IPersistFile_QueryInterface(LPVOID pshl, const void *riid, LPVOID *ppvObject)
{
    return ((PFN_QI)COM_METHOD(pshl, 0))(pshl, riid, ppvObject);
}

HRESULT __stdcall
IPersistFile_Load(LPVOID ppf, const WCHAR *pszFileName, ULONG dwMode)
{
    return ((PFN_PFLOAD)COM_METHOD(ppf, 5))(ppf, pszFileName, dwMode);
}

ULONG __stdcall
IPersistFile_Release(LPVOID ppf)
{
    return ((PFN_REL)COM_METHOD(ppf, 2))(ppf);
}
