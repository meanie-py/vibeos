/* Cohabitation placeholder TU for the usbaudio kernelmodedriver MODULE.
 * The USB Audio class driver is translated to Rust (sdk/lib/rust/usbaudio-rust,
 * libusbaudio_rust.a; DeepSeek-chat V3 + c2rust-anchored, constant-safeguard-verified).
 * The 4 .c bodies (usbaudio/pool/filter/pin) are replaced by Rust and kept on disk per
 * the never-delete rule. guid.c stays compiled as a data-only TU (DEFINE_GUID). The KS/
 * PortCls dispatch tables + #[no_mangle] DriverEntry/AddDevice in the rlib take over the
 * link. usbaudio is a KS/PortCls miniport (imports usbd + ks).
 * Bar: link-clean x3 (NOT boot-loaded in smoke -- the audio stack isn't exercised; this
 * driver was previously excluded from the build entirely and is re-enabled by this flip).
 */
typedef int usbaudio_rust_cohab_placeholder;
