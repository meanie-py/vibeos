/*
 * PROJECT:     ReactOS win32 kernel mode subsystem
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Win32k syscall dispatch tables for main.rs (cohabitation).
 *
 * main.c #included napi.h, which DEFINES the data tables Win32kSSDT /
 * Win32kSSPT / Win32kNumberOfSysCalls from the generated w32ksvc32.h
 * (a (ULONG_PTR)Nt##name, entry per syscall). main.rs declares these as
 * `extern static` (imports) and feeds them to KeAddSystemServiceTable;
 * the generated tables themselves stay in C. This shim is the only TU
 * that includes napi.h now that main.c is replaced by main.rs.
 */

#include <win32k.h>
#include <napi.h>
