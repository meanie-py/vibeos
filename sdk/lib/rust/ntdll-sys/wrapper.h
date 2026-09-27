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

/* Component-private master internal headers: the same move win32k-sys
 * makes by pulling win32kp.h. These declare the intra-component
 * Csr/Sb/Base functions and globals defined across each component's
 * sibling .c files (the win32k Int and co_ analog). Bindgen-ing them
 * lets the resolver see those as catalog symbols instead of phantom
 * "unresolved"; they become Rust items in the output crate at
 * translation time, so the extern decls here are type/signature
 * ground-truth only.
 *
 * srv.h is csrsrv's PCH and resolves via the csrsrv -I already in the
 * lifted build flags (the NINJA_RULE is csrsrv's). It pulls
 * csrsrv/api.h (the CSR server internal API plus global externs).
 *
 * basesrv.h / smss.h are the basesrv / smss master internal headers
 * (Base/BaseSrv and Smp/Sm intra-component surface). They live outside
 * the csrsrv -I, so generate_bindings.sh adds their dirs explicitly.
 * Order: csrsrv, then basesrv (pulls csr/csrsrv.h, guarded), then smss.
 */
#include <srv.h>
#include <basesrv.h>
#include <smss.h>
