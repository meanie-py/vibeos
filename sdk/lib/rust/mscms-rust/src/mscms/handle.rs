use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn cmsCloseProfile(hProfile: *mut core::ffi::c_void);
    fn cmsDeleteTransform(htransform: *mut core::ffi::c_void);
}
unsafe extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree( hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, ) -> i32;
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapReAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, dwBytes: usize) -> *mut core::ffi::c_void;
    fn SetFilePointer(hFile: HANDLE, lDistanceToMove: i32, lpDistanceToMoveHigh: *mut i32, dwMoveMethod: u32) -> u32;
    fn WriteFile(hFile: HANDLE, lpBuffer: *const core::ffi::c_void, nNumberOfBytesToWrite: u32, lpNumberOfBytesWritten: *mut u32, lpOverlapped: *mut core::ffi::c_void) -> i32;
    fn CloseHandle(hObject: HANDLE) -> i32;
}

// ---- file-scope preamble (translated) ----
static MscmsHandleCsFileStr: [u8; 28] = *b"handle.c: mscms_handle_cs\x00";

static mut mscms_handle_cs_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: &raw const mscms_handle_cs as *mut _,
    ProcessLocksList: LIST_ENTRY {
        Flink: &raw const mscms_handle_cs_debug.ProcessLocksList as *mut _,
        Blink: &raw const mscms_handle_cs_debug.ProcessLocksList as *mut _,
    },
    EntryCount: 0,
    ContentionCount: 0,
    Spare: [ &MscmsHandleCsFileStr as *const _ as DWORD_PTR, 0 ],
};

static mut mscms_handle_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: &raw const mscms_handle_cs_debug as *mut _,
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: 0,
    LockSemaphore: 0,
    SpinCount: 0,
};

static mut profiletable: *mut profile = core::ptr::null_mut();
static mut transformtable: *mut transform = core::ptr::null_mut();
static mut num_profile_handles: u32 = 0;
static mut num_transform_handles: u32 = 0;

const HEAP_ZERO_MEMORY: u32 = 0x00000008;
#[no_mangle]
pub unsafe extern "C" fn free_handle_tables() {
    HeapFree(GetProcessHeap(), 0, profiletable as *mut core::ffi::c_void);
    profiletable = core::ptr::null_mut();
    num_profile_handles = 0u32;

    HeapFree(GetProcessHeap(), 0, transformtable as *mut core::ffi::c_void);
    transformtable = core::ptr::null_mut();
    num_transform_handles = 0u32;

    DeleteCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
}
#[no_mangle]
pub unsafe extern "C" fn grab_profile(handle: HPROFILE) -> *mut profile {
    let index: usize;
    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    index = (handle as usize) - 1;
    if index > num_profile_handles {
        LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
        return core::ptr::null_mut();
    }
    return profiletable.add(index);
}
#[no_mangle]
pub unsafe extern "C" fn release_profile(profile: *mut profile) {
    LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
}
#[no_mangle]
pub unsafe extern "C" fn grab_transform(handle: HTRANSFORM) -> *mut transform {
    let index: usize;

    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));

    index = (handle as usize) - 1;
    if index > num_transform_handles as usize {
        LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
        return core::ptr::null_mut();
    }
    transformtable.add(index)
}

#[no_mangle]
pub unsafe extern "C" fn release_transform(transform: *mut Transform) {
    LeaveCriticalSection(&mut mscms_handle_cs);
}

unsafe fn alloc_profile_handle() -> isize {
    let mut index: usize;
    let p: *mut profile;
    let mut count: usize = 128;

    index = 0;
    while index < unsafe { num_profile_handles } {
        if (*unsafe { profiletable.add(index) }).data.is_null() {
            return (index + 1) as isize;
        }
        index += 1;
    }
    if unsafe { profiletable }.is_null() {
        let heap = unsafe { GetProcessHeap() };
        p = unsafe { HeapAlloc(heap, HEAP_ZERO_MEMORY, count.wrapping_mul(core::mem::size_of::<profile>())) as *mut profile };
    } else {
        count = unsafe { num_profile_handles }.wrapping_mul(2);
        let heap = unsafe { GetProcessHeap() };
        p = unsafe { HeapReAlloc(heap, HEAP_ZERO_MEMORY, profiletable as *mut core::ffi::c_void, count.wrapping_mul(core::mem::size_of::<profile>())) as *mut profile };
    }
    if p.is_null() {
        return 0;
    }
    unsafe {
        profiletable = p;
        num_profile_handles = count;
    }
    return (index + 1) as isize;
}
#[no_mangle]
pub unsafe extern "C" fn create_profile(profile: *mut profile) -> HPROFILE {
    let mut handle: HPROFILE = core::ptr::null_mut();
    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    handle = alloc_profile_handle();
    if !handle.is_null() {
        let index: DWORD_PTR = handle as DWORD_PTR - 1;
        core::ptr::write(profiletable.add(index), core::ptr::read(profile));
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    handle
}
#[no_mangle]
pub unsafe extern "C" fn close_profile(handle: HPROFILE) -> i32 {
    const PROFILE_READWRITE: u32 = 2;
    const FILE_BEGIN: u32 = 0;

    let index: usize;
    let profile: *mut profile;

    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));

    index = (handle as usize) - 1;
    if index > num_profile_handles as usize {
        LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
        return 0;
    }
    profile = profiletable.add(index);

    if (*profile).file != INVALID_HANDLE_VALUE {
        if ((*profile).access & PROFILE_READWRITE) != 0 {
            let mut written: u32 = 0;
            if SetFilePointer((*profile).file, 0, core::ptr::null_mut(), FILE_BEGIN) != 0
                || WriteFile((*profile).file, (*profile).data, (*profile).size, core::ptr::addr_of_mut!(written), core::ptr::null_mut()) == 0
                || written != (*profile).size
            {
                ERR!(b"Unable to write color profile\n\0".as_ptr() as *const i8);
            }
        }
        CloseHandle((*profile).file);
    }
    cmsCloseProfile((*profile).cmsprofile);
    HeapFree(GetProcessHeap(), 0, (*profile).data);

    core::ptr::write_bytes(profile as *mut u8, 0, core::mem::size_of::<profile>());

    LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    return 1;
}

unsafe fn alloc_transform_handle() -> usize {
    let mut index: usize = 0;
    let p: *mut transform;
    let mut count: u32 = 128;

    while index < num_transform_handles as usize {
        if (*transformtable.add(index)).cmstransform.is_null() {
            return (index + 1) as usize;
        }
        index += 1;
    }

    if transformtable.is_null() {
        let size = core::mem::size_of::<transform>();
        let heap = GetProcessHeap();
        p = HeapAlloc(heap, HEAP_ZERO_MEMORY, (count as usize).wrapping_mul(size)) as *mut transform;
    } else {
        count = num_transform_handles * 2;
        let size = core::mem::size_of::<transform>();
        let heap = GetProcessHeap();
        p = HeapReAlloc(heap, HEAP_ZERO_MEMORY, transformtable as *mut core::ffi::c_void, (count as usize).wrapping_mul(size)) as *mut transform;
    }
    if p.is_null() {
        return core::ptr::null_mut::<core::ffi::c_void>() as usize;
    }

    transformtable = p;
    num_transform_handles = count;

    (index + 1) as usize
}
#[no_mangle]
pub unsafe extern "C" fn create_transform(transform: *mut crate::Transform) -> HTRANSFORM {
    let handle: HTRANSFORM;
    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    handle = alloc_transform_handle();
    if handle != 0 {
        let index: usize = handle as usize - 1;
        *transformtable.add(index) = *transform;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    handle
}
#[no_mangle]
pub unsafe extern "C" fn close_transform(handle: HTRANSFORM) -> BOOL {
    let index: DWORD_PTR;
    let transform: *mut Transform;

    EnterCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));

    index = (handle as DWORD_PTR).wrapping_sub(1);
    if index > num_transform_handles as DWORD_PTR {
        LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
        return 0;
    }
    transform = transformtable.wrapping_add(index);

    cmsDeleteTransform((*transform).cmstransform as *mut core::ffi::c_void);
    core::ptr::write_bytes(transform as *mut u8, 0u8, core::mem::size_of::<Transform>());

    LeaveCriticalSection(core::ptr::addr_of_mut!(mscms_handle_cs));
    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * MSCMS - Color Management System for Wine
//  *
//  * Copyright 2004, 2005, 2008 Hans Leidekker
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
// #include "config.h"
// #include "wine/debug.h"
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// #ifdef HAVE_LCMS2
//
// static CRITICAL_SECTION mscms_handle_cs;
// static CRITICAL_SECTION_DEBUG mscms_handle_cs_debug =
// {
//     0, 0, &mscms_handle_cs,
//     { &mscms_handle_cs_debug.ProcessLocksList,
//       &mscms_handle_cs_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": mscms_handle_cs") }
// };
// static CRITICAL_SECTION mscms_handle_cs = { &mscms_handle_cs_debug, -1, 0, 0, 0, 0 };
//
// static struct profile *profiletable;
// static struct transform *transformtable;
//
// static unsigned int num_profile_handles;
// static unsigned int num_transform_handles;
//
// WINE_DEFAULT_DEBUG_CHANNEL(mscms);
//
// void free_handle_tables( void )
// {
//     HeapFree( GetProcessHeap(), 0, profiletable );
//     profiletable = NULL;
//     num_profile_handles = 0;
//
//     HeapFree( GetProcessHeap(), 0, transformtable );
//     transformtable = NULL;
//     num_transform_handles = 0;
//
//     DeleteCriticalSection( &mscms_handle_cs );
// }
//
// struct profile *grab_profile( HPROFILE handle )
// {
//     DWORD_PTR index;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     index = (DWORD_PTR)handle - 1;
//     if (index > num_profile_handles)
//     {
//         LeaveCriticalSection( &mscms_handle_cs );
//         return NULL;
//     }
//     return &profiletable[index];
// }
//
// void release_profile( struct profile *profile )
// {
//     LeaveCriticalSection( &mscms_handle_cs );
// }
//
// struct transform *grab_transform( HTRANSFORM handle )
// {
//     DWORD_PTR index;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     index = (DWORD_PTR)handle - 1;
//     if (index > num_transform_handles)
//     {
//         LeaveCriticalSection( &mscms_handle_cs );
//         return NULL;
//     }
//     return &transformtable[index];
// }
//
// void release_transform( struct transform *transform )
// {
//     LeaveCriticalSection( &mscms_handle_cs );
// }
//
// static HPROFILE alloc_profile_handle( void )
// {
//     DWORD_PTR index;
//     struct profile *p;
//     unsigned int count = 128;
//
//     for (index = 0; index < num_profile_handles; index++)
//     {
//         if (!profiletable[index].data) return (HPROFILE)(index + 1);
//     }
//     if (!profiletable)
//     {
//         p = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, count * sizeof(struct profile) );
//     }
//     else
//     {
//         count = num_profile_handles * 2;
//         p = HeapReAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, profiletable, count * sizeof(struct profile) );
//     }
//     if (!p) return NULL;
//
//     profiletable = p;
//     num_profile_handles = count;
//
//     return (HPROFILE)(index + 1);
// }
//
// HPROFILE create_profile( struct profile *profile )
// {
//     HPROFILE handle;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     if ((handle = alloc_profile_handle()))
//     {
//         DWORD_PTR index = (DWORD_PTR)handle - 1;
//         profiletable[index] = *profile;
//     }
//     LeaveCriticalSection( &mscms_handle_cs );
//     return handle;
// }
//
// BOOL close_profile( HPROFILE handle )
// {
//     DWORD_PTR index;
//     struct profile *profile;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     index = (DWORD_PTR)handle - 1;
//     if (index > num_profile_handles)
//     {
//         LeaveCriticalSection( &mscms_handle_cs );
//         return FALSE;
//     }
//     profile = &profiletable[index];
//
//     if (profile->file != INVALID_HANDLE_VALUE)
//     {
//         if (profile->access & PROFILE_READWRITE)
//         {
//             DWORD written;
//
//             if (SetFilePointer( profile->file, 0, NULL, FILE_BEGIN ) ||
//                 !WriteFile( profile->file, profile->data, profile->size, &written, NULL ) ||
//                 written != profile->size)
//             {
//                 ERR( "Unable to write color profile\n" );
//             }
//         }
//         CloseHandle( profile->file );
//     }
//     cmsCloseProfile( profile->cmsprofile );
//     HeapFree( GetProcessHeap(), 0, profile->data );
//
//     memset( profile, 0, sizeof(struct profile) );
//
//     LeaveCriticalSection( &mscms_handle_cs );
//     return TRUE;
// }
//
// static HTRANSFORM alloc_transform_handle( void )
// {
//     DWORD_PTR index;
//     struct transform *p;
//     unsigned int count = 128;
//
//     for (index = 0; index < num_transform_handles; index++)
//     {
//         if (!transformtable[index].cmstransform) return (HTRANSFORM)(index + 1);
//     }
//     if (!transformtable)
//     {
//         p = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, count * sizeof(struct transform) );
//     }
//     else
//     {
//         count = num_transform_handles * 2;
//         p = HeapReAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, transformtable, count * sizeof(struct transform) );
//     }
//     if (!p) return NULL;
//
//     transformtable = p;
//     num_transform_handles = count;
//
//     return (HTRANSFORM)(index + 1);
// }
//
// HTRANSFORM create_transform( struct transform *transform )
// {
//     HTRANSFORM handle;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     if ((handle = alloc_transform_handle()))
//     {
//         DWORD_PTR index = (DWORD_PTR)handle - 1;
//         transformtable[index] = *transform;
//     }
//     LeaveCriticalSection( &mscms_handle_cs );
//     return handle;
// }
//
// BOOL close_transform( HTRANSFORM handle )
// {
//     DWORD_PTR index;
//     struct transform *transform;
//
//     EnterCriticalSection( &mscms_handle_cs );
//
//     index = (DWORD_PTR)handle - 1;
//     if (index > num_transform_handles)
//     {
//         LeaveCriticalSection( &mscms_handle_cs );
//         return FALSE;
//     }
//     transform = &transformtable[index];
//
//     cmsDeleteTransform( transform->cmstransform );
//     memset( transform, 0, sizeof(struct transform) );
//
//     LeaveCriticalSection( &mscms_handle_cs );
//     return TRUE;
// }
//
// #endif /* HAVE_LCMS2 */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
