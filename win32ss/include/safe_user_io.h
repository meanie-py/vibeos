/*
 * PROJECT:     ReactOS Win32k
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Win32k C-side pre-translation: SEH-free user-buffer validation.
 *
 * Win32k pretranslation Pass 1 (probe-site lowering). These helpers replace
 * the `_SEH2_TRY { ProbeFor*(...); ... } _SEH2_EXCEPT { Status = code; }`
 * idiom with a status-returning validation call, so the normalized C maps
 * 1:1 onto the Rust-native probe API (`ntkrnl-sys/src/probe.rs`) when this
 * file is later translated to Rust:
 *
 *   SafeProbeForRead  -> probe_user_buffer(.., previous_mode=UserMode)
 *   SafeProbeForWrite -> probe_user_buffer(.., previous_mode=UserMode)
 *
 * Unlike the DDK ProbeForRead/ProbeForWrite macros these do NOT raise a
 * software exception (ExRaiseAccessViolation) and do NOT touch-test the
 * page. With DUMMY_PSEH the _SEH2_EXCEPT handler is dead code, so a raised
 * exception or a touch-write fault would crash the kernel uncaught; a
 * status return is both safe and exactly what the Rust target does
 * (validate-only, see probe.rs::probe_user_buffer).
 *
 * See docs/engineering/win32k-pretranslation-audit.md.
 */

#pragma once

NTSTATUS
NTAPI
SafeProbeForRead(
    _In_reads_bytes_(Length) const volatile VOID *Address,
    _In_ SIZE_T Length,
    _In_ ULONG Alignment);

NTSTATUS
NTAPI
SafeProbeForWrite(
    _Out_writes_bytes_(Length) volatile VOID *Address,
    _In_ SIZE_T Length,
    _In_ ULONG Alignment);
