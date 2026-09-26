/* Cohabitation placeholder TU for the uniata.sys kernelmodedriver MODULE.
 *
 * Nothing cohabits any more: the UniATA IDE/SATA/AHCI miniport is 100% Rust
 * (sdk/lib/rust/uniata-rust, linked as libuniata_rust.a) and the 36,279 lines of
 * C++ it was translated from were DELETED on 2026-09-20 -- every file claimed a
 * copyright (Telyatnikov, VorontSOFT, Microsoft) and granted nothing, carrying
 * the BSD warranty disclaimer without the redistribution clause, and id_ata.cpp
 * said outright that its skeleton came from the NT4 DDK's ATAPI.SYS. The full
 * argument is the PROVENANCE note at the top of uniata-rust/src/lib.rs.
 *
 * DO NOT restore the C to fix a build. There is no C fallback by design.
 *
 * This TU survives only because CMake wants a C/C++ source in the MODULE beside
 * the static archive; rust_varargs_shim.c is the other one, and it is ours.
 */
typedef int uniata_rust_cohab_placeholder;
