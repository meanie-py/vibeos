//! ISA bus handler — translate ISA bus addresses to system addresses.
//!
//! Translated from `hal/halx86/legacy/bus/isabus.c`.
//!
//! Single function: defer to `HalpTranslateSystemBusAddress` first, and if
//! that doesn't recognise the memory-space address, retry via
//! `HalTranslateBusAddress(Eisa, ...)` so EISA aliases (MMIO 0x00100000+)
//! are picked up.  Compiled for both i386 and amd64.

use ntkrnl_sys::hal_ddk::{
    HalTranslateBusAddress, BOOLEAN, INTERFACE_TYPE, PBUS_HANDLER, PHYSICAL_ADDRESS,
    PPHYSICAL_ADDRESS, PULONG, _INTERFACE_TYPE_Eisa as IFT_EISA,
};

// HalpTranslateSystemBusAddress is also defined in hal-rust (sysbus.rs) as
// `extern "system"` — same crate, so the resolver picks the local symbol.
unsafe extern "system" {
    fn HalpTranslateSystemBusAddress(
        BusHandler: PBUS_HANDLER,
        RootHandler: PBUS_HANDLER,
        BusAddress: PHYSICAL_ADDRESS,
        AddressSpace: PULONG,
        TranslatedAddress: PPHYSICAL_ADDRESS,
    ) -> BOOLEAN;
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpTranslateIsaBusAddress(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: PULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    unsafe {
        let status = HalpTranslateSystemBusAddress(
            BusHandler,
            RootHandler,
            BusAddress,
            AddressSpace,
            TranslatedAddress,
        );

        // If it didn't work and it was memory address space...
        if status == 0 && *AddressSpace == 0 {
            return HalTranslateBusAddress(
                IFT_EISA as INTERFACE_TYPE,
                (*BusHandler).BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            );
        }

        status
    }
}
