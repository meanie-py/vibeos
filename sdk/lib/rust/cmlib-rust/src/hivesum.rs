//! Translation of `sdk/lib/cmlib/hivesum.c`.
//!
//! `HvpHiveHeaderChecksum` — XOR-fold the first 127 ULONGs of the hive base
//! block (the header is the first 512-byte sector; the 128th ULONG is the
//! checksum field itself, excluded). The two fix-ups force the result away from
//! the reserved 0 / 0xFFFFFFFF sentinels, exactly as the C does.

/// `ULONG CMAPI HvpHiveHeaderChecksum(PHBASE_BLOCK HiveHeader)`.
///
/// `CMAPI` is `#define CMAPI NTAPI` (cmlib.h) → stdcall on i386
/// (`_HvpHiveHeaderChecksum@4`), cdecl on amd64/arm64 — so the symbol must be
/// `extern "system"` (NOT `extern "C"`), which resolves to the right convention
/// per arch. The C casts `HiveHeader` straight to `PULONG`, so the faithful
/// signature takes the header as `*const u32` (ABI-identical: one pointer arg,
/// `u32` return). Called intra-cmlib by hiveinit.c / hivewrt.c.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HvpHiveHeaderChecksum(HiveHeader: *const u32) -> u32 {
    let buffer = HiveHeader;
    let mut sum: u32 = 0;

    for i in 0..127isize {
        sum ^= *buffer.offset(i);
    }

    if sum == u32::MAX {
        sum = u32::MAX - 1; // (ULONG)-2
    }
    if sum == 0 {
        sum = 1;
    }

    sum
}
