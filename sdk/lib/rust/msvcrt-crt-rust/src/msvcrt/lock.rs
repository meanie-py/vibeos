// rustconv-assembly: whole-tu
use crate::prelude::*;
use msvcrt_sys::*;
use core::ffi::c_void;

// Explicit, so it shadows both globs: `prelude` and `msvcrt_sys` each define `FALSE` and
// the bare name is otherwise E0659-ambiguous. minwindef.h's is the one lock.c means —
// every use here is compared against or assigned to a `BOOL` field.
use msvcrt_sys::{FALSE, TRUE};

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct LOCKTABLEENTRY {
    pub bInit: BOOL,
    pub crit: CRITICAL_SECTION,
}

pub type PLOCKTABLEENTRY = *mut LOCKTABLEENTRY;

/// C's `static LOCKTABLEENTRY lock_table[_TOTAL_LOCKS]` — 48 slots, zero-initialised in
/// `.bss`. The repeat operand is an inline `const` block because `CRITICAL_SECTION` is not
/// `Copy`; `[expr; N]` would demand it, `[const { expr }; N]` evaluates the initialiser
/// once per slot at compile time instead.
static mut lock_table: [LOCKTABLEENTRY; _TOTAL_LOCKS as usize] = [const {
    LOCKTABLEENTRY {
        bInit: 0,
        crit: CRITICAL_SECTION {
            DebugInfo: core::ptr::null_mut(),
            LockCount: 0,
            RecursionCount: 0,
            OwningThread: core::ptr::null_mut(),
            LockSemaphore: core::ptr::null_mut(),
            SpinCount: 0,
        },
    }
}; _TOTAL_LOCKS as usize];
unsafe extern "C" fn msvcrt_mlock_set_entry_initialized(locknum: i32, initialized: BOOL) {
    lock_table[locknum as usize].bInit = initialized;
}
unsafe extern "C" fn msvcrt_initialize_mlock(locknum: i32) {
    InitializeCriticalSectionEx(
        core::ptr::addr_of_mut!((*(&raw mut lock_table).cast::<LOCKTABLEENTRY>().offset(locknum as isize)).crit),
        0,
        RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO,
    );
    // `Spare` lives in RTL_CRITICAL_SECTION_DEBUG's anonymous union, alongside
    // `WineDebugString` and a Flags/backtrace struct — same storage, three spellings.
    // C: `(DWORD_PTR)(__FILE__ ": LOCKTABLEENTRY.crit")`. `file!()` is the direct analogue
    // — it names THIS source rather than a hardcoded "lock.c", which is the point of the
    // tag: a debugger printing Spare[0] should land on the file that owns the section.
    (*(*lock_table[locknum as usize].crit.DebugInfo).__bindgen_anon_1.Spare.as_mut_ptr()) =
        concat!(file!(), ": LOCKTABLEENTRY.crit\0").as_ptr() as DWORD_PTR;
    msvcrt_mlock_set_entry_initialized(locknum, TRUE as BOOL);
}
unsafe extern "C" fn msvcrt_uninitialize_mlock(locknum: i32) {
    (*lock_table[locknum as usize].crit.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
    DeleteCriticalSection(&mut lock_table[locknum as usize].crit);
    msvcrt_mlock_set_entry_initialized(locknum, FALSE as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_mt_locks() {
    let mut i: i32;

    i = 0;
    while i < _TOTAL_LOCKS as i32 {
        msvcrt_mlock_set_entry_initialized(i, FALSE as BOOL);
        i += 1;
    }

    msvcrt_initialize_mlock(_LOCKTAB_LOCK as i32);
}

#[no_mangle]
pub unsafe extern "C" fn _lock(locknum: i32) {
    if lock_table[locknum as usize].bInit == FALSE as BOOL {
        if lock_table[_LOCKTAB_LOCK as usize].bInit == FALSE as BOOL {
            msvcrt_initialize_mlock(_LOCKTAB_LOCK as i32);
        }

        _lock(_LOCKTAB_LOCK as i32);

        if lock_table[locknum as usize].bInit == FALSE as BOOL {
            msvcrt_initialize_mlock(locknum);
        }

        _unlock(_LOCKTAB_LOCK as i32);
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(lock_table[locknum as usize].crit));
}

#[no_mangle]
pub unsafe extern "C" fn _unlock(locknum: i32) {
    LeaveCriticalSection(&mut lock_table[locknum as usize].crit);
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_locks() {
    for i in 0.._TOTAL_LOCKS as i32 {
        if lock_table[i as usize].bInit != 0 {
            msvcrt_uninitialize_mlock(i);
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright (c) 2002, TransGaming Technologies Inc.
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdarg.h>
// #include <stdbool.h>
//
// #include "wine/debug.h"
// #include "windef.h"
// #include "winbase.h"
// #include "winternl.h"
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "cxx.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// typedef struct
// {
//   BOOL             bInit;
//   CRITICAL_SECTION crit;
// } LOCKTABLEENTRY;
//
// static LOCKTABLEENTRY lock_table[ _TOTAL_LOCKS ];
//
// static inline void msvcrt_mlock_set_entry_initialized( int locknum, BOOL initialized )
// {
//   lock_table[ locknum ].bInit = initialized;
// }
//
// static inline void msvcrt_initialize_mlock( int locknum )
// {
//   InitializeCriticalSectionEx( &(lock_table[ locknum ].crit), 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO );
//   lock_table[ locknum ].crit.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": LOCKTABLEENTRY.crit");
//   msvcrt_mlock_set_entry_initialized( locknum, TRUE );
// }
//
// static inline void msvcrt_uninitialize_mlock( int locknum )
// {
//   lock_table[ locknum ].crit.DebugInfo->Spare[0] = 0;
//   DeleteCriticalSection( &(lock_table[ locknum ].crit) );
//   msvcrt_mlock_set_entry_initialized( locknum, FALSE );
// }
//
// /**********************************************************************
//  *     msvcrt_init_mt_locks (internal)
//  *
//  * Initialize the table lock. All other locks will be initialized
//  * upon first use.
//  *
//  */
// void msvcrt_init_mt_locks(void)
// {
//   int i;
//
//   TRACE( "initializing mtlocks\n" );
//
//   /* Initialize the table */
//   for( i=0; i < _TOTAL_LOCKS; i++ )
//   {
//     msvcrt_mlock_set_entry_initialized( i, FALSE );
//   }
//
//   /* Initialize our lock table lock */
//   msvcrt_initialize_mlock( _LOCKTAB_LOCK );
// }
//
// /**********************************************************************
//  *              _lock (MSVCRT.@)
//  */
// void CDECL _lock( int locknum )
// {
//   TRACE( "(%d)\n", locknum );
//
//   /* If the lock doesn't exist yet, create it */
//   if( lock_table[ locknum ].bInit == FALSE )
//   {
//     /* Ensure _LOCKTAB_LOCK itself is initialized (may not be if DllMain
//      * was never called, e.g. DLL mapped but not initialized). Without
//      * this, _lock(_LOCKTAB_LOCK) recurses infinitely. */
//     if( lock_table[ _LOCKTAB_LOCK ].bInit == FALSE )
//       msvcrt_initialize_mlock( _LOCKTAB_LOCK );
//
//     /* Lock while we're changing the lock table */
//     _lock( _LOCKTAB_LOCK );
//
//     /* Check again if we've got a bit of a race on lock creation */
//     if( lock_table[ locknum ].bInit == FALSE )
//     {
//       TRACE( ": creating lock #%d\n", locknum );
//       msvcrt_initialize_mlock( locknum );
//     }
//
//     /* Unlock ourselves */
//     _unlock( _LOCKTAB_LOCK );
//   }
//
//   EnterCriticalSection( &(lock_table[ locknum ].crit) );
// }
//
// /**********************************************************************
//  *              _unlock (MSVCRT.@)
//  *
//  * NOTE: There is no error detection to make sure the lock exists and is acquired.
//  */
// void CDECL _unlock( int locknum )
// {
//   TRACE( "(%d)\n", locknum );
//
//   LeaveCriticalSection( &(lock_table[ locknum ].crit) );
// }
//
//
// /**********************************************************************
//  *     msvcrt_free_locks (internal)
//  *
//  * Uninitialize all mt locks. Assume that neither _lock or _unlock will
//  * be called once we're calling this routine (ie _LOCKTAB_LOCK can be deleted)
//  *
//  */
// void msvcrt_free_locks(void)
// {
//   int i;
//
//   TRACE( ": uninitializing all mtlocks\n" );
//
//   /* Uninitialize the table */
//   for( i=0; i < _TOTAL_LOCKS; i++ )
//   {
//     if( lock_table[ i ].bInit )
//     {
//       msvcrt_uninitialize_mlock( i );
//     }
//   }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
