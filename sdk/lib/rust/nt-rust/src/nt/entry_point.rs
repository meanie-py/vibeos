use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS  (valueless flag macro; no Rust declaration emitted)
// #define NTOS_MODE_USER   (valueless flag macro; no Rust declaration emitted)
// NTSTATUS __cdecl _main(int argc, char *argv[], char *envp[], ULONG DebugFlag);
//   (prototype only; no definition here. Declared by the caller's extern block if needed)
// #define NDEBUG           (valueless flag macro; no Rust declaration emitted)
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn EnvironmentStringToUnicodeString(mut wsIn: PWCHAR, mut usOut: PUNICODE_STRING) {
    if !wsIn.is_null() {
        let mut CurrentChar: PWCHAR = wsIn;
        while *CurrentChar != 0 {
            loop {
                let fresh15 = CurrentChar;
                CurrentChar = CurrentChar.offset(1);
                if !(*fresh15 != 0) {
                    break;
                }
            }
        }
        CurrentChar = CurrentChar.offset(1);
        (*usOut).Buffer = wsIn as PWSTR;
        (*usOut).Length = (CurrentChar.offset_from(wsIn) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT;
        (*usOut).MaximumLength = (*usOut).Length;
    } else {
        (*usOut).Buffer = core::ptr::null_mut::<WCHAR>();
        (*usOut).Length = 0;
        (*usOut).MaximumLength = (*usOut).Length;
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EnvironmentStringToUnicodeString(mut wsIn: PWCHAR, mut usOut: PUNICODE_STRING) {
    if !wsIn.is_null() {
        let mut CurrentChar: PWCHAR = wsIn;
        while *CurrentChar != 0 {
            loop {
                let fresh15 = CurrentChar;
                CurrentChar = CurrentChar.offset(1);
                if !(*fresh15 != 0) {
                    break;
                }
            }
        }
        CurrentChar = CurrentChar.offset(1);
        (*usOut).Buffer = wsIn as PWSTR;
        (*usOut).Length = (CurrentChar.offset_from(wsIn) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT;
        (*usOut).MaximumLength = (*usOut).Length;
    } else {
        (*usOut).Buffer = core::ptr::null_mut::<WCHAR>();
        (*usOut).Length = 0;
        (*usOut).MaximumLength = (*usOut).Length;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtProcessStartup(Peb: PPEB) {
    let mut Status: NTSTATUS = 0;
    let mut ProcessParameters: PRTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();
    let mut CmdLineString: PUNICODE_STRING = core::ptr::null_mut();
    let mut AnsiCmdLine: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeEnvironment: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiEnvironment: ANSI_STRING = core::mem::zeroed();
    let mut NullPointer: PCHAR = core::ptr::null_mut();
    let mut argc: INT = 0;
    let mut argv: *mut PCHAR;
    let mut envp: *mut PCHAR;
    let mut ArgumentList: *mut PCHAR;
    let mut Source: PCHAR;
    let mut Destination: PCHAR;
    let mut Length: ULONG;
    ASSERT!(!Peb.is_null());

    if false {
        DPRINT1!("%s(%08lx) called\n", "NtProcessStartup\0".as_ptr(), Peb);
        while TRUE != 0 {}
    }

    /* Normalize and get the Process Parameters */
    ProcessParameters = RtlNormalizeProcessParams((*Peb).ProcessParameters);
    ASSERT!(!ProcessParameters.is_null());

    'fail: {
        /* Allocate memory for the argument list, enough for 512 tokens */
        // FIXME: what if 512 is not enough????
        ArgumentList = RtlAllocateHeap(
            RtlGetProcessHeap!(),
            0,
            (512 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<PCHAR>() as SIZE_T),
        ) as *mut PCHAR;
        if ArgumentList.is_null() {
            DPRINT1!("ERR: no mem!");
            Status = STATUS_NO_MEMORY;
            break 'fail;
        }

        /* Use a null pointer as default */
        argv = core::ptr::addr_of_mut!(NullPointer);
        envp = core::ptr::addr_of_mut!(NullPointer);

        /* Set the first pointer to NULL, and set the argument array to the buffer */
        *ArgumentList = core::ptr::null_mut();
        argv = ArgumentList;

        /* Get the pointer to the Command Line */
        CmdLineString = core::ptr::addr_of_mut!((*ProcessParameters).CommandLine);

        /* If we don't have a command line, use the image path instead */
        if (*CmdLineString).Buffer.is_null() || (*CmdLineString).Length == 0 {
            CmdLineString = core::ptr::addr_of_mut!((*ProcessParameters).ImagePathName);
        }

        /* Convert it to an ANSI string */
        Status = RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(AnsiCmdLine),
            CmdLineString,
            (TRUE) as BOOLEAN,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("ERR: no mem(guess)\n");
            break 'fail;
        }

        /* Save parameters for parsing */
        Source = AnsiCmdLine.Buffer;
        Length = AnsiCmdLine.Length as ULONG;

        /* Ensure it's valid */
        if !Source.is_null() {
            /* Allocate a buffer for the destination */
            Destination = RtlAllocateHeap(
                RtlGetProcessHeap!(),
                0,
                (Length as SIZE_T).wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PCHAR;
            if Destination.is_null() {
                DPRINT1!("ERR: no mem!");
                Status = STATUS_NO_MEMORY;
                break 'fail;
            }

            /* Start parsing */
            while *Source != 0 {
                /* Skip the white space */
                while *Source != 0 && *Source as i32 <= ' ' as i32 {
                    Source = Source.add(1);
                }

                /* Copy until the next white space is reached */
                if *Source != 0 {
                    /* Save one token pointer */
                    *ArgumentList = Destination;
                    ArgumentList = ArgumentList.add(1);

                    /* Increase one token count */
                    argc += 1;

                    /* Copy token until white space */
                    while *Source as i32 > ' ' as i32 {
                        *Destination = *Source;
                        Destination = Destination.add(1);
                        Source = Source.add(1);
                    }

                    /* Null terminate it */
                    *Destination = '\0' as i32 as CHAR;
                    Destination = Destination.add(1);
                }
            }
        }

        /* Null terminate the token pointer list */
        *ArgumentList = core::ptr::null_mut();
        ArgumentList = ArgumentList.add(1);

        /* Now handle the environment, point the envp at our current list location. */
        envp = ArgumentList;

        if !(*ProcessParameters).Environment.is_null() {
            EnvironmentStringToUnicodeString(
                (*ProcessParameters).Environment,
                core::ptr::addr_of_mut!(UnicodeEnvironment),
            );
            Status = RtlUnicodeStringToAnsiString(
                core::ptr::addr_of_mut!(AnsiEnvironment),
                core::ptr::addr_of_mut!(UnicodeEnvironment),
                (TRUE) as BOOLEAN,
            );
            if !NT_SUCCESS!(Status) {
                DPRINT1!("ERR: no mem(guess)\n");
                break 'fail;
            }

            ASSERT!(!AnsiEnvironment.Buffer.is_null());

            Source = AnsiEnvironment.Buffer;
            while *Source != 0 {
                /* Save a pointer to this token */
                *ArgumentList = Source;
                ArgumentList = ArgumentList.add(1);

                /* Keep looking for another variable */
                loop {
                    let c = *Source;
                    Source = Source.add(1);
                    if c == 0 {
                        break;
                    }
                }
            }

            /* Null terminate the list again */
            *ArgumentList = core::ptr::null_mut();
            ArgumentList = ArgumentList.add(1);
        }

        /* Breakpoint if we were requested to do so */
        if (*ProcessParameters).DebugFlags != 0 {
            DbgBreakPoint();
        }

        /* Call the Main Function */
        Status = _main(
            argc,
            argv,
            envp,
            ((*ProcessParameters).DebugFlags) as i32,
        );
    }

    /* We're done here */
    NtTerminateProcess(NtCurrentProcess(), Status);
}
