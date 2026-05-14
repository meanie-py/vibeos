//! i386 8259 PIC backend — translated from `hal/halx86/pic/pic.c`.
//!
//! Implements IRQL primitives (KeGetCurrentIrql / KfRaiseIrql / KfLowerIrql)
//! plus PIC initialization, software-interrupt request/clear/dispatch, and
//! the per-IRQ Dismiss handlers used by the trap stubs in `pic.S`.

#![cfg(target_arch = "x86")]
#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe, unused_imports, unused_variables, unused_mut)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, KAFFINITY, KIRQL, PVOID, UCHAR, ULONG, USHORT,
    PKAFFINITY, PKIRQL, PKPCR, PKTRAP_FRAME,
    EISA_ELCR, I8259_ISR, I8259_OCW2, I8259_OCW3, PIC_MASK,
    PHAL_DISMISS_INTERRUPT, PHAL_SW_INTERRUPT_HANDLER, PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY,
    _KINTERRUPT_MODE as KINTERRUPT_MODE,
    _KINTERRUPT_MODE_Latched, _KINTERRUPT_MODE_LevelSensitive,
};
use ntkrnl_sys::ddk::{KeBugCheck, KeBugCheckEx, KiDeliverApc, KiDispatchInterrupt, KiEnterInterruptTrap, KiEoiHelper, KiUserTrap};
use core::ffi::c_void;

// IRQL levels (from sdk/include/ndk/ketypes.h, FORCEINLINE in irql.h).
// These are #defines in C — bindgen doesn't expose them.
pub const PASSIVE_LEVEL: KIRQL  = 0;
pub const APC_LEVEL: KIRQL      = 1;
pub const DISPATCH_LEVEL: KIRQL = 2;
pub const CMCI_LEVEL: KIRQL     = 5;
pub const CLOCK1_LEVEL: KIRQL   = 28;
pub const CLOCK2_LEVEL: KIRQL   = 28;
pub const IPI_LEVEL: KIRQL      = 29;
pub const POWER_LEVEL: KIRQL    = 30;
pub const PROFILE_LEVEL: KIRQL  = 27;
pub const SYNCH_LEVEL: KIRQL    = DISPATCH_LEVEL;
pub const HIGH_LEVEL: KIRQL     = 31;

// EFLAGS bits.
const EFLAGS_INTERRUPT_MASK: ULONG = 0x200;
const EFLAGS_V86_MASK: ULONG       = 0x20000;

// 8259 PIC port-mapped I/O.
const PIC1_CONTROL_PORT: USHORT = 0x20;
const PIC1_DATA_PORT:    USHORT = 0x21;
const PIC2_CONTROL_PORT: USHORT = 0xA0;
const PIC2_DATA_PORT:    USHORT = 0xA1;
const EISA_ELCR_MASTER:  USHORT = 0x4D0;
const EISA_ELCR_SLAVE:   USHORT = 0x4D1;

// IRQ wiring constants.
const PIC_TIMER_IRQ:   ULONG = 0;
const PIC_CASCADE_IRQ: ULONG = 2;
const PRIMARY_VECTOR_BASE: ULONG = 0x30;

// IDT_INTERNAL is one of the InterruptType constants from KE — bindgen-side
// it lives as _KINTERRUPT_TYPE_; the C header defines IDT_INTERNAL=2.
const IDT_INTERNAL: ULONG = 2;
// _KGDT_R0_CODE selector value (0x08).
const KGDT_R0_CODE: USHORT = 0x08;

// I8259_READ_REQUEST enum values from halhw.h.
const ReadIdr: UCHAR = 2;
const ReadIsr: UCHAR = 3;

// PIC bugcheck codes used by KfRaiseIrql / KfLowerIrql in DBG builds.
const IRQL_NOT_GREATER_OR_EQUAL: ULONG = 0x0000_0009;
const IRQL_NOT_LESS_OR_EQUAL:    ULONG = 0x0000_000A;

// Tables defined in irql.rs (HAL component) — read-only at runtime.
unsafe extern "C" {
    static KiI8259MaskTable: [USHORT; 32];
    static FindHigherIrqlMask: [ULONG; 32];
}

// Hand-off tables — these are mutable globals defined in pic.c (today).
// While pic.c still owns these, declare them as externs.  When this file
// takes over the C build, the definitions move here.
unsafe extern "C" {
    static mut HalpEisaELCR: USHORT;
    static mut SWInterruptHandlerTable: [PHAL_SW_INTERRUPT_HANDLER; 20];
    static mut SWInterruptHandlerTable2: [PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY; 3];
    static mut SWInterruptLookUpTable: [KIRQL; 8];
    static mut HalpSpecialDismissTable: [PHAL_DISMISS_INTERRUPT; 16];
    static HalpSpecialDismissLevelTable: [PHAL_DISMISS_INTERRUPT; 16];
    static mut HalpDefaultInterruptAffinity: KAFFINITY;
}

// MSVC intrinsics (lowered by clang into inline asm).  Declared here so each
// fn that uses them doesn't have to re-import.
unsafe extern "C" {
    fn __readeflags() -> ULONG;
    fn __writeeflags(eflags: ULONG);
    fn __inbyte(Port: USHORT) -> UCHAR;
    fn __outbyte(Port: USHORT, Value: UCHAR);
    fn _disable();
    fn _enable();
    fn BitScanReverse(Index: *mut ULONG, Mask: ULONG) -> UCHAR;
    fn KeGetPcr() -> PKPCR;
}

// HAL registration helper — lives in pic_halinit.rs.
unsafe extern "system" {
    fn HalpRegisterVector(Flags: ULONG, BusVector: ULONG, SystemVector: ULONG, Irql: KIRQL);
    fn HalpInitializeLegacyPICs();
}

unsafe extern "fastcall" {
    fn HalpEndSoftwareInterrupt(OldIrql: KIRQL, TrapFrame: PKTRAP_FRAME);
}

// SW interrupt handlers and Dismiss-IRQ functions exposed by pic.S +
// other files — pulled in via spec linkage.  These are PHAL_SW_INTERRUPT_HANDLER
// = Option<unsafe extern "C" fn()>; declare with matching ABI so they can go
// straight into the SWInterruptHandlerTable.
unsafe extern "C" {
    fn HalpApcInterrupt();
    fn HalpDispatchInterrupt();
    fn HalpHardwareInterruptLevel();
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitializePICs(EnableInterrupts: BOOLEAN) {

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
        elcr_found = ((elcr.__bindgen_anon_1.Master.Irq0Level() == 0)
            && (elcr.__bindgen_anon_1.Master.Irq1Level() == 0)
            && (elcr.__bindgen_anon_1.Master.Irq2Level() == 0)
            && (elcr.__bindgen_anon_1.Slave.Irq8Level() == 0)
            && (elcr.__bindgen_anon_1.Slave.Irq13Level() == 0)) as BOOLEAN;
    }

    #[cfg(target_feature = "pc98")]
    {
        if elcr.Bits == 0xFFFF {
            elcr.__bindgen_anon_1.Master.set_Irq0Level(0);
            elcr.__bindgen_anon_1.Master.set_Irq1Level(0);
            elcr.__bindgen_anon_1.Master.set_Irq7Level(0);
            elcr.__bindgen_anon_1.Slave.set_Irq8Level(0);
        }
        elcr_found = 1;
    }

    if elcr_found != 0 {
        HalpEisaELCR = elcr.Bits;

        let mut i: ULONG = 1;
        let mut j: ULONG = 0;
        while j < 16 {
            if (HalpEisaELCR as ULONG & i) != 0 {
                SWInterruptHandlerTable[(j + 4) as usize] = Some(HalpHardwareInterruptLevel);

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
    (PRIMARY_VECTOR_BASE + Irq as ULONG) as UCHAR
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpIrqToVector(Irq: UCHAR) -> UCHAR {
    (PRIMARY_VECTOR_BASE + Irq as ULONG) as UCHAR
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR)
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrq(Vector: UCHAR) -> UCHAR {
    Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR)
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    PROFILE_LEVEL.wrapping_sub(Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR) as KIRQL)
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpVectorToIrql(Vector: u8) -> KIRQL {
    PROFILE_LEVEL.wrapping_sub(Vector.wrapping_sub(PRIMARY_VECTOR_BASE as UCHAR) as KIRQL)
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    
    let pcr = unsafe { KeGetPcr() };
    let current_irql: KIRQL;

    current_irql = unsafe { (*pcr).Irql };
    unsafe { (*pcr).Irql = DISPATCH_LEVEL as KIRQL };

    current_irql
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseIrqlToSynchLevel() -> KIRQL {
    
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
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG;
    let mut Mask: PIC_MASK = unsafe { core::mem::zeroed() };

    unsafe { (*Pcr).Irql = OldIrql };

    loop {
        PendingIrqlMask = unsafe { (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize] };
        if PendingIrqlMask == 0 {
            return None;
        }

        if unsafe { (*Pcr).IrrActive & 0xFFFFFFF0 != 0 } {
            return None;
        }

        unsafe { BitScanReverse(&raw mut PendingIrql, PendingIrqlMask) };
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            Mask.Both = unsafe { ((*Pcr).IDR & 0xFFFF) as USHORT };
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

unsafe fn _HalpDismissIrqGeneric(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    

    let Pcr = KeGetPcr();

    let CurrentIrql = (*Pcr).Irql;

    if Irql > CurrentIrql {
        (*Pcr).Irql = Irql;
        *OldIrql = CurrentIrql;

        let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
        Ocw2.Bits = (0);
        Ocw2.__bindgen_anon_1.set_EoiMode(3);

        if Irq >= 8 {
            #[cfg(not(target_feature = "pc98"))]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);
                __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
            }

            #[cfg(target_feature = "pc98")]
            {
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw2.Bits | ((Irq - 8) & 0xFF) as u8);

                let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
                Ocw3.Bits = (0);
                Ocw3.__bindgen_anon_1.set_Sbo(1);
                Ocw3.__bindgen_anon_1.set_ReadRequest(3);
                __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits);

                let mut Isr: I8259_ISR = core::mem::zeroed();
                Isr.Bits = (__inbyte(PIC2_CONTROL_PORT as u16));

                if Isr.Bits == 0 {
                    __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8);
                }
            }
        } else {
            __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | (Irq & 0xFF) as u8);
        }

        _enable();
        return 1;
    }

    (*Pcr).IRR |= 1 << (Irq + 4);

    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.Both = ((KiI8259MaskTable[CurrentIrql as usize] | (*Pcr).IDR as u16) & 0xFFFF);
    __outbyte(PIC1_DATA_PORT as u16, Mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, Mask.__bindgen_anon_1.Slave);

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

    let mut ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut isr: I8259_ISR;

    ocw3.Bits = (0);
    ocw3.__bindgen_anon_1.set_Sbo(1);
    ocw3.__bindgen_anon_1.set_ReadRequest(3);
    __outbyte(PIC2_CONTROL_PORT as u16, ocw3.Bits);

    isr = core::mem::zeroed();
    isr.Bits = (__inbyte(PIC2_CONTROL_PORT as u16));

    if isr.__bindgen_anon_1.Irq7() == 0 {
        ocw2.Bits = (0);
        ocw2.__bindgen_anon_1.set_EoiMode(1);
        __outbyte(PIC1_CONTROL_PORT as u16, ocw2.Bits | PIC_CASCADE_IRQ as u8);

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


    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(ReadIsr as u8);
    __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits);

    Isr.Bits = (__inbyte(PIC1_CONTROL_PORT as u16));

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqGeneric(Irql, Irq, OldIrql)
}

unsafe fn _HalpDismissIrqLevel(
    Irql: KIRQL,
    Irq: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {

    let Pcr = KeGetPcr();
    
    let mut Mask: PIC_MASK = core::mem::zeroed();
    Mask.Both = ((KiI8259MaskTable[Irql as usize] as ULONG | (*Pcr).IDR) & 0xFFFF) as USHORT;
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

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Ocw2: I8259_OCW2 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(0x3);

    unsafe { __outbyte(PIC2_CONTROL_PORT as u16, Ocw3.Bits) };

    Isr.Bits = (unsafe { __inbyte(PIC2_CONTROL_PORT as u16) });

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        Ocw2.Bits = (0);
        Ocw2.__bindgen_anon_1.set_EoiMode(0x1);
        unsafe { __outbyte(PIC1_CONTROL_PORT as u16, Ocw2.Bits | PIC_CASCADE_IRQ as u8) };

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

    let mut Ocw3: I8259_OCW3 = core::mem::zeroed();
    let mut Isr: I8259_ISR = core::mem::zeroed();

    Ocw3.Bits = (0);
    Ocw3.__bindgen_anon_1.set_Sbo(1);
    Ocw3.__bindgen_anon_1.set_ReadRequest(3);

    unsafe {
        __outbyte(PIC1_CONTROL_PORT as u16, Ocw3.Bits);
    }

    unsafe {
        Isr.Bits = (__inbyte(PIC1_CONTROL_PORT as u16));
    }

    if Isr.__bindgen_anon_1.Irq7() == 0 {
        return 0;
    }

    _HalpDismissIrqLevel(Irql, Irq, OldIrql)
}

#[no_mangle]
pub unsafe extern "C" fn HalpHardwareInterruptLevel2() -> PHAL_SW_INTERRUPT_HANDLER {

    let pcr: PKPCR = KeGetPcr();
    let mut pending_irql: ULONG = 0;

    let pending_irql_mask = (*pcr).IRR & FindHigherIrqlMask[(*pcr).Irql as usize];
    if pending_irql_mask != 0 {
        if (*pcr).IrrActive & 0xFFFFFFF0 != 0 {
            return None;
        }

        BitScanReverse(&raw mut pending_irql, pending_irql_mask);

        (*pcr).IRR ^= 1 << pending_irql;

        return SWInterruptHandlerTable[pending_irql as usize];
    }

    None
}

#[no_mangle]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ULONG,
    Irql: KIRQL,
    InterruptMode: KINTERRUPT_MODE,
) -> BOOLEAN {



    let Irq: ULONG;
    let Pcr: PKPCR = KeGetPcr();
    let mut PicMask: PIC_MASK = core::mem::zeroed();

    Irq = Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;
    if Irq >= CLOCK2_LEVEL as ULONG {
        return 0;
    }

    if InterruptMode == _KINTERRUPT_MODE_LevelSensitive {
        SWInterruptHandlerTable[(Irq + 4) as usize] = Some(HalpHardwareInterruptLevel);

        HalpSpecialDismissTable[Irq as usize] = HalpSpecialDismissLevelTable[Irq as usize];
    }

    _disable();

    (*Pcr).IDR &= !(1 << Irq);

    PicMask.Both = ((KiI8259MaskTable[(*Pcr).Irql as usize] as ULONG | (*Pcr).IDR) & 0xFFFF) as USHORT;
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

    let mut irq_mask: ULONG;
    let mut pic_mask: PIC_MASK;

    if (Vector as ULONG) == PRIMARY_VECTOR_BASE + PIC_TIMER_IRQ {
        __outbyte(0xE9, b'!');
        __outbyte(0xE9, b'P');
        __outbyte(0xE9, b'I');
        __outbyte(0xE9, b'T');
        __outbyte(0xE9, b'!');
    }

    irq_mask = 1u32 << Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;

    _disable();

    (*KeGetPcr()).IDR |= irq_mask;

    pic_mask = core::mem::zeroed();
    pic_mask.__bindgen_anon_1.Master = __inbyte(PIC1_DATA_PORT as u16);
    pic_mask.__bindgen_anon_1.Slave = __inbyte(PIC2_DATA_PORT as u16);

    pic_mask.Both |= irq_mask as USHORT;

    __outbyte(PIC1_DATA_PORT as u16, pic_mask.__bindgen_anon_1.Master);
    __outbyte(PIC2_DATA_PORT as u16, pic_mask.__bindgen_anon_1.Slave);

    _enable();
}

#[no_mangle]
pub unsafe extern "system" fn HalBeginSystemInterrupt(
    Irql: KIRQL,
    Vector: ULONG,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    let Irq: ULONG;

    Irq = Vector.wrapping_sub(PRIMARY_VECTOR_BASE) as ULONG;
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



    let mut PendingIrql: ULONG = 0;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG = 0;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK = core::mem::zeroed();

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return None;
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return None;
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize].unwrap())();

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

    None
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalEndSystemInterrupt2(
    OldIrql: KIRQL,
    TrapFrame: PKTRAP_FRAME,
) -> PHAL_SW_INTERRUPT_HANDLER_2ND_ENTRY {
    let _ = TrapFrame;



    let mut PendingIrql: ULONG = 0;
    let mut PendingIrqlMask: ULONG;
    let mut PendingIrqMask: ULONG = 0;
    let Pcr: PKPCR = KeGetPcr();
    let mut Mask: PIC_MASK = core::mem::zeroed();

    (*Pcr).Irql = OldIrql;

    PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        if ((*Pcr).IrrActive & 0xFFFFFFF0) != 0 {
            return None;
        }

        loop {
            BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
            if PendingIrql > DISPATCH_LEVEL as ULONG {
                Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
                __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
                __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

                PendingIrqMask = 1 << PendingIrql;
                if ((*Pcr).IrrActive & PendingIrqMask) != 0 {
                    return None;
                }

                (*Pcr).IrrActive |= PendingIrqMask;
                (*Pcr).IRR ^= PendingIrqMask;

                (SWInterruptHandlerTable[PendingIrql as usize].unwrap())();

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


    let Pcr: PKPCR = KeGetPcr();
    
    let CurrentIrql: KIRQL = (*Pcr).Irql;
    (*Pcr).Irql = APC_LEVEL as KIRQL;
    
    (*Pcr).IRR &= !(1 << APC_LEVEL);
    
    _enable();
    KiDeliverApc(
        (if KiUserTrap(core::mem::transmute(TrapFrame)) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 { 1i8 } else { 0i8 }),
        core::ptr::null_mut(),
        core::mem::transmute(TrapFrame),
    );
    
    _disable();
    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);
    
    KiEoiHelper(core::mem::transmute(TrapFrame));
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


    (*TrapFrame).EFlags = __readeflags();
    (*TrapFrame).SegCs = KGDT_R0_CODE as u32;
    (*TrapFrame).Eip = (*TrapFrame).Eax;

    KiEnterInterruptTrap(core::mem::transmute(TrapFrame));

    _HalpApcInterruptHandler(TrapFrame);
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpApcInterruptHandler(TrapFrame: PKTRAP_FRAME) {


    (*TrapFrame).EFlags = __readeflags() as u32;
    (*TrapFrame).SegCs = KGDT_R0_CODE as u32;
    (*TrapFrame).Eip = (*TrapFrame).Eax as u32;

    KiEnterInterruptTrap(core::mem::transmute(TrapFrame));

    _HalpApcInterruptHandler(TrapFrame);
}

unsafe fn _HalpDispatchInterruptHandler() -> KIRQL {

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

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(core::mem::transmute(TrapFrame));
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpDispatchInterrupt2ndEntry(TrapFrame: PKTRAP_FRAME) {

    let CurrentIrql = _HalpDispatchInterruptHandler();

    HalpEndSoftwareInterrupt(CurrentIrql, TrapFrame);

    KiEoiHelper(core::mem::transmute(TrapFrame));
}

#[no_mangle]
pub unsafe extern "C" fn HalpDispatchInterrupt2() -> PHAL_SW_INTERRUPT_HANDLER {


    let Pcr = KeGetPcr();

    let OldIrql = _HalpDispatchInterruptHandler();

    (*Pcr).Irql = OldIrql;

    let PendingIrqlMask = (*Pcr).IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        let mut PendingIrql: ULONG = 0;
        BitScanReverse(&raw mut PendingIrql, PendingIrqlMask);
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            let mut Mask: PIC_MASK = core::mem::zeroed();
            Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
            __outbyte(PIC1_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Master);
            __outbyte(PIC2_DATA_PORT as USHORT, Mask.__bindgen_anon_1.Slave);

            (*Pcr).IRR ^= 1 << PendingIrql;
        }

        return SWInterruptHandlerTable[PendingIrql as usize];
    }

    None
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


    unsafe {
        *Affinity = HalpDefaultInterruptAffinity;
    }

    system_vector as ULONG
}

/// Returns the current IRQL by reading PCR.Irql.  pic.c:309-315.
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> KIRQL {
    unsafe { (*KeGetPcr()).Irql }
}

/// Raises the IRQL to NewIrql; returns the previous IRQL.  pic.c:374-401.
/// Caller must ensure NewIrql >= CurrentIrql (or bugcheck in DBG builds).
#[no_mangle]
pub unsafe extern "fastcall" fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    let Pcr = unsafe { KeGetPcr() };
    let CurrentIrql = unsafe { (*Pcr).Irql };
    unsafe { (*Pcr).Irql = NewIrql };
    CurrentIrql
}

/// Lowers the IRQL to OldIrql and dispatches any pending software interrupts
/// whose level is above OldIrql.  pic.c:403-455.
#[no_mangle]
pub unsafe extern "fastcall" fn KfLowerIrql(OldIrql: KIRQL) {
    let Pcr = unsafe { KeGetPcr() };

    // Save EFlags and disable interrupts for the IRR/PIC update.
    let EFlags = unsafe { __readeflags() };
    unsafe { _disable() };

    // Drop the IRQL.
    unsafe { (*Pcr).Irql = OldIrql };

    // Are any pending software interrupts higher than the new IRQL?
    let IRR = unsafe { (*Pcr).IRR };
    let PendingIrqlMask = IRR & FindHigherIrqlMask[OldIrql as usize];
    if PendingIrqlMask != 0 {
        let mut PendingIrql: ULONG = 0;
        unsafe { BitScanReverse(&mut PendingIrql, PendingIrqlMask) };

        // If the pending IRQL is above DISPATCH_LEVEL we need to update the
        // hardware PIC mask too (mask off everything at-or-below).
        if PendingIrql > DISPATCH_LEVEL as ULONG {
            let mut Mask: PIC_MASK = unsafe { core::mem::zeroed() };
            Mask.Both = ((*Pcr).IDR & 0xFFFF) as USHORT;
            unsafe { __outbyte(PIC1_DATA_PORT, Mask.__bindgen_anon_1.Master) };
            unsafe { __outbyte(PIC2_DATA_PORT, Mask.__bindgen_anon_1.Slave) };

            // Clear the IRR bit we're about to dispatch.
            unsafe { (*Pcr).IRR ^= 1 << PendingIrql };
        }

        // Dispatch the pending software interrupt.  The handler tail-jumps
        // back into the SW interrupt context.
        if let Some(handler) = SWInterruptHandlerTable[PendingIrql as usize] {
            unsafe { handler() };
        }
    }

    // Restore interrupt state.
    unsafe { __writeeflags(EFlags) };
}
