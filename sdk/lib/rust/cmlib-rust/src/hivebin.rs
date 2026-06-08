use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{HBIN, HMAP_ENTRY, HSTORAGE_TYPE, PHBIN, PHCELL, PHHIVE, PHMAP_ENTRY, RtlSetBits, RtlInitializeBitMap, HSTORAGE_TYPE_Stable as Stable};
use crate::glue::{round_up, HBLOCK_SIZE, HV_HBIN_SIGNATURE, TAG_CM};
// CMAPI = NTAPI → stdcall on i386 (_HvpAddBin@12): extern "system".
#[no_mangle]
pub unsafe extern "system" fn HvpAddBin(
    RegistryHive: PHHIVE,
    Size: u32,
    Storage: HSTORAGE_TYPE,
) -> PHBIN {
    let mut BlockList: PHMAP_ENTRY;
    let mut Bin: PHBIN;
    let BinSize: u32;
    let mut i: u32;
    let BitmapSize: u32;
    let BlockCount: u32;
    let OldBlockListSize: u32;
    let mut Block: PHCELL;

    BinSize = round_up(Size + core::mem::size_of::<HBIN>() as u32, HBLOCK_SIZE);
    BlockCount = BinSize / HBLOCK_SIZE;

    Bin = ((*RegistryHive).Allocate.unwrap())(BinSize as _, 1u8, TAG_CM) as PHBIN;
    if Bin.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::write_bytes(Bin as *mut u8, 0, BinSize as usize); // RtlZeroMemory: BinSize BYTES

    (*Bin).Signature = HV_HBIN_SIGNATURE;
    (*Bin).FileOffset = (*RegistryHive).Storage[Storage as usize].Length * HBLOCK_SIZE;
    (*Bin).Size = BinSize;

    OldBlockListSize = (*RegistryHive).Storage[Storage as usize].Length;
    BlockList = ((*RegistryHive).Allocate.unwrap())(
        ((core::mem::size_of::<HMAP_ENTRY>() as u32) * (OldBlockListSize + BlockCount)) as _,
        1u8,
        TAG_CM,
    ) as PHMAP_ENTRY;
    if BlockList.is_null() {
        ((*RegistryHive).Free.unwrap())(Bin as *mut core::ffi::c_void, 0);
        return core::ptr::null_mut();
    }

    if OldBlockListSize > 0 {
        core::ptr::copy_nonoverlapping(
            (*RegistryHive).Storage[Storage as usize].BlockList,
            BlockList,
            OldBlockListSize as usize,
        );
        ((*RegistryHive).Free.unwrap())(
            (*RegistryHive).Storage[Storage as usize].BlockList as *mut core::ffi::c_void,
            0,
        );
    }

    (*RegistryHive).Storage[Storage as usize].BlockList = BlockList;
    (*RegistryHive).Storage[Storage as usize].Length += BlockCount;

    i = 0;
    while i < BlockCount {
        // Set only BlockAddress/BinAddress (as the C does) — HMAP_ENTRY also has
        // CmView/MemAlloc, left as Allocate returned them.
        let entry = (*RegistryHive).Storage[Storage as usize]
            .BlockList
            .offset((OldBlockListSize + i) as isize);
        (*entry).BlockAddress = ((Bin as usize) + (i as usize * HBLOCK_SIZE as usize)) as _;
        (*entry).BinAddress = Bin as _;
        i += 1;
    }

    Block = Bin.offset(1) as PHCELL;
    (*Block).Size = (BinSize - core::mem::size_of::<HBIN>() as u32) as i32;

    if Storage == Stable {
        BitmapSize = round_up(
            (*RegistryHive).Storage[Stable as usize].Length,
            core::mem::size_of::<u32>() as u32 * 8,
        ) / 8;

        if BitmapSize > (*RegistryHive).DirtyVector.SizeOfBitMap / 8 {
            let BitmapBuffer: *mut u32;

            BitmapBuffer = ((*RegistryHive).Allocate.unwrap())(BitmapSize as _, 1u8, TAG_CM) as *mut u32;
            core::ptr::write_bytes(BitmapBuffer as *mut u8, 0, BitmapSize as usize); // BYTES
            if (*RegistryHive).DirtyVector.SizeOfBitMap > 0 {
                core::ptr::copy_nonoverlapping(
                    (*RegistryHive).DirtyVector.Buffer,
                    BitmapBuffer,
                    ((*RegistryHive).DirtyVector.SizeOfBitMap / 8) as usize / core::mem::size_of::<u32>(),
                );
                ((*RegistryHive).Free.unwrap())(
                    (*RegistryHive).DirtyVector.Buffer as *mut core::ffi::c_void,
                    0,
                );
            }
            RtlInitializeBitMap(
                core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
                BitmapBuffer,
                BitmapSize * 8,
            );
        }

        RtlSetBits(
            core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
            (*Bin).FileOffset / HBLOCK_SIZE,
            BlockCount,
        );

        (*(*RegistryHive).BaseBlock).Length += BinSize;
    }

    Bin
}


// ============================================================================
