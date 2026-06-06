//! Generate the svcctl RPC server stub as Rust (no C). Replays cmake's widl
//! codegen + reads the exact per-arch symbol decorations, then transliterates
//! to Rust via `tools/widl_to_rust.py` against `midl-rt`. The emitted `.rs` is
//! `include!`d by `src/lib.rs`.
fn main() {
    let rs = reactos_link::generate_rpc_stub_rust(
        "sdk/include/reactos/idl/svcctl.idl",
        "svcctl_s",
        "services_c",
    );
    println!("cargo:rustc-env=SVCCTL_STUB_RS={}", rs.display());
}
