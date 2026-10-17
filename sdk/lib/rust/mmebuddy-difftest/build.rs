//! Compile the genuine DoWaveStreaming + CompleteIO for the host and extract the shipping
//! Rust bodies (and the sealed wide strings they trace with) unchanged.
use quote::ToTokens;
use std::{env, fs, path::PathBuf};

const C_FILE: &str = "sdk/lib/drivers/sound/mmebuddy/wave/streaming.c";
const C_START: &str = "VOID\nDoWaveStreaming(";
const C_NEXT: &str = "MMRESULT\nWriteFileEx_Committer(";
const RUST_FILE: &str = "sdk/lib/rust/mmebuddy-rust/src/mmebuddy/streaming.rs";
const RUST_FNS: [&str; 3] = ["DoWaveStreaming", "CompleteIO", "InitializeWaveHeaderLoops"];

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let (c_path, rust_path) = (root.join(C_FILE), root.join(RUST_FILE));
    for path in [c_path.display().to_string(), rust_path.display().to_string(), "streaming.h".into()] {
        println!("cargo:rerun-if-changed={path}");
    }
    let c = fs::read_to_string(&c_path).unwrap();
    // Fail when the source boundaries move; never substitute a reimplementation.
    let tail = &c[c.find(C_START).expect("original C function start")..];
    let body = &tail[..tail.find(C_NEXT).expect("next C function")];
    assert_eq!(body.matches("\nCompleteIO(").count(), 1);
    let reference = out.join("streaming.c");
    fs::write(&reference, format!("#include \"streaming.h\"\n{body}")).unwrap();
    cc::Build::new()
        .file(&reference)
        .include(".")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .compile("streaming_reference");

    let tree = syn::parse_file(&fs::read_to_string(&rust_path).unwrap()).unwrap();
    let mut rust = String::new();
    let mut found = 0;
    for item in tree.items {
        match item {
            syn::Item::Fn(mut function) if RUST_FNS.contains(&&*function.sig.ident.to_string()) => {
                function.attrs.clear(); // no process-global exported test symbol
                rust.push_str(&function.into_token_stream().to_string());
                found += 1;
            }
            syn::Item::Static(s) if s.ident.to_string().starts_with("RUSTCONV_WSTR_") => {
                rust.push_str(&s.into_token_stream().to_string());
            }
            _ => {}
        }
    }
    assert_eq!(found, RUST_FNS.len(), "every shipping streaming function must exist");
    fs::write(out.join("streaming_shipping.rs"), rust).unwrap();
}
