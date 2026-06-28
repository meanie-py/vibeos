/* Wrapper header for bindgen — the SSPI / security-provider / LSA-LPC /
 * NTLM-MSV / wincred type substrate (secur32-sys) for the secur32.dll
 * C-to-Rust translation (secur32-rust).
 *
 * Sequences the SSPI + NDK header context exactly as secur32's precomp.h does
 * (windef/winbase/winnls/winreg + ndk exfuncs/rtlfuncs + secext.h + security.h),
 * then adds the headers the individual .c TUs pull: ntsecapi.h (MSV1_0 and KERB
 * auth-package types), wincred.h (CREDENTIAL_x), lm.h (Net APIs the wine NTLM SSP
 * calls) and lsass/lsass.h
 * (the LSA-LPC port-message structs lsalpc.c marshals to lsasrv). The allowlist
 * keeps the surface to the secur32-specific types absent from ntdll-sys (usermode
 * DLL). Base types it transitively bindgens are simply not re-exported by
 * secur32-rust's prelude. Per-arch (pointer widths + handle layout differ).
 *
 * NOTE: avoid the literal close-comment digraph in this banner. */
#include <stdarg.h>
#define WIN32_NO_STATUS
#define _INC_WINDOWS
#define COM_NO_WINDOWS_H
#define _USE_DUMMY_PSEH 1
#define __SECUR32__
#include <windef.h>
#include <winbase.h>
#include <winnls.h>
#include <winreg.h>
#define NTOS_MODE_USER
#include <ndk/exfuncs.h>
#include <ndk/rtlfuncs.h>
#include <ndk/lpctypes.h>
#include <ndk/lpcfuncs.h>
#include <secext.h>
#define SECURITY_WIN32
#include <security.h>
#include <schannel.h>
#include <psdk/ntsecapi.h>
#include <wincred.h>
#include <lm.h>
#include <lsass/lsass.h>
