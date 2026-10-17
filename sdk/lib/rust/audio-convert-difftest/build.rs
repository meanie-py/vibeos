//! Compile each copy of the two converters for the host twice -- as amd64/arm64 see
//! them and with `_X86_` as i386 does -- and extract the shipping Rust bodies unchanged.
use quote::ToTokens;
use std::{env, fs, path::Path, path::PathBuf};

/// (name, C file, C start, next C function, fixture header, Rust file)
const COPIES: [(&str, &str, &str, &str, &str, &str); 2] = [
    (
        "kmixer",
        "drivers/wdm/audio/filters/kmixer/pin.c",
        "NTSTATUS\nPerformChannelConversion(",
        "NTSTATUS\nNTAPI\nPin_fnDeviceIoControl(",
        "kmixer.h",
        "sdk/lib/rust/kmixer-rust/src/kmixer/pin.rs",
    ),
    (
        "wdmaud",
        "dll/win32/wdmaud.drv/mixer.c",
        "DWORD\nPerformChannelConversion(",
        "VOID\nCALLBACK\nMixerCompletionRoutine(",
        "wdmaud.h",
        "sdk/lib/rust/wdmaud_drv-rust/src/wdmaud_drv/mixer.rs",
    ),
];

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for (name, c_file, start, next, header, rust_file) in COPIES {
        let (c_path, rust_path) = (root.join(c_file), root.join(rust_file));
        for path in [c_path.as_path(), rust_path.as_path(), Path::new(header)] {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        let c = fs::read_to_string(&c_path).unwrap();
        // Fail when the source boundaries move; never substitute a reimplementation.
        let tail = &c[c.find(start).expect("original C function start")..];
        let body = &tail[..tail.find(next).expect("next C function")];
        assert_eq!(body.matches("\nPerformQualityConversion(").count(), 1);
        let reference = out.join(format!("{name}.c"));
        fs::write(&reference, format!("#include \"{header}\"\n{body}")).unwrap();
        for (variant, x86) in [("", false), ("x86_", true)] {
            let prefix = format!("ref_{name}_{variant}");
            let mut build = cc::Build::new();
            build.file(&reference).include(".").flag_if_supported("-std=c11");
            for function in ["PerformChannelConversion", "PerformQualityConversion"] {
                build.define(function, format!("{prefix}{function}").as_str());
            }
            if x86 {
                build.define("_X86_", None);
            }
            build.compile(&format!("{prefix}reference"));
        }

        let tree = syn::parse_file(&fs::read_to_string(&rust_path).unwrap()).unwrap();
        let mut rust = String::new();
        let mut found = 0;
        for item in tree.items {
            if let syn::Item::Fn(mut function) = item {
                let ident = function.sig.ident.to_string();
                if ident == "PerformChannelConversion" || ident == "PerformQualityConversion" {
                    function.attrs.clear(); // no process-global exported test symbol
                    rust.push_str(&function.into_token_stream().to_string());
                    found += 1;
                }
            }
        }
        assert_eq!(found, 2, "{name}: both shipping converters must exist");
        fs::write(out.join(format!("{name}_shipping.rs")), rust).unwrap();
    }
}
