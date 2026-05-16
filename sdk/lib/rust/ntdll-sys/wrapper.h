/* ntdll-sys bindgen wrapper -- user-mode Native API + SM/CSR/BASE
 * subsystem-protocol FFI surface for the SMSS->CSRSS translation track.
 *
 * User-mode analog of ntkrnl-sys and win32k-sys. ntkrnl-sys bindgens
 * the kernel-internal surface only (no user-mode Rtl/Csr/Sm surface),
 * which is the wrong crate for native user-mode processes (smss.exe,
 * csrss.exe, csrsrv.dll, basesrv.dll, winsrv.dll). See
 * docs/engineering/smss-csrss-translation-audit.md (Substrate section).
 *
 * Uses -fms-extensions but NEVER -fms-compatibility (defines _MSC_VER
 * then mingw _mingw.h takes the MSVC branch and breaks math.h).
 *
 * Include order/contract mirrors the components own PCHs (csrsrv
 * srv.h, basesrv basesrv.h, smss smss.h): stdio first, then
 * WIN32_NO_STATUS + win base headers, then NTOS_MODE_USER + the NDK
 * umbrella, then the sm/csr/win subsystem-protocol headers. ndk/ntndk.h
 * is the NT-internal umbrella ONLY; the SM/CSR/BASE message structs
 * come from the subsys headers enumerated explicitly below (audit OQ2).
 */
#define __REACTOS__
#define _USE_DUMMY_PSEH 1
/* Skip compiler intrinsics -- not available in libclang. */
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
/* ARM64 intrinsic stubs (ntkrnl-sys pattern). */
#define __wfi() ((void)0)
#define __yield() ((void)0)
#undef FORCEINLINE
#define FORCEINLINE

/* vcruntime/vadefs.h (pulled by <stdio.h>) references __gnuc_va_list
   without including stdarg itself; the lifted -nostdinc flags + the
   ROS sdk/include/crt <stdarg.h> shadowing clang's drop the implicit
   setup. __builtin_va_list is a true clang builtin, always available. */
typedef __builtin_va_list __gnuc_va_list;

#include <stdio.h>

/* Win32 base headers (provide TIME_ZONE_INFORMATION, LPSTARTUPINFOA,
   etc. that win/base.h + win/basemsg.h reference). */
#define WIN32_NO_STATUS
#define _INC_WINDOWS
#define COM_NO_WINDOWS_H
#include <windef.h>
#include <winbase.h>
#include <winreg.h>
#include <dbt.h>

/* NT-internal umbrella in USER mode (Nt/Zw/Rtl/Ldr/Dbg + um/ke/mm/ob/
   ps/se/ex types). NTOS_MODE_USER selects the user-mode NDK. */
#define NTOS_MODE_USER
#include <ndk/ntndk.h>

/* SM (Session Manager) port + message protocol. */
#include <sm/ns.h>
#include <sm/smmsg.h>

/* CSR (Client/Server Runtime) API + LPC messages + server surface. */
#include <csr/csr.h>
#include <csr/csrmsg.h>
#include <csr/csrsrv.h>

/* BASE API server protocol (basemsg before base, per basesrv.h). */
#include <win/basemsg.h>
#include <win/base.h>
