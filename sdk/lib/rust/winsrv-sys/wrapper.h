/* winsrv-sys bindgen wrapper — winsrv.dll's USER + Console server FFI
 * surface (consrv/usersrv/concfg static libs + winsrv.dll top-level).
 *
 * Phase 5 of the SMSS+CSRSS translation track. User-mode analog of
 * ntkrnl-sys / win32k-sys / ntdll-sys, but ABOVE ntdll-sys: pulls
 * wincon/winuser/wingdi/imm + the internal headers that ntdll-sys
 * intentionally stops short of (CONSOLE_PROCESS_DATA, CONSRV_*,
 * TERMINAL_VTBL, FRONTEND_VTBL, SHUTDOWN_SETTINGS, …). See
 * docs/engineering/smss-csrss-translation-audit.md (Phase 5 plan) and
 * memory `project_phase5_winsrv_audit`.
 *
 * Uses -fms-extensions (consrv anon nested struct/union, e.g.
 * CONSOLE_SCREEN_BUFFER) but NEVER -fms-compatibility (would flip
 * mingw's _mingw.h to the MSVC branch and break math.h, same gotcha
 * as ntkrnl-sys/win32k-sys/ntdll-sys).
 */
#define __REACTOS__
#define _USE_DUMMY_PSEH 1
/* Skip compiler intrinsics — not available in libclang. */
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE

/* vcruntime/vadefs.h compatibility (same as ntdll-sys). */
typedef __builtin_va_list __gnuc_va_list;

/* concfg/precomp.h pulls a small subset (no winsrv.h, NTOS_MODE_USER)
   so it is includable BEFORE the winsrv master without duplication
   risk thanks to #include guards. We pull it explicitly so the
   bindgen sees TT_FONT_ENTRY, CONSOLE_STATE_INFO, etc. */
#include "concfg/precomp.h"
#include "concfg/concfg.h"

/* winsrv master header (windef/winbase/wingdi/winreg/winuser + imm/
   imm32_undoc + undocuser + NDK cm/ex/ob/ps/rtl + ntuser + csr/csrsrv +
   SHUTDOWN_SETTINGS). */
#include "winsrv.h"

/* consrv internal master (winsrv.h + strsafe + winnls/wincon/
   wincon_undoc + mmfuncs + win/conmsg + heap.h + the consrv-private
   conio/console/settings/term/console/coninput/conoutput/handle/
   lineinput headers + ConsoleClientShutdown decl). */
#include "consrv/consrv.h"

/* usersrv internal master (winsrv.h + win/base + win/winmsg +
   FindTopLevelWnd + UserInitHardErrorsCache decls). */
#include "usersrv/usersrv.h"

/* usersrv harderror.c pulls undocelfapi for ElfReportEventW/
   ElfRegisterEventSourceW/ElfDeregisterEventSource and undocuser
   for MessageBoxTimeoutW (already pulled by winsrv.h, but
   undocelfapi.h is harderror-only). */
#include <undocelfapi.h>

/* usersrv resource IDs (IDS_SEVERITY_*, IDS_SYSTEM_PROCESS, IDC_END_NOW,
   IDD_END_NOW, IDS_OK_TERMINATE_PROGRAM, IDS_CANCEL_DEBUG_PROGRAM …)
   live in usersrv/resource.h and are referenced by harderror.c +
   shutdown.c. Pull explicitly — they're #defines bindgen DOES read
   when the header is included. */
#include "usersrv/resource.h"

/* tuiterm.c pulls NT IO surface (NtLoadDriver/NtUnloadDriver/NtOpenFile/
   NtDeviceIoControlFile + IO_STATUS_BLOCK + FILE_*). NTOS_MODE_USER is
   set by winsrv.h above, so the user-mode ndk/iofuncs.h shape is in
   scope. Add the IOCTL_CONSOLE_* constants the BlueScreen driver
   exposes (ntddblue.h). */
#include <ndk/iofuncs.h>
#include <reactos/drivers/blue/ntddblue.h>

/* Phase 5.6: gui/ frontend private headers — GUI_CONSOLE_DATA,
   GUI_CONSOLE_INFO, FONT_*, KEY_TOGGLED/PRESSED, PM_* WM_APP
   commands, plus the guisettings/guiterm function decls used by
   conwnd/fullscreen/graphics/guisettings/guiterm/text. Order
   matters: guisettings.h defines GUI_CONSOLE_INFO used by
   conwnd.h's GUI_CONSOLE_DATA. */
#include "consrv/frontends/gui/guisettings.h"
#include "consrv/frontends/gui/conwnd.h"
#include "consrv/frontends/gui/guiterm.h"

/* conwnd.c uses DragQueryFileW/DragFinish/DragAcceptFiles via
   <shellapi.h> — pull explicitly for bindgen. */
#include <shellapi.h>
/* conwnd.c also uses strsafe.h's StringCchCopyNW/StringCchCatW. */
#include <strsafe.h>
