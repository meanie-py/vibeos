use crate::prelude::*;
use core::ffi::c_void;

// (&[0x002e, 0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T "..." */ expanded: wide literal, NUL-terminated (190 u16 units)
#[no_mangle]
pub unsafe extern "C" fn MainUsage() {
    let mut c: INT = 0;
    let description = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        '\t' as u16, 'S' as u16, 'C' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'l' as u16, 'i' as u16, 'n' as u16, 'e' as u16, ' ' as u16, 'p' as u16, 'r' as u16, 'o' as u16, 'g' as u16, 'r' as u16, 'a' as u16, 'm' as u16, ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'u' as u16, 'n' as u16, 'i' as u16, 'c' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'h' as u16, '\n' as u16,
        '\t' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'i' as u16, 't' as u16, 's' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        '\t' as u16, 's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, '[' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ']' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '1' as u16, '>' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '2' as u16, '>' as u16, '.' as u16, '.' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, 'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 'a' as u16, 'l' as u16, ' ' as u16, 'p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 't' as u16, 'e' as u16, 'r' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'h' as u16, 'a' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, ' ' as u16, '"' as u16, '\\' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '"' as u16, '\n' as u16,
        '\t' as u16, 'F' as u16, 'u' as u16, 'r' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'p' as u16, ' ' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'c' as u16, 'a' as u16, 'n' as u16, ' ' as u16, 'b' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'b' as u16, 't' as u16, 'a' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ':' as u16, ' ' as u16, '"' as u16, 's' as u16, 'c' as u16, ' ' as u16, '[' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ']' as u16, '"' as u16, '\n' as u16,
        '\t' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, 's' as u16, ':' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ',' as u16, ' ' as u16, 'o' as u16, 'r' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'e' as u16, 'x' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'x' as u16, 't' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ',' as u16, ' ' as u16, 'o' as u16, 'r' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'p' as u16, 'a' as u16, 'u' as u16, 's' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'P' as u16, 'A' as u16, 'U' as u16, 'S' as u16, 'E' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'g' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'I' as u16, 'N' as u16, 'T' as u16, 'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'G' as u16, 'A' as u16, 'T' as u16, 'E' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'i' as u16, 'n' as u16, 'u' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'C' as u16, 'O' as u16, 'N' as u16, 'T' as u16, 'I' as u16, 'N' as u16, 'U' as u16, 'E' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 's' as u16, 't' as u16, 'o' as u16, 'p' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'S' as u16, 'T' as u16, 'O' as u16, 'P' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 'u' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'p' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 's' as u16, 't' as u16, 'e' as u16, 'n' as u16, 't' as u16, ')' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 't' as u16, 'a' as u16, 'k' as u16, 'e' as u16, 'n' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'u' as u16, 'p' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'q' as u16, 'c' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 'u' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'q' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'q' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 't' as u16, 'a' as u16, 'k' as u16, 'e' as u16, 'n' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'u' as u16, 'p' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'D' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'f' as u16, 'r' as u16, 'o' as u16, 'm' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16, ')' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'c' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'a' as u16, 'd' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'i' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16, ')' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 's' as u16, 'd' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\'' as u16, 's' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16, 'y' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'o' as u16, 'r' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 's' as u16, 'd' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\'' as u16, 's' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16, 'y' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'o' as u16, 'r' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'G' as u16, 'e' as u16, 't' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'G' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'G' as u16, 'e' as u16, 't' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'G' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        '\t' as u16, ' ' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ':' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 'c' as u16, 'i' as u16, 'e' as u16, 's' as u16, '.' as u16, '\n' as u16,
        0,
    ];
    _tprintf(description.as_ptr() as *const wchar_t);

    let prompt = [
        '\n' as u16, 'W' as u16, 'o' as u16, 'u' as u16, 'l' as u16, 'd' as u16, ' ' as u16, 'y' as u16, 'o' as u16, 'u' as u16, ' ' as u16, 'l' as u16, 'i' as u16, 'k' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'e' as u16, ' ' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'p' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'Q' as u16, 'U' as u16, 'E' as u16, 'R' as u16, 'Y' as u16, ' ' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'Q' as u16, 'U' as u16, 'E' as u16, 'R' as u16, 'Y' as u16, 'E' as u16, 'X' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, 's' as u16, '?' as u16, ' ' as u16, '[' as u16, ' ' as u16, 'y' as u16, ' ' as u16, '|' as u16, ' ' as u16, 'n' as u16, ' ' as u16, ']' as u16, ':' as u16, ' ' as u16,
        0,
    ];
    _tprintf(prompt.as_ptr() as *const wchar_t);
    c = _getch() as INT; // _gettch isn't defined in our tchar.h

    let fmt = [ '%' as u16, 'c' as u16, '\n' as u16, 0 ];
    _tprintf(fmt.as_ptr() as *const wchar_t, c);
    if tolower(c as ::core::ffi::c_int) == 'y' as i32
    {
        let options = [
            'Q' as u16, 'U' as u16, 'E' as u16, 'R' as u16, 'Y' as u16, ' ' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'Q' as u16, 'U' as u16, 'E' as u16, 'R' as u16, 'Y' as u16, 'E' as u16, 'X' as u16, ' ' as u16, 'O' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, ' ' as u16, ':' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'I' as u16, 'f' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'w' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ',' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'a' as u16, 't' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 't' as u16, 'u' as u16, 'r' as u16, 'n' as u16, 'e' as u16, 'd' as u16, '.' as u16, ' ' as u16, ' ' as u16, 'F' as u16, 'u' as u16, 'r' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 'd' as u16, 'o' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, ' ' as u16, 'a' as u16, 'p' as u16, 'p' as u16, 'l' as u16, 'y' as u16, ' ' as u16, 'i' as u16, 'n' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'c' as u16, 'a' as u16, 's' as u16, 'e' as u16, '.' as u16, ' ' as u16, ' ' as u16, 'I' as u16, 'f' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'w' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, 'h' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'o' as u16, 'n' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'f' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 'l' as u16, 'i' as u16, 's' as u16, 't' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'b' as u16, 'e' as u16, 'l' as u16, 'o' as u16, 'w' as u16, ',' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'r' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16, '.' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'T' as u16, 'y' as u16, 'p' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ',' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ',' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, ',' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, ' ' as u16, '=' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, '=' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ',' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ',' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, ' ' as u16, '=' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '=' as u16, ' ' as u16, 'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, ' ' as u16, '(' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 'b' as u16, 'y' as u16, 't' as u16, 'e' as u16, 's' as u16, ')' as u16, ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 'f' as u16, 'e' as u16, 'r' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, ' ' as u16, '=' as u16, ' ' as u16, '4' as u16, '0' as u16, '9' as u16, '6' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'r' as u16, 'i' as u16, '=' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 's' as u16, 'u' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'x' as u16, ' ' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'b' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'a' as u16, 't' as u16, ' ' as u16, 'w' as u16, 'h' as u16, 'i' as u16, 'c' as u16, 'h' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'b' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, ' ' as u16, '=' as u16, ' ' as u16, '0' as u16, ')' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, ' ' as u16, ' ' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, '\n' as u16,
            ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, ' ' as u16, '=' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, 's' as u16, ')' as u16, '\n' as u16,
            'S' as u16, 'Y' as u16, 'N' as u16, 'T' as u16, 'A' as u16, 'X' as u16, ' ' as u16, 'E' as u16, 'X' as u16, 'A' as u16, 'M' as u16, 'P' as u16, 'L' as u16, 'E' as u16, 'S' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, '&' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'm' as u16, 'e' as u16, 's' as u16, 's' as u16, 'e' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 's' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'm' as u16, 'e' as u16, 's' as u16, 's' as u16, 'e' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'e' as u16, 'x' as u16, ' ' as u16, 'm' as u16, 'e' as u16, 's' as u16, 's' as u16, 'e' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 's' as u16, ' ' as u16, 'e' as u16, 'x' as u16, 't' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'm' as u16, 'e' as u16, 's' as u16, 's' as u16, 'e' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'o' as u16, 'n' as u16, 'l' as u16, 'y' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'o' as u16, 'n' as u16, 'l' as u16, 'y' as u16, ' ' as u16, 'W' as u16, 'i' as u16, 'n' as u16, '3' as u16, '2' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, '=' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, '&' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '=' as u16, ' ' as u16, '5' as u16, '0' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'h' as u16, ' ' as u16, 'a' as u16, ' ' as u16, '5' as u16, '0' as u16, ' ' as u16, 'b' as u16, 'y' as u16, 't' as u16, 'e' as u16, ' ' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 'f' as u16, 'e' as u16, 'r' as u16, '.' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'r' as u16, 'i' as u16, '=' as u16, ' ' as u16, '1' as u16, '4' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'h' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 's' as u16, 'u' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'x' as u16, ' ' as u16, '=' as u16, ' ' as u16, '1' as u16, '4' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'e' as u16, 'x' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, ' ' as u16, '"' as u16, '"' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '\n' as u16,
            's' as u16, 'c' as u16, ' ' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, ' ' as u16, 'N' as u16, 'D' as u16, 'I' as u16, 'S' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'l' as u16, 'l' as u16, ' ' as u16, 'N' as u16, 'D' as u16, 'I' as u16, 'S' as u16, ' ' as u16, 'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 's' as u16, '\n' as u16,
            0,
        ];
        _tprintf(options.as_ptr() as *const wchar_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn StartUsage() {
    let usage: [u16; 114] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'r' as u16, 'u' as u16, 'n' as u16, 'n' as u16, 'i' as u16, 'n' as u16, 'g' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16,
        ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16,
        ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
        ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16,
        ' ' as u16, '<' as u16, 'a' as u16, 'r' as u16, 'g' as u16, '1' as u16, '>' as u16,
        ' ' as u16, '<' as u16, 'a' as u16, 'r' as u16, 'g' as u16, '2' as u16, '>' as u16,
        ' ' as u16, '.' as u16, '.' as u16, '.' as u16, '\n' as u16,
        0,
    ];
    _tprintf(usage.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn PauseUsage() {
    let fmt = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        'P' as u16, 'A' as u16, 'U' as u16, 'S' as u16, 'E' as u16, ' ' as u16,
        'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16,
        't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'p' as u16, 'a' as u16, 'u' as u16, 's' as u16, 'e' as u16, ' ' as u16,
        '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, '\n' as u16,
        0,
    ];
    _tprintf(fmt.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn InterrogateUsage() {
    let DescriptionText = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'n' as u16,
        ' ' as u16, 'I' as u16, 'N' as u16, 'T' as u16, 'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16,
        'G' as u16, 'A' as u16, 'T' as u16, 'E' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16,
        't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16,
        'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16,
        'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16,
        'r' as u16, 'r' as u16, 'o' as u16, 'g' as u16, 'a' as u16, 't' as u16, 'e' as u16, ' ' as u16,
        '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
        ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, '\n' as u16,
        0,
    ];
    _tprintf(DescriptionText.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn StopUsage() {
    let usage_text: [u16; _] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        'S' as u16, 'T' as u16, 'O' as u16, 'P' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16,
        't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16,
        'u' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16,
        'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 's' as u16, 't' as u16, 'o' as u16, 'p' as u16,
        ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, '\n' as u16,
        0,
    ];
    _tprintf(usage_text.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn ContinueUsage() {
    let s = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16,
        'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        'C' as u16, 'O' as u16, 'N' as u16, 'T' as u16, 'I' as u16, 'N' as u16, 'U' as u16, 'E' as u16,
        ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        ' ' as u16, 'r' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'e' as u16, 's' as u16, 't' as u16,
        ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16,
        'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16,
        'i' as u16, 'n' as u16, 'u' as u16, 'e' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16,
        'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16,
        'm' as u16, 'e' as u16, ']' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(s.as_ptr() as *const u16);
}
#[no_mangle]
pub unsafe extern "C" fn DeleteUsage() {
    let Text = [
        // "DESCRIPTION:\n"
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        // "        Deletes a service entry from the registry.\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'D' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'y' as u16, ' ' as u16,
        'f' as u16, 'r' as u16, 'o' as u16, 'm' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16,
        ' ' as u16, 'r' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16,
        'y' as u16, '.' as u16, '\n' as u16,
        // "        If the service is running, or another process has an\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'I' as u16, 'f' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 's' as u16,
        'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'i' as u16,
        's' as u16, ' ' as u16, 'r' as u16, 'u' as u16, 'n' as u16, 'n' as u16, 'i' as u16, 'n' as u16,
        'g' as u16, ',' as u16, ' ' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'a' as u16, 'n' as u16,
        'o' as u16, 't' as u16, 'h' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'p' as u16, 'r' as u16,
        'o' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, ' ' as u16, 'h' as u16, 'a' as u16,
        's' as u16, ' ' as u16, 'a' as u16, 'n' as u16, '\n' as u16,
        // "        open handle to the service, the service is simply marked\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, ' ' as u16, 'h' as u16, 'a' as u16, 'n' as u16,
        'd' as u16, 'l' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 't' as u16,
        'h' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16,
        'c' as u16, 'e' as u16, ',' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'i' as u16, 's' as u16, ' ' as u16, 's' as u16, 'i' as u16, 'm' as u16, 'p' as u16, 'l' as u16,
        'y' as u16, ' ' as u16, 'm' as u16, 'a' as u16, 'r' as u16, 'k' as u16, 'e' as u16, 'd' as u16,
        '\n' as u16,
        // "        for deletion.\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'l' as u16, 'e' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '.' as u16, '\n' as u16,
        // "USAGE:\n"
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        // "        sc <server> delete [service name]\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'l' as u16, 'e' as u16,
        't' as u16, 'e' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16,
        ']' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(Text.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn CreateUsage() {
    let Usage = [
        'C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'y' as u16, ' ' as u16,
        'i' as u16, 'n' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16,
        'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16, ' ' as u16,
        'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'D' as u16, 'a' as u16, 't' as u16, 'a' as u16,
        'b' as u16, 'a' as u16, 's' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'S' as u16, 'Y' as u16, 'N' as u16, 'T' as u16, 'A' as u16, 'X' as u16, ':' as u16, '\n' as u16,
        's' as u16, 'c' as u16, ' ' as u16, 'c' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16,
        'e' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16,
        'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16,
        ' ' as u16, '[' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'P' as u16, 'a' as u16, 't' as u16,
        'h' as u16, '=' as u16, ' ' as u16, ']' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '1' as u16, '>' as u16, ' ' as u16, '<' as u16,
        'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '2' as u16, '>' as u16,
        '.' as u16, '.' as u16, '.' as u16, '\n' as u16,
        'C' as u16, 'R' as u16, 'E' as u16, 'A' as u16, 'T' as u16, 'E' as u16, ' ' as u16, 'O' as u16,
        'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, ':' as u16, '\n' as u16,
        'N' as u16, 'O' as u16, 'T' as u16, 'E' as u16, ':' as u16, ' ' as u16, 'T' as u16, 'h' as u16,
        'e' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'i' as u16, 'n' as u16,
        'c' as u16, 'l' as u16, 'u' as u16, 'd' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16,
        'h' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'a' as u16, 'l' as u16,
        ' ' as u16, 's' as u16, 'i' as u16, 'g' as u16, 'n' as u16, '.' as u16, '\n' as u16,
        ' ' as u16, 't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, '<' as u16,
        'o' as u16, 'w' as u16, 'n' as u16, '|' as u16, 's' as u16, 'h' as u16, 'a' as u16, 'r' as u16,
        'e' as u16, '|' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16,
        'c' as u16, 't' as u16, '|' as u16, 'k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16,
        'l' as u16, '|' as u16, 'f' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 's' as u16, 'y' as u16,
        's' as u16, '|' as u16, 'r' as u16, 'e' as u16, 'c' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16,
        ' ' as u16, '=' as u16, ' ' as u16, 'o' as u16, 'w' as u16, 'n' as u16, ')' as u16, '\n' as u16,
        ' ' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, '=' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '|' as u16, 's' as u16, 'y' as u16,
        's' as u16, 't' as u16, 'e' as u16, 'm' as u16, '|' as u16, 'a' as u16, 'u' as u16, 't' as u16,
        'o' as u16, '|' as u16, 'd' as u16, 'e' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16,
        '|' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16,
        'd' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16,
        ' ' as u16, '=' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'm' as u16, 'a' as u16, 'n' as u16,
        'd' as u16, ')' as u16, '\n' as u16,
        ' ' as u16, 'e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '=' as u16, ' ' as u16,
        '<' as u16, 'n' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 'l' as u16, '|' as u16,
        's' as u16, 'e' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 'e' as u16, '|' as u16, 'c' as u16,
        'r' as u16, 'i' as u16, 't' as u16, 'i' as u16, 'c' as u16, 'a' as u16, 'l' as u16, '|' as u16,
        'i' as u16, 'g' as u16, 'n' as u16, 'o' as u16, 'r' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16,
        ' ' as u16, '=' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16,
        'l' as u16, ')' as u16, '\n' as u16,
        ' ' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'P' as u16, 'a' as u16, 't' as u16, 'h' as u16,
        '=' as u16, ' ' as u16, '<' as u16, 'B' as u16, 'i' as u16, 'n' as u16, 'a' as u16, 'r' as u16,
        'y' as u16, 'P' as u16, 'a' as u16, 't' as u16, 'h' as u16, 'N' as u16, 'a' as u16, 'm' as u16,
        'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, 'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, ' ' as u16,
        '<' as u16, 'L' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'O' as u16, 'r' as u16, 'd' as u16,
        'e' as u16, 'r' as u16, 'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '>' as u16,
        '\n' as u16,
        ' ' as u16, 't' as u16, 'a' as u16, 'g' as u16, '=' as u16, ' ' as u16, '<' as u16, 'y' as u16,
        'e' as u16, 's' as u16, '|' as u16, 'n' as u16, 'o' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, 'd' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, '=' as u16,
        ' ' as u16, '<' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16,
        'e' as u16, 'n' as u16, 'c' as u16, 'i' as u16, 'e' as u16, 's' as u16, '(' as u16, 's' as u16,
        'e' as u16, 'p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16,
        ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, '/' as u16, ' ' as u16, '(' as u16, 'f' as u16,
        'o' as u16, 'r' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 's' as u16,
        'l' as u16, 'a' as u16, 's' as u16, 'h' as u16, ')' as u16, ')' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, 'o' as u16, 'b' as u16, 'j' as u16, '=' as u16, ' ' as u16, '<' as u16, 'A' as u16,
        'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 'N' as u16, 'a' as u16,
        'm' as u16, 'e' as u16, '|' as u16, 'O' as u16, 'b' as u16, 'j' as u16, 'e' as u16, 'c' as u16,
        't' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '(' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16,
        ' ' as u16, '=' as u16, ' ' as u16, 'L' as u16, 'o' as u16, 'c' as u16, 'a' as u16, 'l' as u16,
        'S' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16, ')' as u16, '\n' as u16,
        ' ' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16,
        'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '=' as u16, ' ' as u16, '<' as u16, 'd' as u16,
        'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16, 'n' as u16,
        'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, 'p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16,
        'd' as u16, '=' as u16, ' ' as u16, '<' as u16, 'p' as u16, 'a' as u16, 's' as u16, 's' as u16,
        'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, '>' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(Usage.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn ControlUsage() {
    let buf = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 'C' as u16, 'O' as u16, 'N' as u16, 'T' as u16, 'R' as u16, 'O' as u16, 'L' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'd' as u16, 'e' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16, '<' as u16, 'v' as u16, 'a' as u16, 'l' as u16, 'u' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '<' as u16, 'v' as u16, 'a' as u16, 'l' as u16, 'u' as u16, 'e' as u16, '>' as u16, ' ' as u16, '=' as u16, ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'r' as u16, '-' as u16, 'd' as u16, 'e' as u16, 'f' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'd' as u16, 'e' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '<' as u16, 'v' as u16, 'a' as u16, 'l' as u16, 'u' as u16, 'e' as u16, '>' as u16, ' ' as u16, '=' as u16, ' ' as u16, '<' as u16, 'p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'm' as u16, 'c' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, '|' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'a' as u16, 'd' as u16, 'd' as u16, '|' as u16, 'n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'r' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 'v' as u16, 'e' as u16, '|' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, '|' as u16, 'n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(buf.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn SdShowUsage() {
    let s: [u16; 128] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\'' as u16, 's' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16, 'y' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 'S' as u16, 'D' as u16, 'D' as u16, 'L' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 't' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 's' as u16, 'd' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        0,
    ];
    _tprintf(s.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn SdSetUsage() {
    let usage: [u16; 128] = [
        // "DESCRIPTION:\n"
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16,
        'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        // "        Sets a service's security descriptor.\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
        '\'' as u16, 's' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'i' as u16, 't' as u16, 'y' as u16,
        ' ' as u16,
        'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16,
        'o' as u16, 'r' as u16, '.' as u16, '\n' as u16,
        // "USAGE:\n"
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        // "        sc <server> sdset <service name> <SD in SDDL format>\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16,
        'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        's' as u16, 'd' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16,
        '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16,
        'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, ' ' as u16,
        '<' as u16, 'S' as u16, 'D' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16,
        'S' as u16, 'D' as u16, 'D' as u16, 'L' as u16, ' ' as u16,
        'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 't' as u16, '>' as u16,
        '\n' as u16,
        // NUL terminator
        0,
    ];
    _tprintf(usage.as_ptr() as *const u16);
}
#[no_mangle]
pub unsafe extern "C" fn QueryConfigUsage() {
    let Text: [u16; 133] = [
        // "DESCRIPTION:\n"
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16,
        'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        // "        Queries the configuration information for a service.\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'Q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 'u' as u16,
        'r' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16,
        'i' as u16, 'n' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16,
        'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16,
        'a' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16,
        '\n' as u16,
        // "USAGE:\n"
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        // "        sc <server> qc [service name] <bufferSize>\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16,
        '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'q' as u16, 'c' as u16, ' ' as u16,
        '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 'f' as u16, 'e' as u16, 'r' as u16,
        'S' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16,
        '\n' as u16,
        // terminator
        0,
    ];
    _tprintf(core::ptr::addr_of!(Text) as *const u16);
}
#[no_mangle]
pub unsafe extern "C" fn QueryDescriptionUsage() {
    static MSG: [u16; 137] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'R' as u16, 'e' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 'v' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16,
        's' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16,
        'o' as u16, 'f' as u16, ' ' as u16,
        'a' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'q' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16,
        '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 'f' as u16, 'e' as u16, 'r' as u16, 'S' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16,
        '\n' as u16,
        0,
    ];
    _tprintf(MSG.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn QueryFailureUsage() {
    let Msg: [u16; 138] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'R' as u16, 'e' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'e' as u16, 'v' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16,
        'p' as u16, 'e' as u16, 'r' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
        'o' as u16, 'n' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16,
        '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'q' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, ' ' as u16,
        '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 'f' as u16, 'e' as u16, 'r' as u16,
        'S' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        0,
    ];
    _tprintf(Msg.as_ptr() as *const wchar_t);
}
#[no_mangle]
pub unsafe extern "C" fn SetDescriptionUsage() {
    let fmt = [
        b'D' as u16, b'E' as u16, b'S' as u16, b'C' as u16, b'R' as u16, b'I' as u16,
        b'P' as u16, b'T' as u16, b'I' as u16, b'O' as u16, b'N' as u16, b':' as u16,
        b'\n' as u16,
        b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16,
        b' ' as u16, b' ' as u16,
        b'S' as u16, b'e' as u16, b't' as u16, b's' as u16, b' ' as u16, b't' as u16,
        b'h' as u16, b'e' as u16, b' ' as u16, b'd' as u16, b'e' as u16, b's' as u16,
        b'c' as u16, b'r' as u16, b'i' as u16, b'p' as u16, b't' as u16, b'i' as u16,
        b'o' as u16, b'n' as u16, b' ' as u16, b's' as u16, b't' as u16, b'r' as u16,
        b'i' as u16, b'n' as u16, b'g' as u16, b' ' as u16, b'o' as u16, b'f' as u16,
        b' ' as u16, b'a' as u16, b' ' as u16, b's' as u16, b'e' as u16, b'r' as u16,
        b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'.' as u16, b'\n' as u16,
        b'U' as u16, b'S' as u16, b'A' as u16, b'G' as u16, b'E' as u16, b':' as u16,
        b'\n' as u16,
        b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16, b' ' as u16,
        b' ' as u16, b' ' as u16,
        b's' as u16, b'c' as u16, b' ' as u16, b'<' as u16, b's' as u16, b'e' as u16,
        b'r' as u16, b'v' as u16, b'e' as u16, b'r' as u16, b'>' as u16, b' ' as u16,
        b'd' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16, b'i' as u16,
        b'p' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b' ' as u16,
        b'[' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16,
        b'c' as u16, b'e' as u16, b' ' as u16, b'n' as u16, b'a' as u16, b'm' as u16,
        b'e' as u16, b']' as u16, b' ' as u16,
        b'[' as u16, b'd' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16,
        b'i' as u16, b'p' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16,
        b']' as u16, b'\n' as u16,
        0u16,
    ];
    _tprintf(fmt.as_ptr() as *const u16);
}
#[no_mangle]
pub unsafe extern "C" fn SetConfigUsage() {
    let Usage = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'M' as u16, 'o' as u16, 'd' as u16, 'i' as u16, 'f' as u16, 'i' as u16, 'e' as u16, 's' as u16, ' ' as u16, 'a' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16, ' ' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'D' as u16, 'a' as u16, 't' as u16, 'a' as u16, 'b' as u16, 'a' as u16, 's' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ']' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '1' as u16, '>' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '2' as u16, '>' as u16, '.' as u16, '.' as u16, '.' as u16, '\n' as u16,
        'C' as u16, 'O' as u16, 'N' as u16, 'F' as u16, 'I' as u16, 'G' as u16, ' ' as u16, 'O' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'N' as u16, 'O' as u16, 'T' as u16, 'E' as u16, ':' as u16, ' ' as u16, 'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'c' as u16, 'l' as u16, 'u' as u16, 'd' as u16, 'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'e' as u16, 'q' as u16, 'u' as u16, 'a' as u16, 'l' as u16, ' ' as u16, 's' as u16, 'i' as u16, 'g' as u16, 'n' as u16, '.' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'w' as u16, 'n' as u16, '|' as u16, 's' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '|' as u16, 'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, '|' as u16, 'k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '|' as u16, 'f' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 's' as u16, 'y' as u16, 's' as u16, '|' as u16, 'r' as u16, 'e' as u16, 'c' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, '=' as u16, ' ' as u16, '<' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '|' as u16, 's' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16, '|' as u16, 'a' as u16, 'u' as u16, 't' as u16, 'o' as u16, '|' as u16, 'd' as u16, 'e' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, '|' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, 'd' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '=' as u16, ' ' as u16, '<' as u16, 'n' as u16, 'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 'l' as u16, '|' as u16, 's' as u16, 'e' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 'e' as u16, '|' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 't' as u16, 'i' as u16, 'c' as u16, 'a' as u16, 'l' as u16, '|' as u16, 'i' as u16, 'g' as u16, 'n' as u16, 'o' as u16, 'r' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'b' as u16, 'i' as u16, 'n' as u16, 'P' as u16, 'a' as u16, 't' as u16, 'h' as u16, '=' as u16, ' ' as u16, '<' as u16, 'B' as u16, 'i' as u16, 'n' as u16, 'a' as u16, 'r' as u16, 'y' as u16, 'P' as u16, 'a' as u16, 't' as u16, 'h' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, ' ' as u16, '<' as u16, 'L' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'O' as u16, 'r' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 'G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        't' as u16, 'a' as u16, 'g' as u16, '=' as u16, ' ' as u16, '<' as u16, 'y' as u16, 'e' as u16, 's' as u16, '|' as u16, 'n' as u16, 'o' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'd' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, '=' as u16, ' ' as u16, '<' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 'c' as u16, 'i' as u16, 'e' as u16, 's' as u16, '(' as u16, 's' as u16, 'e' as u16, 'p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, '/' as u16, ' ' as u16, '(' as u16, 'f' as u16, 'o' as u16, 'r' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 's' as u16, 'l' as u16, 'a' as u16, 's' as u16, 'h' as u16, ')' as u16, ')' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'o' as u16, 'b' as u16, 'j' as u16, '=' as u16, ' ' as u16, '<' as u16, 'A' as u16, 'c' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '|' as u16, 'O' as u16, 'b' as u16, 'j' as u16, 'e' as u16, 'c' as u16, 't' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '=' as u16, ' ' as u16, '<' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, '=' as u16, ' ' as u16, '<' as u16, 'p' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, '>' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(Usage.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn SetFailureUsage() {
    // "DESCRIPTION:\n"
    let Buf = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        // "        Changes the actions upon failure.\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16,
        'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 'u' as u16, 'p' as u16, 'o' as u16, 'n' as u16,
        ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16,
        '.' as u16, '\n' as u16,
        // "USAGE:\n"
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        // "        sc <server> failure [service name] <option1> <option2>...\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16,
        'u' as u16, 'r' as u16, 'e' as u16, ' ' as u16, '[' as u16, 's' as u16, 'e' as u16, 'r' as u16,
        'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16,
        'e' as u16, ']' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16, 't' as u16, 'i' as u16,
        'o' as u16, 'n' as u16, '1' as u16, '>' as u16, ' ' as u16, '<' as u16, 'o' as u16, 'p' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '2' as u16, '>' as u16, '.' as u16, '.' as u16,
        '.' as u16, '\n' as u16,
        // "\n"
        '\n' as u16,
        // "OPTIONS:\n"
        'O' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, ':' as u16,
        '\n' as u16,
        // "        reset= <Length of period of no failures (in seconds)\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 's' as u16, 'e' as u16, 't' as u16, '=' as u16, ' ' as u16, '<' as u16,
        'L' as u16, 'e' as u16, 'n' as u16, 'g' as u16, 't' as u16, 'h' as u16, ' ' as u16, 'o' as u16,
        'f' as u16, ' ' as u16, 'p' as u16, 'e' as u16, 'r' as u16, 'i' as u16, 'o' as u16, 'd' as u16,
        ' ' as u16, 'o' as u16, 'f' as u16, ' ' as u16, 'n' as u16, 'o' as u16, ' ' as u16, 'f' as u16,
        'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        '(' as u16, 'i' as u16, 'n' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'c' as u16, 'o' as u16,
        'n' as u16, 'd' as u16, 's' as u16, ')' as u16, '\n' as u16,
        // "                after which to reset the failure count to 0 (may be INFINITE)>\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'a' as u16, 'f' as u16, 't' as u16, 'e' as u16, 'r' as u16, ' ' as u16, 'w' as u16, 'h' as u16,
        'i' as u16, 'c' as u16, 'h' as u16, ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16, 'r' as u16,
        'e' as u16, 's' as u16, 'e' as u16, 't' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16,
        ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16,
        ' ' as u16, 'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, ' ' as u16, 't' as u16,
        'o' as u16, ' ' as u16, '0' as u16, ' ' as u16, '(' as u16, 'm' as u16, 'a' as u16, 'y' as u16,
        ' ' as u16, 'b' as u16, 'e' as u16, ' ' as u16, 'I' as u16, 'N' as u16, 'F' as u16, 'I' as u16,
        'N' as u16, 'I' as u16, 'T' as u16, 'E' as u16, ')' as u16, '>' as u16, '\n' as u16,
        // "               (Must be used in conjunction with actions= )\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        '(' as u16, 'M' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16, 'b' as u16, 'e' as u16,
        ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'i' as u16, 'n' as u16,
        ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'j' as u16, 'u' as u16, 'n' as u16, 'c' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16,
        'h' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        's' as u16, '=' as u16, ' ' as u16, ')' as u16, '\n' as u16,
        // "        reboot= <Message broadcast before rebooting on failure>\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '=' as u16, ' ' as u16,
        '<' as u16, 'M' as u16, 'e' as u16, 's' as u16, 's' as u16, 'a' as u16, 'g' as u16, 'e' as u16,
        ' ' as u16, 'b' as u16, 'r' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'c' as u16, 'a' as u16,
        's' as u16, 't' as u16, ' ' as u16, 'b' as u16, 'e' as u16, 'f' as u16, 'o' as u16, 'r' as u16,
        'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16,
        'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16,
        'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        // "        command= <Command line to be run on failure>\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'c' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, '=' as u16,
        ' ' as u16, '<' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16,
        'd' as u16, ' ' as u16, 'l' as u16, 'i' as u16, 'n' as u16, 'e' as u16, ' ' as u16, 't' as u16,
        'o' as u16, ' ' as u16, 'b' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'u' as u16, 'n' as u16,
        ' ' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16,
        'u' as u16, 'r' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        // "        actions= <Failure actions and their delay time (in milliseconds),\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, '=' as u16,
        ' ' as u16, '<' as u16, 'F' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16,
        'e' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        's' as u16, ' ' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 't' as u16, 'h' as u16,
        'e' as u16, 'i' as u16, 'r' as u16, ' ' as u16, 'd' as u16, 'e' as u16, 'l' as u16, 'a' as u16,
        'y' as u16, ' ' as u16, 't' as u16, 'i' as u16, 'm' as u16, 'e' as u16, ' ' as u16, '(' as u16,
        'i' as u16, 'n' as u16, ' ' as u16, 'm' as u16, 'i' as u16, 'l' as u16, 'l' as u16, 'i' as u16,
        's' as u16, 'e' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'd' as u16, 's' as u16, ')' as u16,
        ',' as u16, '\n' as u16,
        // "                  separated by / (forward slash) -- e.g., run/5000/reboot/800\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 't' as u16, 'e' as u16,
        'd' as u16, ' ' as u16, 'b' as u16, 'y' as u16, ' ' as u16, '/' as u16, ' ' as u16, '(' as u16,
        'f' as u16, 'o' as u16, 'r' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16,
        's' as u16, 'l' as u16, 'a' as u16, 's' as u16, 'h' as u16, ')' as u16, ' ' as u16, '-' as u16,
        '-' as u16, ' ' as u16, 'e' as u16, '.' as u16, 'g' as u16, '.' as u16, ',' as u16, ' ' as u16,
        'r' as u16, 'u' as u16, 'n' as u16, '/' as u16, '5' as u16, '0' as u16, '0' as u16, '0' as u16,
        '/' as u16, 'r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '/' as u16,
        '8' as u16, '0' as u16, '0' as u16, '\n' as u16,
        // "                  Valid actions are <run|restart|reboot> >\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16,
        'V' as u16, 'a' as u16, 'l' as u16, 'i' as u16, 'd' as u16, ' ' as u16, 'a' as u16, 'c' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16, 'a' as u16, 'r' as u16,
        'e' as u16, ' ' as u16, '<' as u16, 'r' as u16, 'u' as u16, 'n' as u16, '|' as u16, 'r' as u16,
        'e' as u16, 's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, '|' as u16, 'r' as u16,
        'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '>' as u16, ' ' as u16, '>' as u16,
        '\n' as u16,
        // "                 (Must be used in conjunction with the reset= option)\n"
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        ' ' as u16,
        '(' as u16, 'M' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16, 'b' as u16, 'e' as u16,
        ' ' as u16, 'u' as u16, 's' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'i' as u16, 'n' as u16,
        ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'j' as u16, 'u' as u16, 'n' as u16, 'c' as u16,
        't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16,
        'h' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'r' as u16, 'e' as u16,
        's' as u16, 'e' as u16, 't' as u16, '=' as u16, ' ' as u16, 'o' as u16, 'p' as u16, 't' as u16,
        'i' as u16, 'o' as u16, 'n' as u16, ')' as u16, '\n' as u16,
        // terminator
        0,
    ];
    _tprintf(Buf.as_ptr() as *const wchar_t);
}
#[no_mangle]
pub unsafe extern "C" fn GetDisplayNameUsage() {
    let fmt = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'G' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16,
        ' ' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16,
        ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 's' as u16,
        's' as u16, 'o' as u16, 'c' as u16, 'i' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16,
        ' ' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'h' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        'p' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'i' as u16, 'c' as u16, 'u' as u16, 'l' as u16,
        'a' as u16, 'r' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16,
        'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16, 'G' as u16, 'e' as u16, 't' as u16, 'D' as u16,
        'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'a' as u16,
        'm' as u16, 'e' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'k' as u16, 'e' as u16, 'y' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, ' ' as u16, '<' as u16, 'b' as u16,
        'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        0u16,
    ];
    _tprintf(fmt.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn GetKeyNameUsage() {
    let DescriptionUsage: [u16; 190] = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16, 'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'G' as u16, 'e' as u16, 't' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'k' as u16, 'e' as u16, 'y' as u16, ' ' as u16, 'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16,
        'a' as u16, 's' as u16, 's' as u16, 'o' as u16, 'c' as u16, 'i' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
        'w' as u16, 'i' as u16, 't' as u16, 'h' as u16, ' ' as u16, 'a' as u16, ' ' as u16,
        'p' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'i' as u16, 'c' as u16, 'u' as u16, 'l' as u16, 'a' as u16, 'r' as u16, ' ' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ',' as u16, ' ' as u16,
        'u' as u16, 's' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'e' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 's' as u16, ' ' as u16,
        'i' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'G' as u16, 'e' as u16, 't' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16,
        '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16, '\n' as u16,
        0,
    ];
    _tprintf(DescriptionUsage.as_ptr() as *const wchar_t);
}
#[no_mangle]
pub unsafe extern "C" fn EnumDependUsage() {
    let usageText = [
        'D' as u16, 'E' as u16, 'S' as u16, 'C' as u16, 'R' as u16, 'I' as u16, 'P' as u16, 'T' as u16,
        'I' as u16, 'O' as u16, 'N' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 't' as u16,
        'e' as u16, 's' as u16, ' ' as u16, 't' as u16, 'e' as u16, ' ' as u16,
        'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'a' as u16, 't' as u16, ' ' as u16,
        'a' as u16, 'r' as u16, 'e' as u16, ' ' as u16,
        'd' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 't' as u16, ' ' as u16,
        'o' as u16, 'n' as u16, ' ' as u16,
        't' as u16, 'h' as u16, 'i' as u16, 's' as u16, ' ' as u16,
        'o' as u16, 'n' as u16, 'e' as u16, '.' as u16, '\n' as u16,
        'U' as u16, 'S' as u16, 'A' as u16, 'G' as u16, 'E' as u16, ':' as u16, '\n' as u16,
        ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16, ' ' as u16,
        's' as u16, 'c' as u16, ' ' as u16, '<' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16,
        'e' as u16, 'r' as u16, '>' as u16, ' ' as u16,
        'E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16,
        'n' as u16, 'd' as u16, ' ' as u16, '<' as u16,
        's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
        'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, '>' as u16, ' ' as u16,
        '<' as u16, 'b' as u16, 'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '>' as u16,
        '\n' as u16,
        0,
    ];
    _tprintf(usageText.as_ptr() as *const u16);
}
