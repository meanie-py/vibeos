//! ke/i386 subsystem — translated from C.

pub mod abios;
pub mod context;
pub mod cpu;
pub mod exp;
pub mod freeze;
pub mod irqobj;
pub mod kiinit;
pub mod ldt;
pub mod mproc;
pub mod mtrr;
pub mod patpge;
pub mod safecopy;
pub mod thrdini;
pub mod traphdlr;
pub mod usercall;
pub mod v86vdm;
pub mod zeropage_asm;
