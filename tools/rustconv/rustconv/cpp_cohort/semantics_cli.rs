mod cpp_std_semantics;

use cpp_std_semantics::{substr_bytes, stol_bytes, NumericError, StolAbi, SubstrError};

fn unhex(hex: &str) -> Vec<u8> {
    assert!(hex.len() % 2 == 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn hex(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 && args.len() != 6 {
        std::process::exit(2);
    }
    let data = unhex(&args[2]);
    if args[1] == "stol" && args.len() == 6 {
        let base: u32 = args[3].parse().unwrap();
        let abi = StolAbi { long_bits: args[4].parse().unwrap(), binary_prefix: args[5] == "1" };
        match stol_bytes(&data, base, abi) {
            Ok((value, consumed)) => println!("ok:{value}:{consumed}"),
            Err(NumericError::InvalidArgument) => println!("invalid"),
            Err(NumericError::OutOfRange) => println!("range"),
            Err(NumericError::UnsupportedInput) => println!("unsupported"),
        }
    } else if args[1] == "substr" && args.len() == 5 {
        let pos: usize = args[3].parse().unwrap();
        let count = if args[4] == "npos" { None } else { Some(args[4].parse().unwrap()) };
        match substr_bytes(&data, pos, count) {
            Ok(value) => println!("ok:{}", hex(&value)),
            Err(SubstrError::OutOfRange) => println!("range"),
        }
    } else {
        std::process::exit(2);
    }
}
