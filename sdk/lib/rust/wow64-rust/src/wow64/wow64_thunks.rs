//! wow64_thunks.c in Rust — the NT syscall dispatcher over the generated thunk table
//! (`gen/wow64_thunks.rs`, from ntdll.spec types in sysfuncs.h service order).
//! `Wow64SystemServiceEx` (wow64.rs) calls `Wow64DispatchSystemService` for any service
//! number that is not one of its own magic ones.

use crate::prelude::*;
use crate::wow64::gen::wow64_thunks::{Wow64ThunkCount, Wow64ThunkTable};
use crate::wow64::wow64_manual::Wow64ManualDispatch;
use crate::wow64::wow64_win32k::Wow64DispatchWin32kService;

/// Serial-noisy per-syscall trace for bring-up (each guest syscall costs a DbgPrint
/// line, callbacks too). OFF since 2026-09-26: every amd64 boot runs user32_winetest as
/// a WoW64 guest, so the trace grew the serial to ~40MB per sweep and slowed the TCG
/// guest until unrelated apps missed their window timeouts. Set to 1 to debug a thunk.
#[no_mangle]
pub static mut Wow64TraceSyscalls: ULONG = 0;

#[no_mangle]
pub unsafe extern "C" fn Wow64DispatchSystemService(ServiceNumber: ULONG, Arguments: PULONG) -> NTSTATUS {
    if (ServiceNumber & 0xF000) == 0x1000 {
        return Wow64DispatchWin32kService(ServiceNumber, Arguments);
    }

    if ServiceNumber >= Wow64ThunkCount {
        DbgPrint(
            c"[WOW64] service 0x%lx OUT OF RANGE (table has %u) args=%p\n".as_ptr(),
            ServiceNumber,
            Wow64ThunkCount,
            Arguments,
        );
        return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
    }

    let Entry = &Wow64ThunkTable[ServiceNumber as usize];
    let Some(Thunk) = Entry.Thunk else {
        let Status = Wow64ManualDispatch(Entry.Name, Arguments);
        if WOW64_IS_UNIMPL(Status) {
            DbgPrint(
                c"[WOW64] %s (0x%lx) NEEDS MANUAL MARSHALLER — not yet implemented, failing loud\n".as_ptr(),
                Entry.Name,
                ServiceNumber,
            );
            return WOW64_STATUS_INVALID_SYSTEM_SERVICE;
        }
        if Wow64TraceSyscalls != 0 {
            DbgPrint(c"[WOW64] %s(0x%lx) [manual] => %08lx\n".as_ptr(), Entry.Name, ServiceNumber, Status);
        }
        return Status;
    };

    let Status = Thunk(Arguments);
    if Wow64TraceSyscalls != 0 {
        DbgPrint(c"[WOW64] %s(0x%lx) => %08lx\n".as_ptr(), Entry.Name, ServiceNumber, Status);
    }
    Status
}
