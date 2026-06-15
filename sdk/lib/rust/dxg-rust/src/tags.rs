//! Pool tags — translated from win32ss/reactx/dxg/tags.h.
//!
//! In C these are multi-character constants (`'ddht'`), which the compiler
//! evaluates with the first character in the most-significant byte:
//! `'abcd'` == `(a<<24)|(b<<16)|(c<<8)|d`. EngAllocMem takes the tag as a
//! ULONG, so we reproduce the exact same integer values here.

/// `'ddht'` → 0x64646874
pub const TAG_THDD: u32 = 0x64646874;
/// `'iniG'` → 0x696E6947
pub const TAG_GINI: u32 = 0x696E6947;
/// `'vddG'` → 0x76646447
pub const TAG_GDDV: u32 = 0x76646447;
/// `'fddG'` → 0x66646447
pub const TAG_GDDF: u32 = 0x66646447;
/// `'pddG'` → 0x70646447
pub const TAG_GDDP: u32 = 0x70646447;
/// `'0 hD'` → 0x30206844
pub const TAG_DH_0: u32 = 0x30206844;
