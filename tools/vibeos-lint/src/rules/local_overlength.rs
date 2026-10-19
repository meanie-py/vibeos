//! A call that hands the address of ONE local of type `T` together with a length of
//! `size_of::<T>() + ...`: the extra bytes land on whatever the compiler placed next.
//!
//! The C shape it comes from is a struct with a trailing buffer (`struct { FILE_FS_VOLUME_
//! INFORMATION Info; WCHAR Name[255]; } v; Nt...(&v, sizeof(v))`), translated as two
//! separate locals. Rust gives no layout guarantee for them, so the callee's tail writes
//! are a stack overrun. kernel32's GetFileInformationByHandle wrote the volume label over
//! its caller's file handle on FAT, and 7-Zip and libarchive failed every archive with
//! "out of memory" (2026-10-18). The fix is one `#[repr(C)]` struct holding both.
use crate::{Corpus, Rule};
use regex::Regex;
use std::sync::OnceLock;

fn skip(path: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)")
            .unwrap()
    })
    .is_match(path)
}

struct Patterns {
    plus: Regex,
    addr: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        // `size_of::<T>()`, optionally cast, then `+`.
        plus: Regex::new(
            r"(?:core::mem::|std::mem::|mem::)?size_of::<\s*((?:[\w:]+::)?\w+)\s*>\(\)\s*(?:as\s+\w+\s*)?\+",
        )
        .unwrap(),
        addr: Regex::new(r"(?:&\s*mut\s+|&\s*|addr_of_mut!\(\s*|addr_of!\(\s*)([A-Za-z_]\w*)\b").unwrap(),
    })
}

fn leaf(t: &str) -> &str {
    t.rsplit("::").next().unwrap()
}

/// The statement around `at`: back to the previous `;`, `{` or `}`, forward to the next `;`.
fn statement(text: &str, at: usize) -> &str {
    let start = text[..at].rfind([';', '{', '}']).map_or(0, |i| i + 1);
    let end = text[at..].find(';').map_or(text.len(), |i| at + i);
    &text[start..end]
}

fn declared_as(text: &str, before: usize, name: &str, ty: &str) -> bool {
    let decl = Regex::new(&format!(
        r"\blet\s+(?:mut\s+)?{}\s*:\s*(?:[\w:]+::)?{}\s*=",
        regex::escape(name),
        regex::escape(ty)
    ))
    .unwrap();
    decl.is_match(&text[..before])
}

pub fn scan_text(text: &str) -> Vec<(usize, String)> {
    let p = patterns();
    let mut out = Vec::new();
    for m in p.plus.captures_iter(text) {
        let whole = m.get(0).unwrap();
        let ty = leaf(m.get(1).unwrap().as_str());
        let stmt = statement(text, whole.start());
        for a in p.addr.captures_iter(stmt) {
            let name = a.get(1).unwrap().as_str();
            if declared_as(text, whole.start(), name, ty) {
                let line = text[..whole.start()].bytes().filter(|b| *b == b'\n').count() + 1;
                out.push((line, format!("&{name}: {ty} passed with size_of::<{ty}>() + ...")));
                break;
            }
        }
    }
    out
}

pub struct LocalOverlength;
impl Rule for LocalOverlength {
    fn name(&self) -> &'static str {
        "scan_local_overlength"
    }
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String> {
        Ok(corpus
            .rust_files()
            .filter(|f| !skip(&f.path.to_string_lossy()))
            .flat_map(|f| {
                scan_text(&f.text)
                    .into_iter()
                    .map(|(n, what)| format!("{}:{n}: local-overlength {what}", f.path.display()))
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_locals_one_length() {
        let bad = "let mut FileFsVolume: FILE_FS_VOLUME_INFORMATION = zeroed();\n\
                   let mut FileFsVolumeName: [WCHAR; 255] = [0; 255];\n\
                   errCode = NtQueryVolumeInformationFile(\n    hFile,\n    &mut IoStatusBlock,\n    \
                   &mut FileFsVolume as *mut _ as PVOID,\n    \
                   (core::mem::size_of::<FILE_FS_VOLUME_INFORMATION>() + core::mem::size_of::<[WCHAR; 255]>()) as ULONG,\n    \
                   FileFsVolumeInformation,\n);";
        assert_eq!(scan_text(bad).len(), 1);
    }
    #[test]
    fn one_struct_or_an_allocation() {
        let good = "let mut FileFsVolume: VolumeInfo = zeroed();\n\
                    errCode = NtQueryVolumeInformationFile(hFile, &mut Io, &mut FileFsVolume as *mut _ as PVOID, \
                    core::mem::size_of::<VolumeInfo>() as ULONG, FileFsVolumeInformation);";
        assert!(scan_text(good).is_empty());
        let alloc = "let p = SHAlloc((size_of::<FMITEM>() + len * size_of::<WCHAR>()) as SIZE_T);";
        assert!(scan_text(alloc).is_empty());
        let buf = "let mut b = [0u8; core::mem::size_of::<BITMAPINFOHEADER>() + core::mem::size_of::<[RGBQUAD; 16]>()];\n\
                   f(&mut b, core::mem::size_of::<BITMAPINFOHEADER>() + 64);";
        assert!(scan_text(buf).is_empty());
    }
}
