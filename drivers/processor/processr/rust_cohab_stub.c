/*
 * processr.sys Rust cohabitation stub.
 *
 * processr.c / misc.c / pnp.c (DriverEntry + ForwardIrpAndForget + ProcessorPnp
 * + ProcessorAddDevice) are translated to Rust (sdk/lib/rust/processr-rust, kept
 * on disk per the never-delete rule). The MODULE link needs at least one C TU;
 * this placeholder supplies it while the real code comes from libprocessr_rust.a.
 */
typedef int processr_rust_cohab_placeholder;
