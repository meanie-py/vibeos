//! IRQL software-interrupt state machine. Portable; takes a hardware
//! backend as a trait so x86 (8259/LAPIC) and a future arm64 (GIC)
//! can share the same logic. Empty until first translation lands.
