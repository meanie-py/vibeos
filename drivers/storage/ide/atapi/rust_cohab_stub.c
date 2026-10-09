/* Cohabitation placeholder TU for the atapi.sys kernelmodedriver MODULE.
 *
 * atapi.sys is upstream's 2026 ATA port driver (atapi/fdo/pdo/enum/satl/scsi/ioctl/
 * portstate/dev_xxx/smart/wmi.c), translated to Rust in sdk/lib/rust/atapi-rust and linked
 * as libatapi_rust.a. The .c files stay on disk, out of the build. DriverEntry and every
 * dispatch routine come from the Rust; this stub is the only C the MODULE compiles.
 *
 * (The previous atapi.sys was the NT4-DDK SCSIPORT miniport. Its C was deleted on
 * 2026-09-20 under the DDK EULA; its Rust is parked as sdk/lib/rust/atapi-legacy-rust.)
 */
typedef int atapi_rust_cohab_placeholder;
