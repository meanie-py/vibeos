use crate::{Corpus, Rule};
use regex::Regex;
pub struct RawUartWrites;
const BASE: &str = "Arm64UartKernelBase";
const SANCTIONED: &str = "sdk/lib/rust/ntoskrnl-rust/src/arm64_dbg.rs";
const WINDOW: usize = 80;
#[cfg(test)]
use crate::lexical::strip_noise as blank_noise;
#[cfg(test)]
fn scan(text: &str) -> Vec<(usize, String)> {
    scan_code(&blank_noise(text))
}
fn scan_code(stripped: &str) -> Vec<(usize, String)> {
    let lines: Vec<_> = stripped.lines().collect();
    let bind = Regex::new(r"\blet\s+(?:mut\s+)?(\w+)\s*(?::[^=;]+)?=").unwrap();
    let store = Regex::new(r"(?:^|[;{}])\s*\*(\w+)\s*=|\bwrite_volatile\(\s*(\w+)\s*,").unwrap();
    let direct=Regex::new(r"\bwrite_volatile\(\s*\(?\s*Arm64UartKernelBase\b|\*\s*\(\s*Arm64UartKernelBase\s+as\b[^)]*\)\s*=").unwrap();
    let mut latest: std::collections::HashMap<String, (usize, bool)> = Default::default();
    let mut out = vec![];
    for (i, line) in lines.iter().enumerate() {
        for m in bind.captures_iter(line) {
            if line
                .as_bytes()
                .get(m.get(0).unwrap().end())
                .is_some_and(|b| *b == b'=')
            {
                continue;
            }
            let name = m.get(1).unwrap().as_str();
            let rest = &line[m.get(0).unwrap().end()..];
            let init = format!("{} {}", rest, lines.get(i + 1).copied().unwrap_or(""));
            latest.insert(name.to_owned(), (i, init.contains(BASE)));
        }
        if direct.find_iter(line).any(|m| {
            !line
                .as_bytes()
                .get(m.end().saturating_sub(1))
                .is_some_and(|b| {
                    *b == b'=' && line.as_bytes().get(m.end()).is_some_and(|n| *n == b'=')
                })
        }) {
            out.push((i + 1, BASE.to_owned()));
            continue;
        }
        for m in store.captures_iter(line) {
            if m.get(0).unwrap().as_str().ends_with('=')
                && line
                    .as_bytes()
                    .get(m.get(0).unwrap().end())
                    .is_some_and(|b| *b == b'=')
            {
                continue;
            }
            let name = m.get(1).or_else(|| m.get(2)).unwrap().as_str();
            if let Some(&(at, true)) = latest.get(name) {
                if i - at <= WINDOW {
                    out.push((i + 1, name.to_owned()));
                }
            }
        }
    }
    out
}
impl Rule for RawUartWrites {
    fn name(&self) -> &'static str {
        "scan_raw_uart_writes"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        let mut out = vec![];
        for f in c.rust_files() {
            if f.path.to_string_lossy() == SANCTIONED || !f.text.contains(BASE) {
                continue;
            }
            for (n, name) in scan_code(f.rust_noise()) {
                out.push(format!("{}:{}: RAW_UART_STORE through `{}` — a bare store to the PL011 data register drops bytes once the FIFO is full; use Arm64UartPutChar / crate::arm64_dbg::uart_putc",f.path.display(),n,name));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hits(src: &str) -> Vec<usize> {
        scan(src).into_iter().map(|x| x.0).collect()
    }
    #[test]
    fn deref_store_through_bound_pointer() {
        assert_eq!(hits("unsafe fn t(msg: &[u8]) {\n let u = Arm64UartKernelBase as *mut ULONG;\n for &b in msg {\n *u = b as ULONG;\n }\n}\n"),vec![4]);
    }
    #[test]
    fn volatile_store_through_bound_pointer() {
        assert_eq!(hits("let uart = core::ptr::read_volatile(\n core::ptr::addr_of!(Arm64UartKernelBase)) as *mut u32;\ncore::ptr::write_volatile(uart, c as u32);\n"),vec![3]);
    }
    #[test]
    fn stores_through_base_fire() {
        assert_eq!(
            hits("*(Arm64UartKernelBase as *mut u32) = 0x41;\n"),
            vec![1]
        );
        assert_eq!(
            hits("write_volatile(Arm64UartKernelBase as *mut u32, 0x41);\n"),
            vec![1]
        );
    }
    #[test]
    fn sanctioned_primitive_does_not_fire() {
        assert!(hits("let u = Arm64UartKernelBase as *mut ULONG;\nif !u.is_null() { Arm64UartPutChar(b as u8); }\n").is_empty());
    }
    #[test]
    fn nearest_unrelated_binding_wins() {
        assert!(hits("let u = Arm64UartKernelBase as *mut ULONG;\nArm64UartPutChar(b'x');\nlet u = buffer.as_mut_ptr();\n*u = 7;\n").is_empty());
    }
    #[test]
    fn comments_and_strings_are_skipped() {
        assert!(hits("let u = Arm64UartKernelBase as *mut ULONG;\n// *u = c as ULONG;\nlet s = \"*u = 1;\";\n").is_empty());
        assert!(hits("let u = Arm64UartKernelBase as *mut ULONG;\nlet s = r#\"*u = 1;\"#;\n/* *u = 2; /* nested */ */\n").is_empty());
    }
    #[test]
    fn comparison_is_not_a_store() {
        assert!(
            hits("let u = Arm64UartKernelBase as *mut ULONG;\nif *u == 0 { return; }\n").is_empty()
        );
    }
    #[test]
    fn tokenizer_preserves_lifetimes_and_unicode() {
        let source =
            "let s: &'static str = \"Arm64UartKernelBase *u = 1;\"; // λ Arm64UartKernelBase\n";
        let stripped = blank_noise(source);
        assert!(stripped.contains("&'static str"));
        assert_eq!(stripped.matches('\n').count(), 1);
    }
}
