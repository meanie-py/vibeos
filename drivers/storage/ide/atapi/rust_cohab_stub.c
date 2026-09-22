/* Cohabitation placeholder TU for the atapi.sys kernelmodedriver MODULE.
 *
 * The NT4-DDK ATAPI IDE miniport (atapi.c — DriverEntry, the SCSIPORT miniport
 * callbacks AtapiHwInitialize/AtapiStartIo/AtapiInterrupt/AtapiFindController +
 * the IDE/ATAPI command machinery) is translated to Rust
 * (sdk/lib/rust/atapi-rust, linked as libatapi_rust.a). atapi.c is GONE from the
 * tree as of 2026-09-20 — not merely dropped from the SOURCES. It was a Microsoft
 * NT4 DDK sample under the DDK EULA, which is not a FLOSS licence, and it was the
 * only such grant in the repository. The never-delete rule protects translated
 * Rust from being reverted to C; it does not require keeping C we may not
 * redistribute. See sdk/lib/rust/atapi-rust/src/lib.rs for the provenance note.
 *
 * atapi.sys is a SCSIPORT miniport: it imports ScsiPort* (now scsiport-rust) and
 * registers via ScsiPortInitialize. It is NOT in txtsetup.sif (the live IDE path
 * is uniata.sys), so it is not boot-loaded — link-clean ×3 is the verification
 * bar. The translation was done BY HAND (5-way parallel), not DeepSeek, because a
 * link-only-verified driver gets no runtime smoke coverage, so hand fidelity wins.
 *
 * Driver-private structs (IDE/IDENTIFY registers, HW_DEVICE_EXTENSION, the SCSI
 * SRB/CDB/sense DDK layouts) come from the per-driver bindgen substrate
 * (atapi-rust/src/types_*.rs). DriverEntry is the only export the link resolves.
 */
typedef int atapi_rust_cohab_placeholder;
