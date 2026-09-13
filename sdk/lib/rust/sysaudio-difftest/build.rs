//! Compile the original C function and extract the shipping Rust function unchanged.
//! Fixtures isolate buffer semantics; these tests do not validate kernel ABI/layout.
use quote::ToTokens;
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let c_path = root.join("drivers/wdm/audio/sysaudio/pin.c");
    let rust_path = root.join("sdk/lib/rust/sysaudio-rust/src/pin.rs");
    let ks_path = root.join("sdk/include/psdk/ks.h");
    for path in [&c_path, &rust_path, &ks_path] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=reference.h");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let c = fs::read_to_string(c_path).unwrap();
    // Fail when the source boundaries change; never substitute a reference reimplementation.
    let start = c
        .find("NTSTATUS\nGetConnectRequest(")
        .expect("original C function start");
    let tail = &c[start..];
    let end = tail
        .find("NTSTATUS\nNTAPI\nDispatchCreateSysAudioPin(")
        .expect("next C function");
    let body = &tail[..end];
    assert_eq!(body.matches("GetConnectRequest(").count(), 1);
    let ks = fs::read_to_string(ks_path).unwrap();
    let literal = ks
        .lines()
        .find_map(|line| line.strip_prefix("#define KSSTRING_Pin L"))
        .unwrap();
    let source = format!(
        "#include \"reference.h\"\nstatic const WCHAR KSSTRING_Pin[] = u{literal};\n{body}"
    );
    fs::write(out.join("reference.c"), source).unwrap();
    cc::Build::new()
        .file(out.join("reference.c"))
        .include(".")
        .flag_if_supported("-std=c11")
        .define("GetConnectRequest", "ref_GetConnectRequest")
        .compile("sysaudio_reference");

    let tree = syn::parse_file(&fs::read_to_string(rust_path).unwrap()).unwrap();
    let mut found = 0;
    let mut rust = String::new();
    for item in tree.items {
        match item {
            syn::Item::Fn(mut function) if function.sig.ident == "GetConnectRequest" => {
                function.attrs.clear(); // prevent a process-global exported test symbol
                rust.push_str(&function.into_token_stream().to_string());
                found += 1;
            }
            syn::Item::Const(value) if value.ident == "KSSTRING_Pin" => {
                rust.push_str(&value.into_token_stream().to_string());
                found += 1;
            }
            _ => {}
        }
    }
    assert_eq!(
        found, 2,
        "shipping function and UTF-16 constant must both exist"
    );
    fs::write(out.join("shipping.rs"), rust).unwrap();
}
