use ntkrnl_sys::hal_ddk::{APC_LEVEL, CLOCK2_LEVEL, DISPATCH_LEVEL, EFLAGS_INTERRUPT_MASK, EFLAGS_V86_MASK, EISA_ELCR, EISA_ELCR_MASTER, EISA_ELCR_SLAVE, HIGH_LEVEL, HalpApcInterrupt, HalpApcInterrupt2ndEntry, HalpDismissIrq07, HalpDismissIrq07Level, HalpDismissIrq13, HalpDismissIrq13Level, HalpDismissIrq15, HalpDismissIrq15Level, HalpDismissIrqGeneric, HalpDismissIrqLevel, HalpDispatchInterrupt, HalpDispatchInterrupt2, HalpDispatchInterrupt2ndEntry, HalpGetRootInterruptVector, HalpHardwareInterruptLevel, HalpInitializeLegacyPICs, HalpInitializePICs, HalpIrqToVector, HalpRegisterVector, HalpVectorToIrq, HalpVectorToIrql, I8259_ISR, I8259_OCW2, I8259_OCW3, IDT_INTERNAL, InterruptMode, KGDT_R0_CODE, KIRQL, 0i32, PASSIVE_LEVEL, PHAL_DISMISS_INTERRUPT, PHAL_SW_INTERRUPT_HANDLER, PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY, PIC1_CONTROL_PORT, PIC1_DATA_PORT, PIC2_CONTROL_PORT, PIC2_DATA_PORT, PIC_CASCADE_IRQ, PIC_MASK, PIC_TIMER_IRQ, PKAFFINITY, PKIRQL, PKPCR, PKTRAP_FRAME, PRIMARY_VECTOR_BASE, PROFILE_LEVEL, SYNCH_LEVEL, 1i32};
use ntkrnl_sys::ddk::{KeBugCheck, KeBugCheckEx, KiDeliverApc, KiDispatchInterrupt, KiEnterInterruptTrap, KiEoiHelper, KiUnexpectedInterrupt, KiUserTrap};
use core::ffi::c_void;

#[inline(always)]

#[no_mangle]
pub unsafe extern "system" fn HalpInitializePICs(EnableInterrupts: BOOLEAN) {
    unsafe extern "C" {
        fn __readeflags() -> ULONG;
        fn __writeeflags(eflags: ULONG);
        fn _disable();
        fn __inbyte(port: USHORT) -> UCHAR;
    }

    let mut eflags: ULONG;
    let mut elcr: EISA_ELCR = core::mem::zeroed();
    let i: ULONG;
    let j: ULONG;
    let elcr_found: BOOLEAN;

    eflags = unsafe { __readeflags() };
    unsafe { _disable() };

    unsafe { HalpInitializeLegacyPICs() };

    elcr.Bits = (unsafe { __inbyte(0x4D1) } as USHORT) << 8 | unsafe { __inbyte(0x4D0) } as USHORT;

    #[cfg(not(target_feature = "pc98"))]
    {
        elcr_found = (elcr.Master.Irq0Level() == 0)
            && (elcr.Master.Irq1Level() == 0)
            && (elcr.Master.Irq2Level() == 0)
            && (elcr.Slave.Irq8Level() == 0)
            && (elcr.Slave.Irq13Level() == 0);
    }

    #[cfg(target_feature = "pc98")]
    {
        if elcr.Bits == 0xFFFF {
            elcr.Master.set_Irq0Level(0);
            elcr.Master.set_Irq1Level(0);
            elcr.Master.set_Irq7Level(0);
            elcr.Slave.set_Irq8Level(0);
        }
        elcr_found = 1;
    }

    if elcr_found != 0 {
        HalpEisaELCR = elcr.Bits;

        let mut i: ULONG = 1;
        let mut j: ULONG = 0;
        while j < 16 {
            if (HalpEisaELCR as ULONG & i) != 0 {
                SWInterruptHandlerTable[(j + 4) as usize] = HalpHardwareInterruptLevel;

                HalpSpecialDismissTable[j as usize] = HalpSpecialDismissLevelTable[j as usize];
            }
            i <<= 1;
            j += 1;
        }
    }

    unsafe {
        HalpRegisterVector(
            IDT_INTERNAL,
            PRIMARY_VECTOR_BASE + PIC_CASCADE_IRQ,
            PRIMARY_VECTOR_BASE + PIC_CASCADE_IRQ,
            HIGH_LEVEL,
        )
    };

    if EnableInterrupts != 0 {
        eflags |= EFLAGS_INTERRUPT_MASK;
    }
    unsafe { __writeeflags(eflags) };
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    PRIMARY_VECTOR_BASE + Irq
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    PRIMARY_VECTOR_BASE + Irq
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE)
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE)
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    (PROFILE_LEVEL - (Vector - PRIMARY_VECTOR_BASE)) as KIRQL
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    (PROFILE_LEVEL - (Vector - PRIMARY_VECTOR_BASE)) as KIRQL
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
    }
    
    let pcr = unsafe { KeGetPcr() };
    let current_irql: KIRQL;

    current_irql = unsafe { (*pcr).Irql };
    unsafe { (*pcr).Irql = DISPATCH_LEVEL as KIRQL };

    current_irql
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToSynchLevel() -> KIRQL {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
    }
    
    let pcr = unsafe { KeGetPcr() };
    let current_irql: KIRQL;

    unsafe {
        current_irql = (*pcr).Irql;
        (*pcr).Irql = SYNCH_LEVEL as KIRQL;
    }

    #[cfg(debug_assertions)]
    unsafe {
        if current_irql > SYNCH_LEVEL as KIRQL {
            KeBugCheckEx(
                0x0A, // IRQL_NOT_GREATER_OR_EQUAL
                current_irql as usize,
                SYNCH_LEVEL as usize,
                0,
                1,
            );
        }
    }

    current_irql
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalRequestSoftwareInterrupt(Irql: KIRQL) {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
        fn __readeflags() -> u32;
        fn __writeeflags(eflags: u32);
        fn _disable();
    }
    
    unsafe extern "C" {
        static SWInterruptLookUpTable: [KIRQL; 4];
        static SWInterruptHandlerTable: [Option<unsafe extern "C" fn()>; 3];
    }

    let eflags = __readeflags();
    _disable();

    let pcr = KeGetPcr();
    (*pcr).IRR |= 1 << Irql;

    let pending_irql = SWInterruptLookUpTable[((*pcr).IRR & 3) as usize];
    if pending_irql > (*pcr).Irql {
        if let Some(handler) = SWInterruptHandlerTable[pending_irql as usize] {
            handler();
        }
    }

    __writeeflags(eflags);
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalRequestSoftwareInterrupt(Irql: KIRQL) {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
        fn __readeflags() -> u64;
        fn __writeeflags(eflags: u64);
        fn _disable();
    }
    
    unsafe extern "C" {
        static SWInterruptLookUpTable: [KIRQL; 4];
        static SWInterruptHandlerTable: [Option<unsafe extern "C" fn()>; 3];
    }

    let eflags = __readeflags();
    _disable();

    let pcr = KeGetPcr();
    (*pcr).IRR |= 1 << Irql;

    let pending_irql = SWInterruptLookUpTable[((*pcr).IRR & 3) as usize];
    if pending_irql > (*pcr).Irql {
        if let Some(handler) = SWInterruptHandlerTable[pending_irql as usize] {
            handler();
        }
    }

    __writeeflags(eflags);
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalClearSoftwareInterrupt(Irql: KIRQL) {
    unsafe {
        (*KeGetPcr()).IRR &= !(1 << Irql);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalClearSoftwareInterrupt(Irql: KIRQL) {
    unsafe {
        (*KeGetPcr()).IRR &= !(1 << Irql);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpEndSoftwareInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;

    let Pcr = unsafe { KeGetPcr() };
    let mut PendingIrql: ULONG = 0;
    let PendingIrqlMask: ULONG;
    let PendingIrqMask: ULONG;
    let mut Mask: PIC_MASK = unsafe { core::mem::zeroed() };

    unsafe { (*Pcr).Irql = OldIrql };

    loop {
        PendingIrqlMask = unsafe { (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize] };
        if PendingIrqlMask == 0 {
            return core::ptr::null_mut();
        }

        if unsafe { (*Pcr).IrrActive & 0xFFFFFFF0 != 0 } {
            return core::ptr::null_mut();
        }

        unsafe { BitScanReverse(&raw mut PendingIrql, PendingIrqlMask) };
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            Mask.Both = unsafe { (*Pcr).IDR as USHORT & 0xFFFF };
            unsafe {
                write_port_uchar(PIC1_DATA_PORT as *mut u8, Mask.__bindgen_anon_1.Master);
                write_port_uchar(PIC2_DATA_PORT as *mut u8, Mask.__bindgen_anon_1.Slave);
            }

            PendingIrqMask = 1 << PendingIrql;
            unsafe {
                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;
            }

            unsafe {
                let handler = SWInterruptHandlerTable[PendingIrql as usize];
                if let Some(f) = handler {
                    f();
                }
            }

            unsafe { (*Pcr).IrrActive ^= PendingIrqMask };
        } else {
            return unsafe { SWInterruptHandlerTable2[PendingIrql as usize] };
        }
    }
}

#[inline(always)]
unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

unsafe extern "C" {
    fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
    fn KeGetPcr() -> PKPCR;
    static mut FindHigherIrqlMask: [ULONG; 32];
    static mut SWInterruptHandlerTable: [Option<unsafe extern "C" fn()>; 32];
    static mut SWInterruptHandlerTable2: [PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY; 32];
}

unsafe fn _HalpDismissIrqGeneric(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(Port: u16) -> u8;
        fn __outbyte(Port: u16, Value: u8);
        fn _enable();
    }
    
    unsafe extern "C" {
        static mut KiI8259MaskTable: [u16; 32];
    }

    let Pcr = KeGetPcr();

    let CurrentIrql = (*Pcr).Irql;

    if Irql > CurrentIrql {
        (*Pcr).Irql = Irql;
        *OldIrql = CurrentIrql;

        let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
        Ocw2.set_Bits(0);
        Ocw2.set_EoiMode(3);

        if Irq >= 8 {
            #[cfg(not(target_feature = "pc98"))]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits() | ((Irq - 8) & 0xFF) as u8);
                __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits() | PIC_CASCADE_IRQ as u8);
            }

            #[cfg(target_feature = "pc98")]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits() | ((Irq - 8) & 0xFF) as u8);

                let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
                Ocw3.set_Bits(0);
                Ocw3.set_Sbo(1);
                Ocw3.set_ReadRequest(3);
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits());

                let mut Isr: I8259_ISR = core::mem::zeroed();
                Isr.set_Bits(__inbyte(PIC2_CONTROL_PORT as u16));

                if Isr.Bits() == 0 {
                    __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits() | PIC_CASCADE_IRQ as u8);
                }
            }
        } else {
            __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits() | (Irq & 0xFF) as u8);
        }

        _enable();
        return 1;
    }

    (*Pcr).IRR |= 1 << (Irq + 4);

    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.set_Both((KiI8259MaskTable[CurrentIrql as usize] | (*Pcr).IDR as u16) & 0xFFFF);
    __outbyte(PIC1_DATA_PORT as u16, Mask.Master());
    __outbyte(PIC2_DATA_PORT as u16, Mask.Slave());

    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrqGeneric(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe { _HalpDismissIrqGeneric(Irql, Irq, OldIrql) }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq15(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(Port: u16) -> u8;
        fn __outbyte(Port: u16, Value: u8);
    }

    let mut ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut isr: I8259_ISR;

    ocw3.set_Bits(0);
    ocw3.set_Sbo(1);
    ocw3.set_ReadRequest(3);
    __outbyte(PIC2_CONTROL_PORT as u16, ocw3.Bits());

    isr = core::mem::zeroed();
    isr.set_Bits(__inbyte(PIC2_CONTROL_PORT as u16));

    if isr.Irq7() == 0 {
        ocw2.set_Bits(0);
        ocw2.set_EoiMode(1);
        __outbyte(PIC1_CONTROL_PORT as u16, ocw2.Bits() | PIC_CASCADE_IRQ as u8);

        return 0;
    }

    _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq13(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __outbyte(Port: u16, Value: u8);
    }
    
    unsafe 
    
    unsafe {
        __outbyte(0xF0, 0);
        _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq08(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __outbyte(Port: u16, Value: u8);
    }

    const CPU_IO_o_FPU_BUSY_LATCH: u16 = 0xF0;

    unsafe {
        __outbyte(CPU_IO_o_FPU_BUSY_LATCH, 0);
        _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq07(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(Port: u16) -> u8;
        fn __outbyte(Port: u16, Value: u8);
    }

    unsafe 

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.set_Bits(0);
    Ocw3.set_Sbo(1);
    Ocw3.set_ReadRequest(ReadIsr as u8);
    __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits());

    Isr.set_Bits(__inbyte(PIC1_CONTROL_PORT as u16));

    if Isr.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
}

unsafe fn _HalpDismissIrqLevel(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(port: u16) -> u8;
        fn __outbyte(port: u16, value: u8);
        fn _enable();
    }

    let Pcr = KeGetPcr();
    
    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.Both = (KiI8259MaskTable[Irql as usize] | (*Pcr).IDR) & 0xFFFF;
    __outbyte(PIC1_DATA_PORT as u16, Mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, Mask.__bindgen_anon_1.Slave);
    
    (*Pcr).IRR |= 1 << (Irq + 4);
    
    let CurrentIrql = (*Pcr).Irql;
    
    let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
    Ocw2.__bindgen_anon_1.set_EoiMode(3);
    
    if Irq >= 8 {
        #[cfg(target_feature = "pc98")]
        {
            let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
            let Isr: I8259_ISR;
            
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
            
            Ocw3.__bindgen_anon_1.set_Sbo(1);
            Ocw3.__bindgen_anon_1.set_ReadRequest(3);
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits);
            
            Isr.Bits = __inbyte(PIC2_CONTROL_PORT as u16);
            
            if Isr.Bits == 0 {
                __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
            }
        }
        
        #[cfg(not(target_feature = "pc98"))]
        {
            __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
            __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
        }
    } else {
        __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | (Irq & 0xFF) as u8);
    }
    
    if Irql > CurrentIrql {
        (*Pcr).Irql = Irql;
        *OldIrql = CurrentIrql;
        
        _enable();
        return 1;
    }
    
    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrqLevel(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq15Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(port: u16) -> u8;
        fn __outbyte(port: u16, value: u8);
    }

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.set_Bits(0);
    Ocw3.set_Sbo(1);
    Ocw3.set_ReadRequest(0x3);

    unsafe { __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits()) };

    Isr.set_Bits(unsafe { __inbyte(PIC2_CONTROL_PORT as u16) });

    if Isr.Irq7() == 0 {
        Ocw2.set_Bits(0);
        Ocw2.set_EoiMode(0x1);
        unsafe { __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits() | PIC_CASCADE_IRQ as u8) };

        return 0;
    }

    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq13Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __outbyte(port: u16, value: u8);
    }

    unsafe 

    unsafe {
        __outbyte(0xF0, 0);
        _HalpDismissIrqLevel(Irql, Irq, OldIrql)
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq08Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __outbyte(port: u16, value: u8);
    }

    const CPU_IO_o_FPU_BUSY_LATCH: u16 = 0xF0;

    unsafe {
        __outbyte(CPU_IO_o_FPU_BUSY_LATCH, 0);
    }

    unsafe { _HalpDismissIrqLevel(Irql, Irq, OldIrql) }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDismissIrq07Level(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    unsafe extern "C" {
        fn __inbyte(port: u16) -> u8;
        fn __outbyte(port: u16, value: u8);
    }

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.set_Bits(0);
    Ocw3.set_Sbo(1);
    Ocw3.set_ReadRequest(3);

    unsafe {
        __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits());
    }

    unsafe {
        Isr.set_Bits(__inbyte(PIC1_CONTROL_PORT as u16));
    }

    if Isr.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqLevel(Irql, Irq, OldIrql)
}

#[no_mangle]
pub unsafe extern "C" fn HalpHardwareInterruptLevel2() -> PHAL_SW_INTERRUPT_HANDLER {
    unsafe extern "system" {
        fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
    }

    let pcr: PKPCR = KeGetPcr();
    let mut pending_irql: ULONG = 0;

    let pending_irql_mask = (*pcr).IRR & FindHigherIrqlMask[(*pcr).Irql as usize];
    if pending_irql_mask != 0 {
        if (*pcr).IrrActive & 0xFFFFFFF0 != 0 {
            return core::ptr::null_mut();
        }

        BitScanReverse(&raw mut pending_irql, pending_irql_mask);

        (*pcr).IRR ^= 1 << pending_irql;

        return SWInterruptHandlerTable[pending_irql as usize];
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ULONG,
    Irql: KIRQL,
    InterruptMode: KINTERRUPT_MODE,
) -> BOOLEAN {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
    }

    unsafe extern "system" {
        fn __outbyte(Port: ::core::ffi::c_ushort, Data: ::core::ffi::c_uchar);
        fn _disable();
        fn _enable();
    }

    unsafe extern "C" {
        static mut SWInterruptHandlerTable: [PVOID; 256];
        static mut HalpSpecialDismissTable: [PVOID; 16];
        static mut HalpSpecialDismissLevelTable: [PVOID; 16];
        static mut KiI8259MaskTable: [USHORT; 32];
    }

    let Irq: ULONG;
    let Pcr: PKPCR = KeGetPcr();
    let mut PicMask: PIC_MASK = core::mem::zeroed();

    Irq = Vector - PRIMARY_VECTOR_BASE;
    if Irq >= CLOCK2_LEVEL {
        return 0;
    }

    if InterruptMode == InterruptMode as KINTERRUPT_MODE {
        SWInterruptHandlerTable[(Irq + 4) as usize] = HalpHardwareInterruptLevel as PVOID;

        HalpSpecialDismissTable[Irq as usize] = HalpSpecialDismissLevelTable[Irq as usize];
    }

    _disable();

    (*Pcr).IDR &= !(1 << Irq);

    PicMask.Both = (KiI8259MaskTable[(*Pcr).Irql as usize] | (*Pcr).IDR) & 0xFFFF;
    __outbyte(PIC1_DATA_PORT as u16, PicMask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, PicMask.__bindgen_anon_1.Slave);

    _enable();
    1
}

#[no_mangle]
pub unsafe extern "system" fn HalDisableSystemInterrupt(
    Vector: ULONG,
    Irql: KIRQL,
) {
    unsafe extern "C" {
        fn KeGetPcr() -> PKPCR;
        fn __inbyte(port: u16) -> u8;
        fn __outbyte(port: u16, value: u8);
        fn _disable();
        fn _enable();
    }

    let mut irq_mask: ULONG;
    let mut pic_mask: PIC_MASK;

    if Vector == PRIMARY_VECTOR_BASE + PIC_TIMER_IRQ {
        __outbyte(0xE9, b'!');
        __outbyte(0xE9, b'P');
        __outbyte(0xE9, b'I');
        __outbyte(0xE9, b'T');
        __outbyte(0xE9, b'!');
    }

    irq_mask = 1 << (Vector - PRIMARY_VECTOR_BASE);

    _disable();

    (*KeGetPcr()).IDR |= irq_mask;

    pic_mask = core::mem::zeroed();
    pic_mask.Master = __inbyte(PIC1_DATA_PORT as u16);
    pic_mask.Slave = __inbyte(PIC2_DATA_PORT as u16);

    pic_mask.Both |= irq_mask;

    __outbyte(PIC1_DATA_PORT as u16, pic_mask.Master);
    __outbyte(PIC2_DATA_PORT as u16, pic_mask.Slave);

    _enable();
}

#[no_mangle]
pub unsafe extern "system" fn HalBeginSystemInterrupt(
    Irql: KIRQL,
    Vector: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    let Irq: ULONG;

    Irq = Vector - PRIMARY_VECTOR_BASE;
    unsafe {
        HalpSpecialDismissTable[Irq as usize].unwrap()(Irql, Irq, OldIrql)
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalEndSystemInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;

    unsafe extern "C" {
        fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
    }

    unsafe extern "system" {
        fn __outbyte(Port: USHORT, Data: UCHAR);
    }

    let mut PendingIrql: ULONG;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK;

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return core::ptr::null_mut();
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return core::ptr::null_mut();
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize])();

                (*Pcr).IrrActive ^= PendingIrqMask;

                PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[(*Pcr).Irql as usize];
                if PendingIrqlMask == 0 {
                    break;
                }
            } else {
                return SWInterruptHandlerTable2[PendingIrql as usize];
            }
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalEndSystemInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;

    unsafe extern "C" {
        fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
    }

    unsafe extern "system" {
        fn __outbyte(Port: USHORT, Data: UCHAR);
    }

    let mut PendingIrql: ULONG;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK;

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return core::ptr::null_mut();
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return core::ptr::null_mut();
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize])();

                (*Pcr).IrrActive ^= PendingIrqMask;

                PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[(*Pcr).Irql as usize];
                if PendingIrqlMask == 0 {
                    break;
                }
            } else {
                return SWInterruptHandlerTable2[PendingIrql as usize];
            }
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn _HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    unsafe extern "system" {
        fn HalpEndSoftwareInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME);
    }

    unsafe extern "C" {
        fn _disable();
        fn _enable();
    }

    let Pcr: PKPCR = KeGetPcr();
    
    let CurrentIrql: KIRQL = (*Pcr).Irql;
    (*Pcr).Irql = APC_LEVEL as KIRQL;
    
    (*Pcr).IRR &= !(1 << APC_LEVEL);
    
    _enable();
    KiDeliverApc(
        if KiUserTrap(TrapFrame) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
            1i32
        } else {
            0i32
        },
        core::ptr::null_mut(),
        TrapFrame,
    );
    
    _disable();
    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);
    
    KiEoiHelper(TrapFrame);
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpApcInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe {
        _HalpApcInterruptHandler(TrapFrame);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpApcInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe {
        _HalpApcInterruptHandler(TrapFrame);
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    unsafe extern "C" {
        fn __readeflags() -> u32;
    }

    unsafe 

    (*TrapFrame).EFlags = __readeflags();
    (*TrapFrame).SegCs = KGDT_R0_CODE as u16;
    (*TrapFrame).Eip = (*TrapFrame).Eax;

    KiEnterInterruptTrap(TrapFrame);

    _HalpApcInterruptHandler(TrapFrame);
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    unsafe extern "C" {
        fn __readeflags() -> u64;
    }

    unsafe 

    (*TrapFrame).EFlags = __readeflags() as u32;
    (*TrapFrame).SegCs = KGDT_R0_CODE as u16;
    (*TrapFrame).Eip = (*TrapFrame).Eax as u32;

    KiEnterInterruptTrap(TrapFrame);

    _HalpApcInterruptHandler(TrapFrame);
}

unsafe fn _HalpDispatchInterruptHandler() -> KIRQL {
    unsafe extern "C" {
        fn _enable();
        fn _disable();
    }

    let Pcr = KeGetPcr();

    let CurrentIrql = (*Pcr).Irql;
    (*Pcr).Irql = DISPATCH_LEVEL as KIRQL;

    (*Pcr).IRR &= !(1 << DISPATCH_LEVEL);

    _enable();
    KiDispatchInterrupt();
    _disable();

    CurrentIrql
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpDispatchInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe extern "system" {
        fn HalpEndSoftwareInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME);
    }

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(TrapFrame);
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpDispatchInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {
    unsafe extern "system" {
        fn HalpEndSoftwareInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME);
    }

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(TrapFrame);
}

#[no_mangle]
pub unsafe extern "C" fn HalpDispatchInterrupt2() -> PHAL_SW_INTERRUPT_HANDLER {
    unsafe extern "C" {
        fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
        fn __outbyte(Port: USHORT, Data: UCHAR);
    }

    unsafe extern "C" {
        static FindHigherIrqlMask: [ULONG; 32];
        static SWInterruptHandlerTable: [PHAL_SW_INTERRUPT_HANDLER; 32];
    }

    let Pcr = KeGetPcr();

    let OldIrql = _HalpDispatchInterruptHandler();

    (*Pcr).Irql = OldIrql;

    let PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        let mut PendingIrql: ULONG = 0;
        BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
        if PendingIrql > DISPATCH_LEVEL {
            let mut Mask: PIC_MASK = core::mem::zeroed();
            Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
            __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
            __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

            (*Pcr).IRR ^= 1 << PendingIrql;
        }

        return SWInterruptHandlerTable[PendingIrql as usize];
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetRootInterruptVector(
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    let _ = BusInterruptVector;

    if BusInterruptLevel > 23 {
        return 0;
    }

    #[cfg(target_arch = "x86")]
    let system_vector = unsafe {
        HalpIrqToVector(BusInterruptLevel as UCHAR)
    };

    #[cfg(target_arch = "x86_64")]
    let system_vector = unsafe {
        HalpIrqToVector(BusInterruptLevel as UCHAR)
    };

    #[cfg(target_arch = "x86")]
    unsafe {
        *Irql = HalpVectorToIrql(system_vector);
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        *Irql = HalpVectorToIrql(system_vector);
    }

    unsafe extern "system" {
        static HalpDefaultInterruptAffinity: KAFFINITY;
    }

    unsafe {
        *Affinity = HalpDefaultInterruptAffinity;
    }

    system_vector as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> KIRQL {
    PASSIVE_LEVEL as KIRQL
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn KfLowerIrql(OldIrql: KIRQL) {
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn KfLowerIrql(OldIrql: KIRQL) {
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    NewIrql
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    NewIrql
}
