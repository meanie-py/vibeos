//! Generate the winreg RPC server stub as Rust (no C). See svcctl-stub/build.rs.
fn main() {
    let rs = reactos_link::generate_rpc_stub_rust(
        "sdk/include/reactos/idl/winreg.idl",
        "winreg_s",
        "winlogon_c",
    );
    println!("cargo:rustc-env=WINREG_STUB_RS={}", rs.display());
}
