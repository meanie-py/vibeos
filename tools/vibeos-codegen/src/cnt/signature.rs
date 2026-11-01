//! C signature parsing used by cnt code generation.

pub fn parse_params(sig: &str) -> Result<Vec<(String, String)>, String> {
    let begin = sig
        .find('(')
        .ok_or_else(|| format!("missing '(' in {sig}"))?;
    let end = sig
        .rfind(')')
        .filter(|&end| end > begin)
        .ok_or_else(|| format!("missing ')' in {sig}"))?;
    let inner = sig[begin + 1..end].trim();
    if inner.is_empty() || inner == "void" {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|part| {
            let spaced = part.replace('*', " * ");
            let mut tokens: Vec<_> = spaced.split_whitespace().collect();
            let name = tokens
                .pop()
                .ok_or_else(|| format!("empty parameter in {sig}"))?;
            if tokens.is_empty() {
                return Err(format!("missing type for {name} in {sig}"));
            }
            Ok((tokens.join(" "), name.to_owned()))
        })
        .collect()
}

pub fn rs_ty(ctype: &str) -> Result<&'static str, String> {
    let c: String = ctype.chars().filter(|c| !c.is_whitespace()).collect();
    match c.as_str() {
        "constchar*" | "charconst*" => Ok("*const u8"),
        "wchar_tconst*" | "constwchar_t*" => Ok("*const u16"),
        "wchar_t**" => Ok("*mut *mut u16"),
        "char*" => Ok("*mut u8"),
        "constvoid*" => Ok("*const core::ffi::c_void"),
        "void*" => Ok("*mut core::ffi::c_void"),
        "char**" => Ok("*mut *mut u8"),
        "wchar_t*" => Ok("*mut u16"),
        "wchar_t" | "wint_t" => Ok("u16"),
        "size_t" => Ok("usize"),
        "int" => Ok("i32"),
        "__int64" => Ok("i64"),
        "unsigned__int64" => Ok("u64"),
        "long" => Ok("i32"),
        "unsignedlong" | "unsigned" | "unsignedint" => Ok("u32"),
        "_locale_t" => Ok("*mut core::ffi::c_void"),
        _ => Err(format!("unsupported parameter type {ctype}")),
    }
}

pub fn ret_ty(sig: &str) -> Result<&'static str, String> {
    let head = sig
        .split_once('(')
        .ok_or_else(|| format!("missing '(' in {sig}"))?
        .0;
    let spaced = head.replace('*', " * ");
    let mut tokens: Vec<_> = spaced
        .split_whitespace()
        .filter(|t| !matches!(*t, "__cdecl" | "CDECL" | "__stdcall" | "WINAPI" | "NTAPI"))
        .collect();
    tokens
        .pop()
        .ok_or_else(|| format!("missing function name in {sig}"))?;
    match tokens.join("").as_str() {
        "size_t" => Ok("usize"),
        "int" | "long" => Ok("i32"),
        "unsignedlong" => Ok("u32"),
        "unsignedlonglong" | "unsigned__int64" => Ok("u64"),
        "wint_t" => Ok("u16"),
        "__int64" => Ok("i64"),
        "char*" => Ok("*mut u8"),
        "wchar_t*" => Ok("*mut u16"),
        "void*" => Ok("*mut core::ffi::c_void"),
        "void" => Ok("()"),
        _ => Err(format!("unsupported return type in {sig}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_pointer_and_calling_convention() {
        let sig = "unsigned long __cdecl example(const char *input, char **end, int radix)";
        let p = parse_params(sig).unwrap();
        assert_eq!(
            p.iter()
                .map(|(t, n)| (rs_ty(t).unwrap(), n.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("*const u8", "input"),
                ("*mut *mut u8", "end"),
                ("i32", "radix")
            ]
        );
        assert_eq!(ret_ty(sig).unwrap(), "u32");
    }
    #[test]
    fn rejects_unknown_types() {
        assert!(rs_ty("float").is_err());
        assert!(ret_ty("float f(void)").is_err());
    }
}
