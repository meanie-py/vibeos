// i386idt.c — IDT setup for FreeLDR (i386). Globals i386Idt and
// i386IdtDescriptor are referenced from arch/i386/entry.S and uefiasm.S
// (`EXTERN _i386Idt:DWORD`, `call _InitIdt`). They both inline `_i386Idt`
// into their own GDT/IDT descriptor; `_i386IdtDescriptor` itself has no
// external consumer in the codebase but the C source defined it, so we
// preserve it (Base is patched at InitIdt() since pointer-to-int cast
// is not const-stable in Rust).
use core::ffi::c_void;
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{KIDTENTRY, KDESCRIPTOR};

// x86common.h: #define PMODE_CS HEX(08) — pmode code selector.
pub const PMODE_CS: USHORT = 0x08;

// Trap handlers live in arch/i386/i386trap.S (`TRAP_STUB _i386<Name>`).
extern "C" {
    fn i386DivideByZero();
    fn i386DebugException();
    fn i386NMIException();
    fn i386Breakpoint();
    fn i386Overflow();
    fn i386BoundException();
    fn i386InvalidOpcode();
    fn i386FPUNotAvailable();
    fn i386DoubleFault();
    fn i386CoprocessorSegment();
    fn i386InvalidTSS();
    fn i386SegmentNotPresent();
    fn i386StackException();
    fn i386GeneralProtectionFault();
    fn i386PageFault();
    fn i386CoprocessorError();
    fn i386AlignmentCheck();
    fn i386MachineCheck();
}

// `KIDTENTRY DECLSPEC_ALIGN(4) i386Idt[32];` — exported by C name.
// KIDTENTRY is 8 bytes (USHORT Offset/Selector/Access/ExtendedOffset),
// naturally 2-byte aligned; the original C bumps it to 4. The array
// itself is what entry.S/uefiasm.S references — match the symbol shape
// (raw `[KIDTENTRY; 32]`, no wrapper struct) so the linker-visible
// layout is identical to the C definition.
#[no_mangle]
pub static mut i386Idt: [KIDTENTRY; 32] = [KIDTENTRY {
    Offset: 0,
    Selector: 0,
    Access: 0,
    ExtendedOffset: 0,
}; 32];

// `KDESCRIPTOR i386IdtDescriptor = {0, 255, (ULONG)i386Idt};`
// Pointer-to-int cast is not const-stable, so Base starts at 0 and is
// patched at InitIdt() runtime — matches the C value once initialized.
// No external consumer of this symbol (entry.S/uefiasm.S inline `_i386Idt`
// into their own descriptors) but the C source defined it.
#[no_mangle]
pub static mut i386IdtDescriptor: KDESCRIPTOR = KDESCRIPTOR {
    Pad: 0,
    Limit: 255,
    Base: 0,
};

unsafe fn InitIdtVector(
    Vector: UCHAR,
    ServiceHandler: PVOID,
    Access: USHORT)
{
    let slot = &mut i386Idt[Vector as usize];
    slot.Offset = ((ServiceHandler as ULONG) & 0xffff) as USHORT;
    slot.ExtendedOffset = ((ServiceHandler as ULONG) >> 16) as USHORT;
    slot.Selector = PMODE_CS;
    slot.Access = Access;
}

#[no_mangle]
pub unsafe extern "C" fn InitIdt() {
    // Patch the descriptor Base now that i386Idt has a real address.
    i386IdtDescriptor.Base = core::ptr::addr_of_mut!(i386Idt) as ULONG;

    InitIdtVector(0, i386DivideByZero as PVOID, 0x8e00);
    InitIdtVector(1, i386DebugException as PVOID, 0x8e00);
    InitIdtVector(2, i386NMIException as PVOID, 0x8e00);
    InitIdtVector(3, i386Breakpoint as PVOID, 0x8e00);
    InitIdtVector(4, i386Overflow as PVOID, 0x8e00);
    InitIdtVector(5, i386BoundException as PVOID, 0x8e00);
    InitIdtVector(6, i386InvalidOpcode as PVOID, 0x8e00);
    InitIdtVector(7, i386FPUNotAvailable as PVOID, 0x8e00);
    InitIdtVector(8, i386DoubleFault as PVOID, 0x8e00);
    InitIdtVector(9, i386CoprocessorSegment as PVOID, 0x8e00);
    InitIdtVector(10, i386InvalidTSS as PVOID, 0x8e00);
    InitIdtVector(11, i386SegmentNotPresent as PVOID, 0x8e00);
    InitIdtVector(12, i386StackException as PVOID, 0x8e00);
    InitIdtVector(13, i386GeneralProtectionFault as PVOID, 0x8e00);
    InitIdtVector(14, i386PageFault as PVOID, 0x8e00);
    InitIdtVector(16, i386CoprocessorError as PVOID, 0x8e00);
    InitIdtVector(17, i386AlignmentCheck as PVOID, 0x8e00);
    InitIdtVector(18, i386MachineCheck as PVOID, 0x8e00);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// #include <freeldr.h>
//
//
// KIDTENTRY DECLSPEC_ALIGN(4) i386Idt[32];
// KDESCRIPTOR i386IdtDescriptor = {0, 255, (ULONG)i386Idt};
//
// static
// void
// InitIdtVector(
//     UCHAR Vector,
//     PVOID ServiceHandler,
//     USHORT Access)
// {
//     i386Idt[Vector].Offset = (ULONG)ServiceHandler & 0xffff;
//     i386Idt[Vector].ExtendedOffset = (ULONG)ServiceHandler >> 16;
//     i386Idt[Vector].Selector = PMODE_CS;
//     i386Idt[Vector].Access = Access;
// }
//
// void
// __cdecl
// InitIdt(void)
// {
//     InitIdtVector(0, i386DivideByZero, 0x8e00);
//     InitIdtVector(1, i386DebugException, 0x8e00);
//     InitIdtVector(2, i386NMIException, 0x8e00);
//     InitIdtVector(3, i386Breakpoint, 0x8e00);
//     InitIdtVector(4, i386Overflow, 0x8e00);
//     InitIdtVector(5, i386BoundException, 0x8e00);
//     InitIdtVector(6, i386InvalidOpcode, 0x8e00);
//     InitIdtVector(7, i386FPUNotAvailable, 0x8e00);
//     InitIdtVector(8, i386DoubleFault, 0x8e00);
//     InitIdtVector(9, i386CoprocessorSegment, 0x8e00);
//     InitIdtVector(10, i386InvalidTSS, 0x8e00);
//     InitIdtVector(11, i386SegmentNotPresent, 0x8e00);
//     InitIdtVector(12, i386StackException, 0x8e00);
//     InitIdtVector(13, i386GeneralProtectionFault, 0x8e00);
//     InitIdtVector(14, i386PageFault, 0x8e00);
//     InitIdtVector(16, i386CoprocessorError, 0x8e00);
//     InitIdtVector(17, i386AlignmentCheck, 0x8e00);
//     InitIdtVector(18, i386MachineCheck, 0x8e00);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
