//! Native process entry point — Rust port of `sdk/lib/nt/entry_point.c`
//! (ReactOS native NT runtime, Alex Ionescu). This is the PE entry for the
//! native subsystem exes smss.exe and csrss.exe (`_NtProcessStartup@4` on i386,
//! `NtProcessStartup` on amd64/arm64). It normalizes the process parameters,
//! builds a C-style `argc`/`argv`/`envp` from the command line + environment
//! block, calls the binary's `_main`, and terminates the process with the
//! result.
//!
//! Porting it here lets both smss-cargo and csrss-cargo drop cmake's
//! `libnt.a` (its sole symbol was `_NtProcessStartup@4`). Compiled only under
//! the two native-exe feature sets — the DLL builds (csrsrv/basesrv) keep their
//! own `DllMain` entry and never reference `_main`.

use ntdll_sys::nt::*;

extern "C" {
    /// The binary's real entry — csrss::_main / smss::_main (whichever this
    /// feature build emits as `#[no_mangle] _main`). `__cdecl` in the C.
    fn _main(argc: i32, argv: *mut *mut i8, envp: *mut *mut i8, DebugFlag: ULONG) -> NTSTATUS;
}

/// Port of `EnvironmentStringToUnicodeString`: wrap a double-NUL-terminated
/// wide environment block in a UNICODE_STRING (Length/MaximumLength include the
/// terminating wide NULs, matching the C — see its FIXME).
unsafe fn environment_string_to_unicode_string(ws_in: PWSTR, us_out: *mut UNICODE_STRING) {
    if !ws_in.is_null() {
        let mut current = ws_in;
        // Walk past each NUL-terminated variable until the empty string.
        while *current != 0 {
            while {
                let c = *current;
                current = current.add(1);
                c != 0
            } {}
        }
        // Skip the final (double) NUL terminator.
        current = current.add(1);

        (*us_out).Buffer = ws_in;
        let chars = current.offset_from(ws_in) as usize;
        let bytes = (chars * core::mem::size_of::<u16>()) as u16;
        (*us_out).Length = bytes;
        (*us_out).MaximumLength = bytes;
    } else {
        (*us_out).Buffer = core::ptr::null_mut();
        (*us_out).Length = 0;
        (*us_out).MaximumLength = 0;
    }
}

#[inline(always)]
unsafe fn process_heap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

#[inline(always)]
unsafe fn current_process() -> HANDLE {
    -1isize as HANDLE
}

/// Port of `NtProcessStartup`. NTAPI (= stdcall on i386 → `_NtProcessStartup@4`).
#[no_mangle]
pub unsafe extern "system" fn NtProcessStartup(peb: PPEB) {
    let mut status: NTSTATUS;
    let mut ansi_cmd_line: ANSI_STRING = core::mem::zeroed();
    let mut unicode_environment: UNICODE_STRING = core::mem::zeroed();
    let mut ansi_environment: ANSI_STRING = core::mem::zeroed();
    let mut argc: i32 = 0;

    // Normalize and get the Process Parameters.
    let process_parameters = RtlNormalizeProcessParams((*peb).ProcessParameters);  // nolint: raw-user-access (user-mode process entry, not a syscall: the PEB is its own)

    // Argument list: enough for 512 tokens (matches the C; FIXME upstream too).
    let argument_list = RtlAllocateHeap(
        process_heap(),
        0,
        (512 * core::mem::size_of::<*mut i8>()) as SIZE_T,
    ) as *mut *mut i8;
    if argument_list.is_null() {
        status = STATUS_NO_MEMORY;
        NtTerminateProcess(current_process(), status);
        return;
    }

    // argv starts at the list head (the C seeds slot 0 with NULL first).
    let mut cursor = argument_list;
    *cursor = core::ptr::null_mut();
    let argv: *mut *mut i8 = argument_list;

    // Command line (fall back to the image path if empty).
    let mut cmd_line_string: PUNICODE_STRING = &mut (*process_parameters).CommandLine;
    if (*cmd_line_string).Buffer.is_null() || (*cmd_line_string).Length == 0 {
        cmd_line_string = &mut (*process_parameters).ImagePathName;
    }

    // Convert to ANSI (allocates ansi_cmd_line.Buffer).
    status = RtlUnicodeStringToAnsiString(&mut ansi_cmd_line, cmd_line_string, TRUE as BOOLEAN);
    if !nt_success(status) {
        NtTerminateProcess(current_process(), status);
        return;
    }

    let mut source = ansi_cmd_line.Buffer as *mut i8;
    let length = ansi_cmd_line.Length;

    if !source.is_null() {
        // Destination buffer for the whitespace-tokenized argument bytes.
        let destination_base = RtlAllocateHeap(
            process_heap(),
            0,
            (length as usize + core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut i8;
        if destination_base.is_null() {
            status = STATUS_NO_MEMORY;
            NtTerminateProcess(current_process(), status);
            return;
        }
        let mut destination = destination_base;

        while *source != 0 {
            // Skip whitespace.
            while *source != 0 && *source <= b' ' as i8 {
                source = source.add(1);
            }
            // Copy one token, NUL-terminate it.
            if *source != 0 {
                *cursor = destination;
                cursor = cursor.add(1);
                argc += 1;

                while *source > b' ' as i8 {
                    *destination = *source;
                    destination = destination.add(1);
                    source = source.add(1);
                }
                *destination = 0;
                destination = destination.add(1);
            }
        }
    }

    // NULL-terminate the argv list; envp begins right after.
    *cursor = core::ptr::null_mut();
    cursor = cursor.add(1);
    let envp: *mut *mut i8 = cursor;

    if !(*process_parameters).Environment.is_null() {
        environment_string_to_unicode_string(
            (*process_parameters).Environment,
            &mut unicode_environment,
        );
        status = RtlUnicodeStringToAnsiString(
            &mut ansi_environment,
            &mut unicode_environment,
            TRUE as BOOLEAN,
        );
        if !nt_success(status) {
            NtTerminateProcess(current_process(), status);
            return;
        }

        let mut env_source = ansi_environment.Buffer as *mut i8;
        while *env_source != 0 {
            *cursor = env_source;
            cursor = cursor.add(1);
            // Advance past this NUL-terminated variable.
            while {
                let c = *env_source;
                env_source = env_source.add(1);
                c != 0
            } {}
        }
        // NULL-terminate the combined argv+envp list (the C's trailing
        // `*ArgumentList++ = NULL`; the post-increment result is unused).
        *cursor = core::ptr::null_mut();
    }

    // Break if requested.
    if (*process_parameters).DebugFlags != 0 {
        DbgBreakPoint();
    }

    // Call the binary's main, then terminate with its result.
    status = _main(argc, argv, envp, (*process_parameters).DebugFlags);
    NtTerminateProcess(current_process(), status);
}

#[inline(always)]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}
