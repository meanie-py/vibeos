//! ARM3 ARM64 init code — translated from ARM3/arm/init.c.

pub mod init;
// kdserial.rs (PL011) is for arm32; arm64 has its own arm64/kdserial.rs
// that defines the same KdPortGetByteEx/KdPortPutByteEx symbols.
#[cfg(target_arch = "arm")]
pub mod kdserial;
