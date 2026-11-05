//! wow64_win32k.c in Rust — the win32k (NtGdi*/NtUser*) dispatcher over the generated
//! table (`gen/wow64_win32k_thunks.rs`). Guest EAX carries 0x1000 + the w32ksvc32.h
//! index; the host calls resolve through the native win32u.dll importlib.

use crate::prelude::*;
use crate::wow64::gen::wow64_win32k_thunks::{Wow64Win32kThunkCount, Wow64Win32kThunkTable};
use crate::wow64::wow64::{Wow64LastErrorToGuestIfChanged, Wow64LastErrorToHost};
use crate::wow64::wow64_manual::Wow64ManualDispatch;
use crate::wow64::wow64_thunks::Wow64TraceSyscalls;

#[no_mangle]
pub unsafe extern "C" fn Wow64DispatchWin32kService(ServiceNumber: ULONG, Arguments: PULONG) -> NTSTATUS {
    let Index = ServiceNumber.wrapping_sub(0x1000);

    if Index >= Wow64Win32kThunkCount {
        DbgPrint(
            c"[WOW64] win32k service 0x%lx OUT OF RANGE (table has %u) args=%p\n".as_ptr(),
            ServiceNumber,
            Wow64Win32kThunkCount,
            Arguments,
        );
        return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
    }

    let Entry = &Wow64Win32kThunkTable[Index as usize];
    // win32k reports failure by setting last-error in the 64-bit TEB, which a 32-bit
    // guest cannot see; every win32k syscall passes through here, so this is the one
    // place that bridges the syscall direction (wow64_callback bridges the other).
    let LastError = Wow64LastErrorToHost();
    let mut Status: NTSTATUS;

    match Entry.Thunk {
        None => {
            Status = Wow64ManualDispatch(Entry.Name, Arguments);
            if WOW64_IS_UNIMPL(Status) {
                DbgPrint(
                    c"[WOW64] %s (0x%lx) NEEDS MANUAL MARSHALLER (win32k) — failing loud\n".as_ptr(),
                    Entry.Name,
                    ServiceNumber,
                );
                Status = WOW64_STATUS_INVALID_SYSTEM_SERVICE;
            } else if Wow64TraceSyscalls != 0 {
                DbgPrint(c"[WOW64] %s(0x%lx) [manual] => %08lx\n".as_ptr(), Entry.Name, ServiceNumber, Status);
            }
        }
        Some(Thunk) => {
            // On-entry trace, so a thunk that FAULTS the kernel still names itself on
            // serial; the "=>" line below never prints in that case.
            if Wow64TraceSyscalls != 0 {
                DbgPrint(
                    c"[WOW64>] %s(0x%lx) enter arg0=%08lx arg1=%08lx arg2=%08lx\n".as_ptr(),
                    Entry.Name,
                    ServiceNumber,
                    *Arguments.add(0),
                    *Arguments.add(1),
                    *Arguments.add(2),
                );
            }
            Status = Thunk(Arguments);
            if Wow64TraceSyscalls != 0 {
                DbgPrint(c"[WOW64] %s(0x%lx) => %08lx\n".as_ptr(), Entry.Name, ServiceNumber, Status);
            }
        }
    }

    Wow64LastErrorToGuestIfChanged(LastError);
    Status
}
