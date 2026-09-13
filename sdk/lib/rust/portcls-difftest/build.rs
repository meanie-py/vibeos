use quote::ToTokens;
use std::{env, fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let source = root.join("drivers/wdm/audio/backpln/portcls/resource.cpp");
    let base = root.join("drivers/wdm/audio/backpln/portcls/private.hpp");
    let private = fs::read_to_string(&base).unwrap();
    let start = private.find("template<typename... Interfaces>").unwrap();
    let end = private[start..].find("\n};").unwrap() + start + 3;
    fs::write(out.join("unknown_base.hpp"), &private[start..end]).unwrap();
    let tag = private
        .lines()
        .find(|line| line.starts_with("#define TAG_PORTCLASS "))
        .unwrap();
    fs::write(out.join("source_tag.hpp"), tag).unwrap();
    fs::copy(&source, out.join("reference.cpp")).unwrap();
    for path in [&source, &base] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=fixtures");
    cc::Build::new()
        .compiler("c++")
        .cpp(true)
        .flag("-fno-aggressive-loop-optimizations")
        .opt_level(1)
        .flag("-fno-exceptions")
        .flag("-fms-extensions")
        .flag("-fno-strict-aliasing")
        .file(out.join("reference.cpp"))
        .include("fixtures")
        .include(&out)
        .flag_if_supported("-std=c++17")
        .flag("-fcheck-new")
        .define("PcNewResourceList", "ref_PcNewResourceList")
        .define("PcNewResourceSublist", "ref_PcNewResourceSublist")
        .compile("portcls_original");
    cc::Build::new()
        .compiler("c++")
        .cpp(true)
        .flag("-fno-aggressive-loop-optimizations")
        .opt_level(1)
        .flag("-fno-exceptions")
        .flag("-fms-extensions")
        .flag("-fno-strict-aliasing")
        .files(["fixtures/runtime.cpp", "fixtures/exercise.cpp"])
        .include("fixtures")
        .include(&out)
        .flag_if_supported("-std=c++17")
        .compile("portcls_fixture");
    let src = root.join("sdk/lib/rust/portcls-rust/src");
    for name in [
        "resource",
        "resource_owner",
        "resource_factory",
        "resource_support",
    ] {
        let path = src.join(format!("{name}.rs"));
        println!("cargo:rerun-if-changed={}", path.display());
        let text = fs::read_to_string(path).unwrap();
        let text = text
            .lines()
            .map(|line| {
                line.strip_prefix("//!")
                    .map_or_else(|| line.to_owned(), |doc| format!("//{doc}"))
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(out.join(format!("{name}.rs")), text).unwrap();
    }
    let path = src.join("lib.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    let tree = syn::parse_file(&fs::read_to_string(path).unwrap()).unwrap();
    let macros: String = tree
        .items
        .into_iter()
        .filter_map(|i| match i {
            syn::Item::Macro(m) => Some(m.into_token_stream().to_string()),
            _ => None,
        })
        .collect();
    fs::write(out.join("macros.rs"), macros).unwrap();
    let path = src.join("types_amd64.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    let tree = syn::parse_file(&fs::read_to_string(path).unwrap()).unwrap();
    let names = [
        "IUnknown",
        "IUnknownVtbl",
        "PUNKNOWN",
        "IResourceList",
        "IResourceListVtbl",
        "PRESOURCELIST",
        "RustBridge_CResourceList_Context",
        "RUSTOWNED_TAG_PORTCLASS",
    ];
    let mut types = String::new();
    let mut found = 0;
    for item in tree.items {
        let name = match &item {
            syn::Item::Struct(i) => i.ident.to_string(),
            syn::Item::Type(i) => i.ident.to_string(),
            syn::Item::Const(i) => i.ident.to_string(),
            _ => continue,
        };
        if names.contains(&name.as_str()) {
            types.push_str(&item.into_token_stream().to_string());
            found += 1;
        }
    }
    assert_eq!(found, names.len());
    fs::write(out.join("interfaces.rs"), types).unwrap();
}
