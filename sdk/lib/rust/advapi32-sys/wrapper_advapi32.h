/* Wrapper header for bindgen — the Win32 SDK / RPC-client / winsafer / aclapi /
 * evntrace / EFS type substrate (advapi32-sys) for the advapi32.dll C-to-Rust
 * translation.
 *
 * Pulls the advapi32-specific type surface by including the module's own
 * advapi32.h (its win32+NDK context already sequences windows.h, aclapi.h,
 * winsafer.h, ntsecapi.h, services/services.h and the generated
 * svcctl_c.h/winreg_c.h/lsa_c.h/seclogon_c.h/eventlogrpc_c.h RPC client stubs
 * correctly) plus wine/crypt.h (ustring + CRYPT_ decls). EVENT_TRACE / TRACE-
 * HANDLE types are hand-declared in the prelude (evntrace.h needs wmistr.h
 * ordering bindgen can't satisfy). The allowlist keeps the surface to the
 * advapi32-specific types absent from ntdll-sys (this is a usermode DLL). Base
 * types it transitively bindgens are simply not re-exported by advapi32-rust's
 * prelude. Per-arch (pointer widths + RPC string layout differ).
 *
 * NOTE: avoid the literal close-comment digraph in this banner. */
#include <stdarg.h>
#define WIN32_NO_STATUS
#define _USE_DUMMY_PSEH 1
#define _ADVAPI32_
#include "advapi32.h"
#include <lsa_c.h>
#include <eventlogrpc_c.h>
#include <wincred.h>
#include <winefs.h>
#include "wine/crypt.h"
#include <md4.h>
#include <md5.h>
