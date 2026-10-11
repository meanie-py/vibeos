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

/* mk_wcwidth_cjk is Rust (sdk/lib/rust/wcwidth-rust, exported cdecl). */
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

/* _InterlockedExchange — condrv/coninput.rs extern "system" -> __InterlockedExchange@8. */
LONG __stdcall shim_ixchg_stdcall(LONG volatile *p, LONG v) __asm__("__InterlockedExchange@8");
LONG __stdcall shim_ixchg_stdcall(LONG volatile *p, LONG v)
{ return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }

/* CONTAINING_RECORD(addr, type, field_off) — NT macro; condrv coninput/conoutput.rs
 * extern "system" (3 ptr-sized args) -> _CONTAINING_RECORD@12.  type is unused. */
PVOID __stdcall shim_containing_record(const void *addr, PVOID type, ULONG field_off) __asm__("_CONTAINING_RECORD@12");
PVOID __stdcall shim_containing_record(const void *addr, PVOID type, ULONG field_off)
{ (void)type; return (PVOID)((const char *)addr - field_off); }

/* InitializeObjectAttributes(p, ObjectName, Attributes, RootDir, SecDesc) — NT
 * macro; condrv/coninput.rs extern "system" (5 args) -> _InitializeObjectAttributes@20. */
struct shim_obja { ULONG Length; PVOID RootDirectory; PVOID ObjectName; ULONG Attributes; PVOID SecurityDescriptor; PVOID SecurityQualityOfService; };
void __stdcall shim_init_obja(struct shim_obja *p, PVOID n, ULONG a, PVOID r, PVOID s) __asm__("_InitializeObjectAttributes@20");
void __stdcall shim_init_obja(struct shim_obja *p, PVOID n, ULONG a, PVOID r, PVOID s)
{ p->Length = sizeof(struct shim_obja); p->RootDirectory = r; p->ObjectName = n;
  p->Attributes = a; p->SecurityDescriptor = s; p->SecurityQualityOfService = (PVOID)0; }
#else /* !__i386__ : amd64 / arm64 */
/*
 * On amd64/arm64 there is no stdcall decoration and no i386 leading-underscore:
 * the Rust consrv condrv/coninput.rs + conoutput.rs `extern "system"` decls
 * resolve to the UNDECORATED names below. The macro/intrinsic providers were
 * previously i386-only; a stale C consrv archive (ar never prunes) masked their
 * absence on amd64/arm64 until the archive was rebuilt. Same logic as the i386
 * block, minus decoration; CONTAINING_RECORD takes a pointer-sized offset
 * (Rust `usize`, 64-bit here) rather than i386's ULONG.
 */
LONG shim_ili(LONG volatile *p) __asm__("_InterlockedIncrement");
LONG shim_ild(LONG volatile *p) __asm__("_InterlockedDecrement");
LONG shim_ixchg(LONG volatile *p, LONG v) __asm__("_InterlockedExchange");
LONG shim_ili(LONG volatile *p)   { return __atomic_add_fetch(p, 1, __ATOMIC_SEQ_CST); }
LONG shim_ild(LONG volatile *p)   { return __atomic_sub_fetch(p, 1, __ATOMIC_SEQ_CST); }
LONG shim_ixchg(LONG volatile *p, LONG v) { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }

PVOID shim_containing_record(const void *addr, PVOID type, unsigned long long field_off) __asm__("CONTAINING_RECORD");
PVOID shim_containing_record(const void *addr, PVOID type, unsigned long long field_off)
{ (void)type; return (PVOID)((const char *)addr - field_off); }

struct shim_obja { ULONG Length; PVOID RootDirectory; PVOID ObjectName; ULONG Attributes; PVOID SecurityDescriptor; PVOID SecurityQualityOfService; };
void shim_init_obja(struct shim_obja *p, PVOID n, ULONG a, PVOID r, PVOID s) __asm__("InitializeObjectAttributes");
void shim_init_obja(struct shim_obja *p, PVOID n, ULONG a, PVOID r, PVOID s)
{ p->Length = sizeof(struct shim_obja); p->RootDirectory = r; p->ObjectName = n;
  p->Attributes = a; p->SecurityDescriptor = s; p->SecurityQualityOfService = (PVOID)0; }
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
