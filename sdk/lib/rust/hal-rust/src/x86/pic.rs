//! Generic 8259 PIC initialization. Programs both master and slave
//! 8259 chips with cascade mode, edge-triggered, vector base 0x30,
//! 8086 mode, normal EOI, all interrupts masked.
//!
//! Translated from `hal/halx86/generic/pic.c`. Shared between PIC
//! and APIC HAL flavours (both must initialize the legacy 8259s
//! before deciding what to do with them).

use ntkrnl_sys::hal_ddk::{
    EFLAGS_INTERRUPT_MASK, PIC1_CONTROL_PORT, PIC1_DATA_PORT, PIC2_CONTROL_PORT,
    PIC2_DATA_PORT, PRIMARY_VECTOR_BASE,
    _I8259_ICW1, _I8259_ICW2, _I8259_ICW3, _I8259_ICW4,
    _I8259_ICW1_OPERATING_MODE_Cascade as ICW1_CASCADE,
    _I8259_ICW1_INTERRUPT_MODE_EdgeTriggered as ICW1_EDGE_TRIGGERED,
    _I8259_ICW1_INTERVAL_Interval8 as ICW1_INTERVAL_8,
    _I8259_ICW4_SYSTEM_MODE_New8086Mode as ICW4_8086_MODE,
    _I8259_ICW4_EOI_MODE_NormalEoi as ICW4_NORMAL_EOI,
    _I8259_ICW4_BUFFERED_MODE_NonBuffered as ICW4_NON_BUFFERED,
};

/// Write a byte to an x86 port.
#[inline(always)]
unsafe fn outbyte(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

/// Read EFLAGS / RFLAGS via pushf/pop. Mirrors __readeflags().
#[inline(always)]
unsafe fn read_flags() -> usize {
    let f: usize;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("pushfq; pop {}", out(reg) f, options(nostack));
    #[cfg(target_arch = "x86")]
    core::arch::asm!("pushfd; pop {}", out(reg) f, options(nostack));
    f
}

/// HalpInitializeLegacyPICs — program both 8259s with the canonical
/// PC-AT cascade configuration: master vectors `PRIMARY_VECTOR_BASE`
/// (0x30) onward, slave on master's IRQ 2 vectoring at +8, all IRQs
/// initially masked. Called with interrupts disabled.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitializeLegacyPICs() {
    ntkrnl_sys::ASSERT!(unsafe { read_flags() } & EFLAGS_INTERRUPT_MASK as usize == 0);

    // ── master ──
    let mut icw1 = _I8259_ICW1 { Bits: 0 };
    let mut icw2 = _I8259_ICW2 { Bits: 0 };
    let mut icw3 = _I8259_ICW3 { Bits: 0 };
    let mut icw4 = _I8259_ICW4 { Bits: 0 };

    unsafe {
        icw1.__bindgen_anon_1.set_NeedIcw4(1);
        icw1.__bindgen_anon_1.set_OperatingMode(ICW1_CASCADE as u8);
        icw1.__bindgen_anon_1.set_Interval(ICW1_INTERVAL_8 as u8);
        icw1.__bindgen_anon_1.set_InterruptMode(ICW1_EDGE_TRIGGERED as u8);
        icw1.__bindgen_anon_1.set_Init(1);
        icw1.__bindgen_anon_1.set_InterruptVectorAddress(0);
        outbyte(PIC1_CONTROL_PORT as u16, icw1.Bits);

        // ICW2: master vector base (low byte holds the offset).
        icw2.Bits = PRIMARY_VECTOR_BASE as u8;
        outbyte(PIC1_DATA_PORT as u16, icw2.Bits);

        // ICW3: slave is wired to master's IRQ 2.
        icw3.__bindgen_anon_1.__bindgen_anon_1.set_SlaveIrq2(1);
        outbyte(PIC1_DATA_PORT as u16, icw3.Bits);

        icw4.__bindgen_anon_1.set_SystemMode(ICW4_8086_MODE as u8);
        icw4.__bindgen_anon_1.set_EoiMode(ICW4_NORMAL_EOI as u8);
        icw4.__bindgen_anon_1.set_BufferedMode(ICW4_NON_BUFFERED as u8);
        icw4.__bindgen_anon_1.set_SpecialFullyNestedMode(0);
        icw4.__bindgen_anon_1.set_Reserved(0);
        outbyte(PIC1_DATA_PORT as u16, icw4.Bits);

        // Mask all master IRQs.
        outbyte(PIC1_DATA_PORT as u16, 0xFF);
    }

    // ── slave ──
    let mut icw1 = _I8259_ICW1 { Bits: 0 };
    let mut icw2 = _I8259_ICW2 { Bits: 0 };
    let mut icw3 = _I8259_ICW3 { Bits: 0 };
    let mut icw4 = _I8259_ICW4 { Bits: 0 };

    unsafe {
        icw1.__bindgen_anon_1.set_NeedIcw4(1);
        icw1.__bindgen_anon_1.set_InterruptMode(ICW1_EDGE_TRIGGERED as u8);
        icw1.__bindgen_anon_1.set_OperatingMode(ICW1_CASCADE as u8);
        icw1.__bindgen_anon_1.set_Interval(ICW1_INTERVAL_8 as u8);
        icw1.__bindgen_anon_1.set_Init(1);
        icw1.__bindgen_anon_1.set_InterruptVectorAddress(0);
        outbyte(PIC2_CONTROL_PORT as u16, icw1.Bits);

        // ICW2: slave vector base = master + 8.
        icw2.Bits = (PRIMARY_VECTOR_BASE + 8) as u8;
        outbyte(PIC2_DATA_PORT as u16, icw2.Bits);

        // ICW3: this is the slave; tell it its slave-id is 2.
        icw3.__bindgen_anon_1.__bindgen_anon_2.set_SlaveId(2);
        outbyte(PIC2_DATA_PORT as u16, icw3.Bits);

        icw4.__bindgen_anon_1.set_SystemMode(ICW4_8086_MODE as u8);
        icw4.__bindgen_anon_1.set_EoiMode(ICW4_NORMAL_EOI as u8);
        icw4.__bindgen_anon_1.set_BufferedMode(ICW4_NON_BUFFERED as u8);
        icw4.__bindgen_anon_1.set_SpecialFullyNestedMode(0);
        icw4.__bindgen_anon_1.set_Reserved(0);
        outbyte(PIC2_DATA_PORT as u16, icw4.Bits);

        // Mask all slave IRQs.
        outbyte(PIC2_DATA_PORT as u16, 0xFF);
    }
}
