//! System bus handler — Translate physical bus addresses to system
//! addresses by walking the BUS_HANDLER's SUPPORTED_RANGES list, and
//! map bus interrupt vectors through HalpGetRootInterruptVector.
//!
//! Translated from `hal/halx86/legacy/bus/sysbus.c`.

use ntkrnl_sys::hal_ddk::{
    HalpGetRootInterruptVector, HalpIDTUsageFlags, BOOLEAN, IDT_REGISTERED, PBUS_HANDLER,
    PHYSICAL_ADDRESS, PKAFFINITY, PKIRQL, PPHYSICAL_ADDRESS, PSUPPORTED_RANGE, PULONG, ULONG,
};

// ---------------------------------------------------------------------------
// HalpTranslateSystemBusAddress — figure out which SUPPORTED_RANGE a bus
// address falls inside (by walking IO/Memory/PrefetchMemory linked lists),
// translate to system base, optionally update the address-space tag.
// AddressSpace 0 = memory (try prefetch first, then plain), 1 = I/O.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpTranslateSystemBusAddress(
    BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    unsafe {
        let bus_addr = BusAddress.QuadPart;
        let bus_addresses = (*BusHandler).BusAddresses;

        // Walk one of the SUPPORTED_RANGE linked lists.  Returns the
        // matching node (or null if none).  The first node lives inline
        // in the BUS_HANDLER's SUPPORTED_RANGES struct; subsequent
        // overflow nodes hang off Next.
        let walk = |head: PSUPPORTED_RANGE| -> PSUPPORTED_RANGE {
            let mut r = head;
            while !r.is_null() {
                if bus_addr >= (*r).Base && bus_addr <= (*r).Limit {
                    return r;
                }
                r = (*r).Next;
            }
            core::ptr::null_mut()
        };

        let mut range: PSUPPORTED_RANGE = core::ptr::null_mut();

        match *AddressSpace {
            0 => {
                // Memory — try prefetch first (per C order), then non-prefetch.
                range = walk(&raw mut (*bus_addresses).PrefetchMemory);
                if range.is_null() {
                    range = walk(&raw mut (*bus_addresses).Memory);
                }
            }
            1 => {
                // I/O space.
                range = walk(&raw mut (*bus_addresses).IO);
            }
            _ => {}
        }

        if range.is_null() {
            return 0; /* FALSE */
        }

        core::ptr::write_unaligned(
            core::ptr::addr_of_mut!((*TranslatedAddress).QuadPart),
            bus_addr + (*range).SystemBase,
        );
        *AddressSpace = (*range).SystemAddressSpace;
        1 /* TRUE */
    }
}

// ---------------------------------------------------------------------------
// HalpGetSystemInterruptVector — defer to HalpGetRootInterruptVector,
// then check the IDT-usage flag table to see if the vector is already
// owned by something else (the HAL itself or another driver).  Returns
// 0 in that case (failure), else the vector.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetSystemInterruptVector(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    unsafe {
        let vector =
            HalpGetRootInterruptVector(BusInterruptLevel, BusInterruptVector, Irql, Affinity);

        let entry = (&raw const HalpIDTUsageFlags as *const ntkrnl_sys::hal_ddk::IDTUsageFlags)
            .add(vector as usize);
        if (*entry).Flags & (IDT_REGISTERED as u8) != 0 {
            0
        } else {
            vector
        }
    }
}
