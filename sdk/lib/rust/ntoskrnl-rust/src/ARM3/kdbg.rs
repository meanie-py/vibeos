use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::RtlZeroMemory;

use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, PUNICODE_STRING,
    IoGetCurrentIrpStackLocation, MmIsAddressValid,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// TAG_IRP = ' prI' (little-endian): bytes 'I'=0x49, 'r'=0x72, 'p'=0x70, ' '=0x20
const TAG_IRP: u32 = 0x20707249;

struct IRP_FIND_CTXT {
    RestartAddress: ULONG_PTR,
    SData: ULONG_PTR,
    Criteria: ULONG,
}

extern "C" {
    static mut MmNonPagedPoolEnd0: PVOID;
    static PoolBigPageTableSize: SIZE_T;
    static PoolBigPageTable: PPOOL_TRACKER_BIG_PAGES;
}

const POOL_BLOCK_SIZE: usize = 8;

unsafe fn POOL_ENTRY(x: PVOID) -> PPOOL_HEADER {
    ((x as ULONG_PTR) - core::mem::size_of::<POOL_HEADER>()) as PPOOL_HEADER
}

unsafe fn POOL_FREE_BLOCK(x: PVOID) -> PVOID {
    ((x as ULONG_PTR) + core::mem::size_of::<POOL_HEADER>()) as PVOID
}

unsafe fn POOL_BLOCK(x: PPOOL_HEADER, i: USHORT) -> PPOOL_HEADER {
    ((x as ULONG_PTR) + ((i as usize) * POOL_BLOCK_SIZE)) as PPOOL_HEADER
}

extern "C" {
    fn MiDumpPoolConsumers(CalledFromDbg: BOOLEAN, Tag: ULONG, Mask: ULONG, Flags: ULONG);
}

// KDBG functions — `extern "system"` maps to stdcall on i386 (with the
// expected `@N` decoration) and cdecl on amd64/arm64. KdbpGetHexNumber
// is NTAPI in `kdb.h`; RtlFindSetBits is NTAPI in bindgen.
extern "system" {
    fn KdbpGetHexNumber(Arg: PCHAR, Number: *mut ULONG_PTR) -> BOOLEAN;
    fn RtlFindSetBits(
        BitMapHeader: PRTL_BITMAP,
        NumberToFind: ULONG,
        HintIndex: ULONG,
    ) -> ULONG;
}
// KdbpPrint is variadic (cdecl even on i386), strtoul/strcmp/strlen are CRT (cdecl)
extern "C" {
    fn strtoul(nptr: PCHAR, endptr: *mut PCHAR, base: i32) -> ULONG;
    fn strcmp(s1: PCHAR, s2: PCHAR) -> i32;
    fn strlen(s: PCHAR) -> SIZE_T;
}

    #[no_mangle]
    pub unsafe extern "C" fn ExpKdbgExtPool(
        Argc: ULONG,
        Argv: *mut PCHAR,
    ) -> BOOLEAN {
    let Argv = opaque_ptr(Argv);
        let mut Address: ULONG_PTR = 0;
        let mut Flags: ULONG_PTR = 0;
        let mut PoolPage: PVOID;
        let mut Entry: PPOOL_HEADER;
        let mut ThisOne: BOOLEAN;
        let mut Data: *mut ULONG;

        if Argc > 1 {
            if KdbpGetHexNumber(*Argv.add(1), &mut Address) == 0 {
                crate::kdbp_print!(b"Invalid parameter: %s\n\0", *Argv.add(1));
                return 1u8;
            }
        }

        if Argc > 2 {
            if KdbpGetHexNumber(*Argv.add(2), &mut Flags) == 0 {
                crate::kdbp_print!(b"Invalid parameter: %s\n\0", *Argv.add(2));
                return 1u8;
            }
        }

        if Address != 0 {
            PoolPage = PAGE_ALIGN(Address as usize) as PVOID;
        } else {
            crate::kdbp_print!(b"Heap is unimplemented\n\0");
            return 1u8;
        }

        if MmIsAddressValid(PoolPage) == 0 {
            crate::kdbp_print!(b"Address not accessible!\n\0");
            return 1u8;
        }

        if (Address >= (MmPagedPoolStart as ULONG_PTR)) && (Address <= (MmPagedPoolEnd as ULONG_PTR)) {
            crate::kdbp_print!(b"Allocation is from PagedPool region\n\0");
        } else if (Address >= (MmNonPagedPoolStart as ULONG_PTR)) && (Address <= (MmNonPagedPoolEnd as ULONG_PTR)) {
            crate::kdbp_print!(b"Allocation is from NonPagedPool region\n\0");
        } else {
            crate::kdbp_print!(b"Address 0x%p is not within any pool!\n\0", Address as PVOID);
            return 1u8;
        }

        Entry = PoolPage as PPOOL_HEADER;
        loop {
            ThisOne = ((Address >= (Entry as ULONG_PTR)) &&
                       (Address < ((Entry as ULONG_PTR) + ((*Entry).BlockSize() as ULONG_PTR * POOL_BLOCK_SIZE)))) as BOOLEAN;

            if (Flags & 1) == 0 || ThisOne != 0 {
                crate::kdbp_print!(b"%c%p size: %4d previous size: %4d  %s  %.4s\n\0",
                    if ThisOne != 0 { b'*' as i32 } else { b' ' as i32 },
                    Entry,
                    (*Entry).BlockSize() as i32,
                    (*Entry).PreviousSize() as i32,
                    if (Flags & 0x80000000) != 0 {
                        b"\0".as_ptr() as PCHAR
                    } else if (*Entry).PoolType() != 0 {
                        b"(Allocated)\0".as_ptr() as PCHAR
                    } else {
                        b"(Free)     \0".as_ptr() as PCHAR
                    },
                    if (Flags & 0x80000000) != 0 {
                        b"\0".as_ptr() as PCHAR
                    } else {
                        core::ptr::addr_of!((*Entry).PoolTag) as PCHAR
                    },
                );
            }

            if (Flags & 1) != 0 {
                Data = (Entry.add(1)) as *mut ULONG;
                crate::kdbp_print!(b"    %p  %08lx %08lx %08lx %08lx\n    %p  %08lx %08lx %08lx %08lx\n\0",
                    Data,
                    *Data.add(0), *Data.add(1), *Data.add(2), *Data.add(3),
                    Data.add(4),
                    *Data.add(4), *Data.add(5), *Data.add(6), *Data.add(7),
                );
            }

            Entry = POOL_BLOCK(Entry, (*Entry).BlockSize());

            if (*Entry).BlockSize() == 0 || (Entry as ULONG_PTR) >= ((PoolPage as ULONG_PTR) + PAGE_SIZE) {
                break;
            }
        }

        1u8
    }

    unsafe fn ExpKdbgExtPoolUsedGetTag(Arg: PCHAR, Tag: *mut ULONG, Mask: *mut ULONG) {
        let mut Tmp: [i8; 4] = [0; 4];
        let mut Len: SIZE_T;
        let mut i: USHORT;

        Len = strlen(Arg);
        if Len > 4 {
            Len = 4;
        }

        for i in 0..Len as USHORT {
            Tmp[i as usize] = *Arg.add(i as usize);
            if Tmp[i as usize] != b'?' as i8 {
                *Mask |= (0xFF << (i * 8));
            }
        }

        *Tag = (Tmp.as_ptr() as *const ULONG).read_unaligned();
    }

    #[no_mangle]
    pub unsafe extern "C" fn ExpKdbgExtPoolUsed(
        Argc: ULONG,
        Argv: *mut PCHAR,
    ) -> BOOLEAN {
    let Argv = opaque_ptr(Argv);
        let mut Tag: ULONG = 0;
        let mut Mask: ULONG = 0;
        let mut Flags: ULONG_PTR = 0;

        if Argc > 1 {
            if Argc > 2 {
                ExpKdbgExtPoolUsedGetTag(*Argv.add(2), &mut Tag, &mut Mask);
                if KdbpGetHexNumber(*Argv.add(1), &mut Flags) == 0 {
                    crate::kdbp_print!(b"Invalid parameter: %s\n\0", *Argv.add(1));
                }
            } else {
                if strlen(*Argv.add(1)) == 1 ||
                   (strlen(*Argv.add(1)) == 3 && *(*Argv.add(1)) == b'0' as i8 &&
                    (*(*Argv.add(1)).add(1) == b'x' as i8 || *(*Argv.add(1)).add(1) == b'X' as i8))
                {
                    if KdbpGetHexNumber(*Argv.add(1), &mut Flags) == 0 {
                        ExpKdbgExtPoolUsedGetTag(*Argv.add(1), &mut Tag, &mut Mask);
                    }
                } else {
                    ExpKdbgExtPoolUsedGetTag(*Argv.add(1), &mut Tag, &mut Mask);
                }
            }
        }

        MiDumpPoolConsumers(1u8, Tag, Mask, Flags as ULONG);

        1u8
    }

    unsafe fn ExpKdbgExtPoolFindLargePool(
        Tag: ULONG,
        Mask: ULONG,
        FoundCallback: Option<unsafe extern "system" fn(PPOOL_TRACKER_BIG_PAGES, PVOID)>,
        CallbackContext: PVOID,
    ) {
        let mut i: ULONG;

        crate::kdbp_print!(b"Scanning large pool allocation table for Tag: %.4s (%p : %p)\n\0",
            core::ptr::addr_of!(Tag) as PCHAR,
            PoolBigPageTable,
            PoolBigPageTable.add(PoolBigPageTableSize - 1),
        );

        for i in 0..PoolBigPageTableSize as ULONG {
            if ((*PoolBigPageTable.add(i as usize)).Va as ULONG_PTR) & POOL_BIG_TABLE_ENTRY_FREE as ULONG_PTR != 0 {
                continue;
            }

            if ((*PoolBigPageTable.add(i as usize)).Key & Mask) == (Tag & Mask) {
                if let Some(callback) = FoundCallback {
                    callback(PoolBigPageTable.add(i as usize), CallbackContext);
                } else {
                    crate::kdbp_print!(b"%p: tag %.4s, size: %I64x\n\0",
                        (*PoolBigPageTable.add(i as usize)).Va,
                        core::ptr::addr_of!((*PoolBigPageTable.add(i as usize)).Key) as PCHAR,
                        ((*PoolBigPageTable.add(i as usize)).NumberOfPages as u64) << PAGE_SHIFT,
                    );
                }
            }
        }
    }

    unsafe fn ExpKdbgExtValidatePoolHeader(
        BaseVa: PVOID,
        Entry: PPOOL_HEADER,
        BasePoolType: POOL_TYPE,
    ) -> BOOLEAN {
        if (*Entry).BlockSize() <= 0 {
            return 0u8;
        }
        if ((*Entry).BlockSize() as usize * 8) + (Entry as ULONG_PTR) - (BaseVa as ULONG_PTR) > PAGE_SIZE {
            return 0u8;
        }

        if (*Entry).PreviousSize() == 0 && BaseVa != Entry as PVOID {
            return 0u8;
        }
        if ((*Entry).PreviousSize() as usize * 8) > (Entry as ULONG_PTR) - (BaseVa as ULONG_PTR) {
            return 0u8;
        }

        if (((*Entry).PoolType() - 1) & BASE_POOL_TYPE_MASK as u16) != BasePoolType as u16 {
            return 0u8;
        }

        if ((*Entry).PoolTag & 0x00808080) != 0 {
            return 0u8;
        }

        1u8
    }

    unsafe fn ExpKdbgExtPoolFindPagedPool(
        Tag: ULONG,
        Mask: ULONG,
        FoundCallback: Option<unsafe extern "system" fn(PPOOL_HEADER, PVOID)>,
        CallbackContext: PVOID,
    ) {
        let mut i: ULONG = 0;
        let mut Entry: PPOOL_HEADER;
        let mut BaseVa: PVOID;
        let mut PointerPde: PMMPDE;

        crate::kdbp_print!(b"Searching Paged pool (%p : %p) for Tag: %.4s\n\0",
            MmPagedPoolStart,
            MmPagedPoolEnd,
            core::ptr::addr_of!(Tag) as PCHAR,
        );

        i = RtlFindSetBits(MmPagedPoolInfo.PagedPoolAllocationMap, 1, 0);
        while i != 0xFFFFFFFF {
            BaseVa = ((MmPagedPoolStart as ULONG_PTR) + ((i as ULONG_PTR) << PAGE_SHIFT)) as PVOID;
            Entry = BaseVa as PPOOL_HEADER;

            if (BaseVa as ULONG_PTR) > (MmPagedPoolEnd as ULONG_PTR) ||
               ((BaseVa as ULONG_PTR) + PAGE_SIZE) > (MmPagedPoolEnd as ULONG_PTR)
            {
                break;
            }

            PointerPde = MiAddressToPde(BaseVa);
            if PointerPde >= MmPagedPoolInfo.NextPdeForPagedPoolExpansion as PMMPDE {
                break;
            }

            if MmIsAddressValid(BaseVa) != 0 {
                Entry = BaseVa as PPOOL_HEADER;
                while (Entry as ULONG_PTR) + core::mem::size_of::<POOL_HEADER>() < ((BaseVa as ULONG_PTR) + PAGE_SIZE) {
                    if ExpKdbgExtValidatePoolHeader(BaseVa, Entry, PagedPool) == 0 {
                        Entry = ((Entry as ULONG_PTR) + 8) as PPOOL_HEADER;
                        continue;
                    }

                    if ((*Entry).PoolTag & Mask) == (Tag & Mask) {
                        if let Some(callback) = FoundCallback {
                            callback(Entry, CallbackContext);
                        } else {
                            crate::kdbp_print!(b"%p size: %4d previous size: %4d  %s  %.4s\n\0",
                                Entry,
                                (*Entry).BlockSize() as i32,
                                (*Entry).PreviousSize() as i32,
                                if (*Entry).PoolType() != 0 {
                                    b"(Allocated)\0".as_ptr() as PCHAR
                                } else {
                                    b"(Free)     \0".as_ptr() as PCHAR
                                },
                                core::ptr::addr_of!((*Entry).PoolTag) as PCHAR,
                            );
                        }
                    }

                    Entry = ((Entry as ULONG_PTR) + 8) as PPOOL_HEADER;
                }
            }

            i = RtlFindSetBits(MmPagedPoolInfo.PagedPoolAllocationMap, 1, i + 1);
        }
    }

    unsafe fn ExpKdbgExtPoolFindNonPagedPool(
        Tag: ULONG,
        Mask: ULONG,
        FoundCallback: Option<unsafe extern "system" fn(PPOOL_HEADER, PVOID)>,
        CallbackContext: PVOID,
    ) {
        let mut Entry: PPOOL_HEADER;
        let mut BaseVa: PVOID;

        crate::kdbp_print!(b"Searching NonPaged pool (%p : %p) for Tag: %.4s\n\0",
            MmNonPagedPoolStart,
            MmNonPagedPoolEnd0,
            core::ptr::addr_of!(Tag) as PCHAR,
        );

        BaseVa = MmNonPagedPoolStart;
        while ((BaseVa as ULONG_PTR) + PAGE_SIZE) <= (MmNonPagedPoolEnd0 as ULONG_PTR) {
            if BaseVa >= MmNonPagedPoolExpansionStart {
                break;
            }

            if MmIsAddressValid(BaseVa) == 0 {
                BaseVa = ((BaseVa as ULONG_PTR) + PAGE_SIZE) as PVOID;
                continue;
            }

            Entry = BaseVa as PPOOL_HEADER;
            while (Entry as ULONG_PTR) + core::mem::size_of::<POOL_HEADER>() < ((BaseVa as ULONG_PTR) + PAGE_SIZE) {
                if ExpKdbgExtValidatePoolHeader(BaseVa, Entry, NonPagedPool) == 0 {
                    Entry = ((Entry as ULONG_PTR) + 8) as PPOOL_HEADER;
                    continue;
                }

                if ((*Entry).PoolTag & Mask) == (Tag & Mask) {
                    if let Some(callback) = FoundCallback {
                        callback(Entry, CallbackContext);
                    } else {
                        crate::kdbp_print!(b"%p size: %4d previous size: %4d  %s  %.4s\n\0",
                            Entry,
                            (*Entry).BlockSize() as i32,
                            (*Entry).PreviousSize() as i32,
                            if (*Entry).PoolType() != 0 {
                                b"(Allocated)\0".as_ptr() as PCHAR
                            } else {
                                b"(Free)     \0".as_ptr() as PCHAR
                            },
                            core::ptr::addr_of!((*Entry).PoolTag) as PCHAR,
                        );
                    }
                }

                Entry = ((Entry as ULONG_PTR) + 8) as PPOOL_HEADER;
            }

            BaseVa = ((BaseVa as ULONG_PTR) + PAGE_SIZE) as PVOID;
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn ExpKdbgExtPoolFind(
        Argc: ULONG,
        Argv: *mut PCHAR,
    ) -> BOOLEAN {
    let Argv = opaque_ptr(Argv);
        let mut Tag: ULONG = 0;
        let mut Mask: ULONG = 0;
        let mut PoolType: ULONG = NonPagedPool;

        if Argc == 1 {
            crate::kdbp_print!(b"Specify a tag string\n\0");
            return 1u8;
        }

        if strlen(*Argv.add(1)) != 1 || *(*Argv.add(1)) != b'*' as i8 {
            ExpKdbgExtPoolUsedGetTag(*Argv.add(1), &mut Tag, &mut Mask);
        }

        if Argc > 2 {
            PoolType = strtoul(*Argv.add(2), core::ptr::null_mut(), 0);

            if PoolType > 1 {
                crate::kdbp_print!(b"Only (non) paged pool are supported\n\0");
                return 1u8;
            }
        }

        ExpKdbgExtPoolFindLargePool(Tag, Mask, None, core::ptr::null_mut());

        if PoolType == NonPagedPool {
            ExpKdbgExtPoolFindNonPagedPool(Tag, Mask, None, core::ptr::null_mut());
        } else if PoolType == PagedPool {
            ExpKdbgExtPoolFindPagedPool(Tag, Mask, None, core::ptr::null_mut());
        }

        1u8
    }

    #[no_mangle]
    pub unsafe extern "system" fn ExpKdbgExtIrpFindPrint(
        Entry: PPOOL_HEADER,
        Context: PVOID,
    ) {
    let Entry = opaque_ptr(Entry);
    let Context = opaque_ptr(Context);
        let Irp: PIRP;
        let mut IsComplete: BOOLEAN = 0u8;
        let FindCtxt: *mut IRP_FIND_CTXT = Context as *mut IRP_FIND_CTXT;
        let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
        let mut DriverName: PUNICODE_STRING = core::ptr::null_mut();
        let SData: ULONG_PTR = (*FindCtxt).SData;
        let Criteria: ULONG = (*FindCtxt).Criteria;

        if (*Entry).PoolType() == 0 {
            return;
        }

        Irp = POOL_FREE_BLOCK(Entry as PVOID) as PIRP;

        if (Irp as ULONG_PTR) < (*FindCtxt).RestartAddress {
            return;
        }

        if (*Irp).CurrentLocation <= (*Irp).StackCount + 1 {
            IoStack = IoGetCurrentIrpStackLocation(Irp);

            if !(*IoStack).DeviceObject.is_null() && !(*(*IoStack).DeviceObject).DriverObject.is_null() {
                DriverName = core::ptr::addr_of_mut!((*(*(*IoStack).DeviceObject).DriverObject).DriverName);
            }
        } else {
            IsComplete = 1u8;
        }

        if SData == 0 || Criteria == 0 ||
           (Criteria & 0x1 != 0 && !IoStack.is_null() && SData == (*IoStack).DeviceObject as ULONG_PTR) ||
           (Criteria & 0x2 != 0 && SData == (*Irp).Tail.Overlay.OriginalFileObject as ULONG_PTR) ||
           (Criteria & 0x4 != 0 && !(*Irp).MdlAddress.is_null() && SData == (*(*Irp).MdlAddress).Process as ULONG_PTR) ||
           (Criteria & 0x8 != 0 && SData == (*Irp).Tail.Overlay.Thread as ULONG_PTR) ||
           (Criteria & 0x10 != 0 && SData == (*Irp).UserEvent as ULONG_PTR)
        {
            if IsComplete == 0 {
                crate::kdbp_print!(b"%p Thread %p current stack (%x, %x) belongs to %wZ\n\0",
                    Irp,
                    (*Irp).Tail.Overlay.Thread,
                    (*IoStack).MajorFunction as i32,
                    (*IoStack).MinorFunction as i32,
                    DriverName,
                );
            } else {
                crate::kdbp_print!(b"%p Thread %p is complete (CurrentLocation %d > StackCount %d)\n\0",
                    Irp,
                    (*Irp).Tail.Overlay.Thread,
                    (*Irp).CurrentLocation as i32,
                    ((*Irp).StackCount + 1) as i32,
                );
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn ExpKdbgExtIrpFind(
        Argc: ULONG,
        Argv: *mut PCHAR,
    ) -> BOOLEAN {
    let Argv = opaque_ptr(Argv);
        let mut PoolType: ULONG = NonPagedPool;
        let mut FindCtxt: IRP_FIND_CTXT = core::mem::zeroed();

        if Argc > 1 {
            PoolType = strtoul(*Argv.add(1), core::ptr::null_mut(), 0);

            if PoolType > 1 {
                crate::kdbp_print!(b"Only (non) paged pool are supported\n\0");
                return 1u8;
            }
        }

        RtlZeroMemory(
            core::ptr::addr_of_mut!(FindCtxt) as PVOID,
            core::mem::size_of::<IRP_FIND_CTXT>(),
        );

        if Argc > 2 {
            if KdbpGetHexNumber(*Argv.add(2), &mut FindCtxt.RestartAddress) == 0 {
                crate::kdbp_print!(b"Invalid parameter: %s\n\0", *Argv.add(2));
                FindCtxt.RestartAddress = 0;
            }
        }

        if Argc > 4 {
            if KdbpGetHexNumber(*Argv.add(4), &mut FindCtxt.SData) == 0 {
                FindCtxt.SData = 0;
            } else {
                if strcmp(*Argv.add(3), b"device\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x1;
                } else if strcmp(*Argv.add(3), b"fileobject\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x2;
                } else if strcmp(*Argv.add(3), b"mdlprocess\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x4;
                } else if strcmp(*Argv.add(3), b"thread\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x8;
                } else if strcmp(*Argv.add(3), b"userevent\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x10;
                } else if strcmp(*Argv.add(3), b"arg\0".as_ptr() as PCHAR) == 0 {
                    FindCtxt.Criteria = 0x1f;
                }
            }
        }

        if PoolType == NonPagedPool {
            ExpKdbgExtPoolFindNonPagedPool(
                TAG_IRP,
                0xFFFFFFFF,
                Some(ExpKdbgExtIrpFindPrint),
                core::ptr::addr_of_mut!(FindCtxt) as PVOID,
            );
        } else if PoolType == PagedPool {
            ExpKdbgExtPoolFindPagedPool(
                TAG_IRP,
                0xFFFFFFFF,
                Some(ExpKdbgExtIrpFindPrint),
                core::ptr::addr_of_mut!(FindCtxt) as PVOID,
            );
        }

        1u8
    }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/kdbg.c
//  * PURPOSE:         ARM Memory Manager Kernel Debugger routines
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// /* GLOBALS ********************************************************************/
//
// typedef struct _IRP_FIND_CTXT
// {
//     ULONG_PTR RestartAddress;
//     ULONG_PTR SData;
//     ULONG Criteria;
// } IRP_FIND_CTXT, *PIRP_FIND_CTXT;
//
// extern PVOID MmNonPagedPoolEnd0;
// extern SIZE_T PoolBigPageTableSize;
// extern PPOOL_TRACKER_BIG_PAGES PoolBigPageTable;
//
// #define POOL_BIG_TABLE_ENTRY_FREE 0x1
//
// /* Pool block/header/list access macros */
// #define POOL_ENTRY(x)       (PPOOL_HEADER)((ULONG_PTR)(x) - sizeof(POOL_HEADER))
// #define POOL_FREE_BLOCK(x)  (PLIST_ENTRY)((ULONG_PTR)(x)  + sizeof(POOL_HEADER))
// #define POOL_BLOCK(x, i)    (PPOOL_HEADER)((ULONG_PTR)(x) + ((i) * POOL_BLOCK_SIZE))
// #define POOL_NEXT_BLOCK(x)  POOL_BLOCK((x), (x)->BlockSize)
// #define POOL_PREV_BLOCK(x)  POOL_BLOCK((x), -((x)->PreviousSize))
//
// VOID MiDumpPoolConsumers(BOOLEAN CalledFromDbg, ULONG Tag, ULONG Mask, ULONG Flags);
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// #if DBG && defined(KDBG)
//
// #include <kdbg/kdb.h>
//
// BOOLEAN
// ExpKdbgExtPool(
//     ULONG Argc,
//     PCHAR Argv[])
// {
//     ULONG_PTR Address = 0, Flags = 0;
//     PVOID PoolPage;
//     PPOOL_HEADER Entry;
//     BOOLEAN ThisOne;
//     PULONG Data;
//
//     if (Argc > 1)
//     {
//         /* Get address */
//         if (!KdbpGetHexNumber(Argv[1], &Address))
//         {
//             KdbpPrint("Invalid parameter: %s\n", Argv[1]);
//             return TRUE;
//         }
//     }
//
//     if (Argc > 2)
//     {
//         /* Get flags */
//         if (!KdbpGetHexNumber(Argv[2], &Flags))
//         {
//             KdbpPrint("Invalid parameter: %s\n", Argv[2]);
//             return TRUE;
//         }
//     }
//
//     /* Check if we got an address */
//     if (Address != 0)
//     {
//         /* Get the base page */
//         PoolPage = PAGE_ALIGN(Address);
//     }
//     else
//     {
//         KdbpPrint("Heap is unimplemented\n");
//         return TRUE;
//     }
//
//     /* No paging support! */
//     if (!MmIsAddressValid(PoolPage))
//     {
//         KdbpPrint("Address not accessible!\n");
//         return TRUE;
//     }
//
//     /* Get pool type */
//     if ((Address >= (ULONG_PTR)MmPagedPoolStart) && (Address <= (ULONG_PTR)MmPagedPoolEnd))
//         KdbpPrint("Allocation is from PagedPool region\n");
//     else if ((Address >= (ULONG_PTR)MmNonPagedPoolStart) && (Address <= (ULONG_PTR)MmNonPagedPoolEnd))
//         KdbpPrint("Allocation is from NonPagedPool region\n");
//     else
//     {
//         KdbpPrint("Address 0x%p is not within any pool!\n", (PVOID)Address);
//         return TRUE;
//     }
//
//     /* Loop all entries of that page */
//     Entry = PoolPage;
//     do
//     {
//         /* Check if the address is within that entry */
//         ThisOne = ((Address >= (ULONG_PTR)Entry) &&
//                    (Address < (ULONG_PTR)(Entry + Entry->BlockSize)));
//
//         if (!(Flags & 1) || ThisOne)
//         {
//             /* Print the line */
//             KdbpPrint("%c%p size: %4d previous size: %4d  %s  %.4s\n",
//                      ThisOne ? '*' : ' ', Entry, Entry->BlockSize, Entry->PreviousSize,
//                      (Flags & 0x80000000) ? "" : (Entry->PoolType ? "(Allocated)" : "(Free)     "),
//                      (Flags & 0x80000000) ? "" : (PCHAR)&Entry->PoolTag);
//         }
//
//         if (Flags & 1)
//         {
//             Data = (PULONG)(Entry + 1);
//             KdbpPrint("    %p  %08lx %08lx %08lx %08lx\n"
//                      "    %p  %08lx %08lx %08lx %08lx\n",
//                      &Data[0], Data[0], Data[1], Data[2], Data[3],
//                      &Data[4], Data[4], Data[5], Data[6], Data[7]);
//         }
//
//         /* Go to next entry */
//         Entry = POOL_BLOCK(Entry, Entry->BlockSize);
//     }
//     while ((Entry->BlockSize != 0) && ((ULONG_PTR)Entry < (ULONG_PTR)PoolPage + PAGE_SIZE));
//
//     return TRUE;
// }
//
// static
// VOID
// ExpKdbgExtPoolUsedGetTag(PCHAR Arg, PULONG Tag, PULONG Mask)
// {
//     CHAR Tmp[4];
//     SIZE_T Len;
//     USHORT i;
//
//     /* Get the tag */
//     Len = strlen(Arg);
//     if (Len > 4)
//     {
//         Len = 4;
//     }
//
//     /* Generate the mask to have wildcards support */
//     for (i = 0; i < Len; ++i)
//     {
//         Tmp[i] = Arg[i];
//         if (Tmp[i] != '?')
//         {
//             *Mask |= (0xFF << i * 8);
//         }
//     }
//
//     /* Get the tag in the ulong form */
//     *Tag = *((PULONG)Tmp);
// }
//
// BOOLEAN
// ExpKdbgExtPoolUsed(
//     ULONG Argc,
//     PCHAR Argv[])
// {
//     ULONG Tag = 0;
//     ULONG Mask = 0;
//     ULONG_PTR Flags = 0;
//
//     if (Argc > 1)
//     {
//         /* If we have 2+ args, easy: flags then tag */
//         if (Argc > 2)
//         {
//             ExpKdbgExtPoolUsedGetTag(Argv[2], &Tag, &Mask);
//             if (!KdbpGetHexNumber(Argv[1], &Flags))
//             {
//                 KdbpPrint("Invalid parameter: %s\n", Argv[1]);
//             }
//         }
//         else
//         {
//             /* Otherwise, try to find out whether that's flags */
//             if (strlen(Argv[1]) == 1 ||
//                 (strlen(Argv[1]) == 3 && Argv[1][0] == '0' && (Argv[1][1] == 'x' || Argv[1][1] == 'X')))
//             {
//                 /* Fallback: if reading flags failed, assume it's a tag */
//                 if (!KdbpGetHexNumber(Argv[1], &Flags))
//                 {
//                     ExpKdbgExtPoolUsedGetTag(Argv[1], &Tag, &Mask);
//                 }
//             }
//             /* Or tag */
//             else
//             {
//                 ExpKdbgExtPoolUsedGetTag(Argv[1], &Tag, &Mask);
//             }
//         }
//     }
//
//     /* Call the dumper */
//     MiDumpPoolConsumers(TRUE, Tag, Mask, Flags);
//
//     return TRUE;
// }
//
// static
// VOID
// ExpKdbgExtPoolFindLargePool(
//     ULONG Tag,
//     ULONG Mask,
//     VOID (NTAPI* FoundCallback)(PPOOL_TRACKER_BIG_PAGES, PVOID),
//     PVOID CallbackContext)
// {
//     ULONG i;
//
//     KdbpPrint("Scanning large pool allocation table for Tag: %.4s (%p : %p)\n", (PCHAR)&Tag, &PoolBigPageTable[0], &PoolBigPageTable[PoolBigPageTableSize - 1]);
//
//     for (i = 0; i < PoolBigPageTableSize; i++)
//     {
//         /* Free entry? */
//         if ((ULONG_PTR)PoolBigPageTable[i].Va & POOL_BIG_TABLE_ENTRY_FREE)
//         {
//             continue;
//         }
//
//         if ((PoolBigPageTable[i].Key & Mask) == (Tag & Mask))
//         {
//             if (FoundCallback != NULL)
//             {
//                 FoundCallback(&PoolBigPageTable[i], CallbackContext);
//             }
//             else
//             {
//                 /* Print the line */
//                 KdbpPrint("%p: tag %.4s, size: %I64x\n",
//                           PoolBigPageTable[i].Va, (PCHAR)&PoolBigPageTable[i].Key,
//                           PoolBigPageTable[i].NumberOfPages << PAGE_SHIFT);
//             }
//         }
//     }
// }
//
// static
// BOOLEAN
// ExpKdbgExtValidatePoolHeader(
//     PVOID BaseVa,
//     PPOOL_HEADER Entry,
//     POOL_TYPE BasePoolTye)
// {
//     /* Block size cannot be NULL or negative and it must cover the page */
//     if (Entry->BlockSize <= 0)
//     {
//         return FALSE;
//     }
//     if (Entry->BlockSize * 8 + (ULONG_PTR)Entry - (ULONG_PTR)BaseVa > PAGE_SIZE)
//     {
//         return FALSE;
//     }
//
//     /*
//      * PreviousSize cannot be 0 unless on page begin
//      * And it cannot be bigger that our current
//      * position in page
//      */
//     if (Entry->PreviousSize == 0 && BaseVa != Entry)
//     {
//         return FALSE;
//     }
//     if (Entry->PreviousSize * 8 > (ULONG_PTR)Entry - (ULONG_PTR)BaseVa)
//     {
//         return FALSE;
//     }
//
//     /* Must be paged pool */
//     if (((Entry->PoolType - 1) & BASE_POOL_TYPE_MASK) != BasePoolTye)
//     {
//         return FALSE;
//     }
//
//     /* Match tag mask */
//     if ((Entry->PoolTag & 0x00808080) != 0)
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static
// VOID
// ExpKdbgExtPoolFindPagedPool(
//     ULONG Tag,
//     ULONG Mask,
//     VOID (NTAPI* FoundCallback)(PPOOL_HEADER, PVOID),
//     PVOID CallbackContext)
// {
//     ULONG i = 0;
//     PPOOL_HEADER Entry;
//     PVOID BaseVa;
//     PMMPDE PointerPde;
//
//     KdbpPrint("Searching Paged pool (%p : %p) for Tag: %.4s\n", MmPagedPoolStart, MmPagedPoolEnd, (PCHAR)&Tag);
//
//     /*
//      * To speed up paged pool search, we will use the allocation bipmap.
//      * This is possible because we live directly in the kernel :-)
//      */
//     i = RtlFindSetBits(MmPagedPoolInfo.PagedPoolAllocationMap, 1, 0);
//     while (i != 0xFFFFFFFF)
//     {
//         BaseVa = (PVOID)((ULONG_PTR)MmPagedPoolStart + (i << PAGE_SHIFT));
//         Entry = BaseVa;
//
//         /* Validate our address */
//         if ((ULONG_PTR)BaseVa > (ULONG_PTR)MmPagedPoolEnd || (ULONG_PTR)BaseVa + PAGE_SIZE > (ULONG_PTR)MmPagedPoolEnd)
//         {
//             break;
//         }
//
//         /* Check whether we are beyond expansion */
//         PointerPde = MiAddressToPde(BaseVa);
//         if (PointerPde >= MmPagedPoolInfo.NextPdeForPagedPoolExpansion)
//         {
//             break;
//         }
//
//         /* Check if allocation is valid */
//         if (MmIsAddressValid(BaseVa))
//         {
//             for (Entry = BaseVa;
//                  (ULONG_PTR)Entry + sizeof(POOL_HEADER) < (ULONG_PTR)BaseVa + PAGE_SIZE;
//                  Entry = (PVOID)((ULONG_PTR)Entry + 8))
//             {
//                 /* Try to find whether we have a pool entry */
//                 if (!ExpKdbgExtValidatePoolHeader(BaseVa, Entry, PagedPool))
//                 {
//                     continue;
//                 }
//
//                 if ((Entry->PoolTag & Mask) == (Tag & Mask))
//                 {
//                     if (FoundCallback != NULL)
//                     {
//                         FoundCallback(Entry, CallbackContext);
//                     }
//                     else
//                     {
//                         /* Print the line */
//                         KdbpPrint("%p size: %4d previous size: %4d  %s  %.4s\n",
//                                   Entry, Entry->BlockSize, Entry->PreviousSize,
//                                   Entry->PoolType ? "(Allocated)" : "(Free)     ",
//                                   (PCHAR)&Entry->PoolTag);
//                     }
//                 }
//             }
//         }
//
//         i = RtlFindSetBits(MmPagedPoolInfo.PagedPoolAllocationMap, 1, i + 1);
//     }
// }
//
// static
// VOID
// ExpKdbgExtPoolFindNonPagedPool(
//     ULONG Tag,
//     ULONG Mask,
//     VOID (NTAPI* FoundCallback)(PPOOL_HEADER, PVOID),
//     PVOID CallbackContext)
// {
//     PPOOL_HEADER Entry;
//     PVOID BaseVa;
//
//     KdbpPrint("Searching NonPaged pool (%p : %p) for Tag: %.4s\n", MmNonPagedPoolStart, MmNonPagedPoolEnd0, (PCHAR)&Tag);
//
//     /* Brute force search: start browsing the whole non paged pool */
//     for (BaseVa = MmNonPagedPoolStart;
//          (ULONG_PTR)BaseVa + PAGE_SIZE <= (ULONG_PTR)MmNonPagedPoolEnd0;
//          BaseVa = (PVOID)((ULONG_PTR)BaseVa + PAGE_SIZE))
//     {
//         Entry = BaseVa;
//
//         /* Check whether we are beyond expansion */
//         if (BaseVa >= MmNonPagedPoolExpansionStart)
//         {
//             break;
//         }
//
//         /* Check if allocation is valid */
//         if (!MmIsAddressValid(BaseVa))
//         {
//             continue;
//         }
//
//         for (Entry = BaseVa;
//              (ULONG_PTR)Entry + sizeof(POOL_HEADER) < (ULONG_PTR)BaseVa + PAGE_SIZE;
//              Entry = (PVOID)((ULONG_PTR)Entry + 8))
//         {
//             /* Try to find whether we have a pool entry */
//             if (!ExpKdbgExtValidatePoolHeader(BaseVa, Entry, NonPagedPool))
//             {
//                 continue;
//             }
//
//             if ((Entry->PoolTag & Mask) == (Tag & Mask))
//             {
//                 if (FoundCallback != NULL)
//                 {
//                     FoundCallback(Entry, CallbackContext);
//                 }
//                 else
//                 {
//                     /* Print the line */
//                     KdbpPrint("%p size: %4d previous size: %4d  %s  %.4s\n",
//                               Entry, Entry->BlockSize, Entry->PreviousSize,
//                               Entry->PoolType ? "(Allocated)" : "(Free)     ",
//                               (PCHAR)&Entry->PoolTag);
//                 }
//             }
//         }
//     }
// }
//
// BOOLEAN
// ExpKdbgExtPoolFind(
//     ULONG Argc,
//     PCHAR Argv[])
// {
//     ULONG Tag = 0;
//     ULONG Mask = 0;
//     ULONG PoolType = NonPagedPool;
//
//     if (Argc == 1)
//     {
//         KdbpPrint("Specify a tag string\n");
//         return TRUE;
//     }
//
//     /* First arg is tag */
//     if (strlen(Argv[1]) != 1 || Argv[1][0] != '*')
//     {
//         ExpKdbgExtPoolUsedGetTag(Argv[1], &Tag, &Mask);
//     }
//
//     /* Second arg might be pool to search */
//     if (Argc > 2)
//     {
//         PoolType = strtoul(Argv[2], NULL, 0);
//
//         if (PoolType > 1)
//         {
//             KdbpPrint("Only (non) paged pool are supported\n");
//             return TRUE;
//         }
//     }
//
//     /* First search for large allocations */
//     ExpKdbgExtPoolFindLargePool(Tag, Mask, NULL, NULL);
//
//     if (PoolType == NonPagedPool)
//     {
//         ExpKdbgExtPoolFindNonPagedPool(Tag, Mask, NULL, NULL);
//     }
//     else if (PoolType == PagedPool)
//     {
//         ExpKdbgExtPoolFindPagedPool(Tag, Mask, NULL, NULL);
//     }
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// ExpKdbgExtIrpFindPrint(
//     PPOOL_HEADER Entry,
//     PVOID Context)
// {
//     PIRP Irp;
//     BOOLEAN IsComplete = FALSE;
//     PIRP_FIND_CTXT FindCtxt = Context;
//     PIO_STACK_LOCATION IoStack = NULL;
//     PUNICODE_STRING DriverName = NULL;
//     ULONG_PTR SData = FindCtxt->SData;
//     ULONG Criteria = FindCtxt->Criteria;
//
//     /* Free entry, ignore */
//     if (Entry->PoolType == 0)
//     {
//         return;
//     }
//
//     /* Get the IRP */
//     Irp = (PIRP)POOL_FREE_BLOCK(Entry);
//
//     /* Bail out if not matching restart address */
//     if ((ULONG_PTR)Irp < FindCtxt->RestartAddress)
//     {
//         return;
//     }
//
//     /* Avoid bogus IRP stack locations */
//     if (Irp->CurrentLocation <= Irp->StackCount + 1)
//     {
//         IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//         /* Get associated driver */
//         if (IoStack->DeviceObject && IoStack->DeviceObject->DriverObject)
//             DriverName = &IoStack->DeviceObject->DriverObject->DriverName;
//     }
//     else
//     {
//         IsComplete = TRUE;
//     }
//
//     /* Display if: no data, no criteria or if criteria matches data */
//     if (SData == 0 || Criteria == 0 ||
//         (Criteria & 0x1 && IoStack && SData == (ULONG_PTR)IoStack->DeviceObject) ||
//         (Criteria & 0x2 && SData == (ULONG_PTR)Irp->Tail.Overlay.OriginalFileObject) ||
//         (Criteria & 0x4 && Irp->MdlAddress && SData == (ULONG_PTR)Irp->MdlAddress->Process) ||
//         (Criteria & 0x8 && SData == (ULONG_PTR)Irp->Tail.Overlay.Thread) ||
//         (Criteria & 0x10 && SData == (ULONG_PTR)Irp->UserEvent))
//     {
//         if (!IsComplete)
//         {
//             KdbpPrint("%p Thread %p current stack (%x, %x) belongs to %wZ\n", Irp, Irp->Tail.Overlay.Thread, IoStack->MajorFunction, IoStack->MinorFunction, DriverName);
//         }
//         else
//         {
//             KdbpPrint("%p Thread %p is complete (CurrentLocation %d > StackCount %d)\n", Irp, Irp->Tail.Overlay.Thread, Irp->CurrentLocation, Irp->StackCount + 1);
//         }
//     }
// }
//
// BOOLEAN
// ExpKdbgExtIrpFind(
//     ULONG Argc,
//     PCHAR Argv[])
// {
//     ULONG PoolType = NonPagedPool;
//     IRP_FIND_CTXT FindCtxt;
//
//     /* Pool type */
//     if (Argc > 1)
//     {
//         PoolType = strtoul(Argv[1], NULL, 0);
//
//         if (PoolType > 1)
//         {
//             KdbpPrint("Only (non) paged pool are supported\n");
//             return TRUE;
//         }
//     }
//
//     RtlZeroMemory(&FindCtxt, sizeof(IRP_FIND_CTXT));
//
//     /* Restart address */
//     if (Argc > 2)
//     {
//         if (!KdbpGetHexNumber(Argv[2], &FindCtxt.RestartAddress))
//         {
//             KdbpPrint("Invalid parameter: %s\n", Argv[2]);
//             FindCtxt.RestartAddress = 0;
//         }
//     }
//
//     if (Argc > 4)
//     {
//         if (!KdbpGetHexNumber(Argv[4], &FindCtxt.SData))
//         {
//             FindCtxt.SData = 0;
//         }
//         else
//         {
//             if (strcmp(Argv[3], "device") == 0)
//             {
//                 FindCtxt.Criteria = 0x1;
//             }
//             else if (strcmp(Argv[3], "fileobject") == 0)
//             {
//                 FindCtxt.Criteria = 0x2;
//             }
//             else if (strcmp(Argv[3], "mdlprocess") == 0)
//             {
//                 FindCtxt.Criteria = 0x4;
//             }
//             else if (strcmp(Argv[3], "thread") == 0)
//             {
//                 FindCtxt.Criteria = 0x8;
//             }
//             else if (strcmp(Argv[3], "userevent") == 0)
//             {
//                 FindCtxt.Criteria = 0x10;
//             }
//             else if (strcmp(Argv[3], "arg") == 0)
//             {
//                 FindCtxt.Criteria = 0x1f;
//             }
//         }
//     }
//
//     if (PoolType == NonPagedPool)
//     {
//         ExpKdbgExtPoolFindNonPagedPool(TAG_IRP, 0xFFFFFFFF, ExpKdbgExtIrpFindPrint, &FindCtxt);
//     }
//     else if (PoolType == PagedPool)
//     {
//         ExpKdbgExtPoolFindPagedPool(TAG_IRP, 0xFFFFFFFF, ExpKdbgExtIrpFindPrint, &FindCtxt);
//     }
//
//     return TRUE;
// }
//
// #endif // DBG && defined(KDBG)
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
