//! musl-math vs the musl C, bit for bit, plus the error classification the CRTs rely on.
use musl_math::MathErr;
// The musl C reference (build.rs) rides on this crate's lib; name it so it links.
use musl_math_difftest as _;

extern "C" {
    fn musl_exp(x: f64) -> f64;
    fn musl_exp2(x: f64) -> f64;
    fn musl_log(x: f64) -> f64;
    fn musl_log2(x: f64) -> f64;
    fn musl_pow(x: f64, y: f64) -> f64;
    fn musl_expf(x: f32) -> f32;
    fn musl_exp2f(x: f32) -> f32;
    fn musl_logf(x: f32) -> f32;
    fn musl_log2f(x: f32) -> f32;
    fn musl_powf(x: f32, y: f32) -> f32;
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}
fn samef(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

fn inputs() -> Vec<f64> {
    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    let mut xs = vec![
        0.0, -0.0, 1.0, -1.0, 0.5, 2.0, 10.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN,
        f64::MIN_POSITIVE, 5e-324, f64::MAX, 709.78, 709.79, -745.2, -745.1, 1024.0, -1075.0,
        -1074.5, 1e-300, 3.0, -3.0, 0.1, 88.7, 88.8, -103.9, -104.0, 128.0, -150.0, -149.5,
    ];
    for _ in 0..500_000 {
        xs.push(f64::from_bits(r.next())); // every binade, sign, NaN payload
    }
    for _ in 0..500_000 {
        xs.push((r.unit() - 0.5) * 1600.0); // the over/underflow edges of exp
    }
    xs
}

fn diff_unary() -> usize {
    let mut bad = 0;
    for x in inputs() {
        let f = x as f32;
        let checks: [(&str, bool); 8] = unsafe {
            [
                ("exp", same(musl_math::exp(x).0, musl_exp(x))),
                ("exp2", same(musl_math::exp2(x).0, musl_exp2(x))),
                ("log", same(musl_math::log(x).0, musl_log(x))),
                ("log2", same(musl_math::log2(x).0, musl_log2(x))),
                ("expf", samef(musl_math::expf(f).0, musl_expf(f))),
                ("exp2f", samef(musl_math::exp2f(f).0, musl_exp2f(f))),
                ("logf", samef(musl_math::logf(f).0, musl_logf(f))),
                ("log2f", samef(musl_math::log2f(f).0, musl_log2f(f))),
            ]
        };
        for (name, ok) in checks {
            if !ok {
                bad += 1;
                if bad < 10 {
                    eprintln!("MISMATCH {name}({x:e})");
                }
            }
        }
    }
    bad
}

fn diff_pow() -> usize {
    let xs = inputs();
    let n = xs.len();
    let mut r = Rng(0x5eed_0000_0000_0001);
    let mut bad = 0;
    for i in 0..2_000_000usize {
        let (x, y) = match i % 4 {
            0 => (xs[(i * 13) % n], xs[(i * 7 + 1) % n]),
            1 => (r.unit() * 200.0 - 100.0, (r.unit() * 60.0 - 30.0).round()),
            2 => (r.unit() * 4.0, r.unit() * 2000.0 - 1000.0),
            _ => (r.unit() * 200.0 - 100.0, r.unit() * 60.0 - 30.0),
        };
        let (fx, fy) = (x as f32, y as f32);
        unsafe {
            if !same(musl_math::pow(x, y).0, musl_pow(x, y)) {
                bad += 1;
                if bad < 10 {
                    eprintln!("MISMATCH pow({x:e}, {y:e})");
                }
            }
            if !samef(musl_math::powf(fx, fy).0, musl_powf(fx, fy)) {
                bad += 1;
                if bad < 10 {
                    eprintln!("MISMATCH powf({fx:e}, {fy:e})");
                }
            }
        }
    }
    bad
}

/// The reason the port exists: pow(10, n) is the nearest double nearly always. The libm
/// crate's FreeBSD pow misses 65 of these 601; musl's misses 3, glibc's 2.
fn pow10_rounding() -> usize {
    (-300i32..=300)
        .filter(|k| musl_math::pow(10.0, *k as f64).0 != format!("1e{k}").parse::<f64>().unwrap())
        .count()
}

/// The classification the CRT layer turns into errno/_matherr (Wine's math_error points).
fn classification() -> usize {
    use MathErr::*;
    let cases: [(&str, (f64, MathErr), MathErr); 16] = [
        ("exp(1000)", musl_math::exp(1000.0), Overflow),
        ("exp(-1000)", musl_math::exp(-1000.0), Underflow),
        ("exp(-740) subnormal", musl_math::exp(-740.0), Underflow),
        ("exp(nan)", musl_math::exp(f64::NAN), Domain),
        ("exp(1)", musl_math::exp(1.0), None),
        ("exp2(2000)", musl_math::exp2(2000.0), Overflow),
        ("exp2(-1080)", musl_math::exp2(-1080.0), Underflow),
        ("log(0)", musl_math::log(0.0), Sing),
        ("log(-1)", musl_math::log(-1.0), Domain),
        ("log(inf)", musl_math::log(f64::INFINITY), None),
        ("log2(-0)", musl_math::log2(-0.0), Sing),
        ("pow(0, -1)", musl_math::pow(0.0, -1.0), Sing),
        ("pow(-1, 1.5)", musl_math::pow(-1.0, 1.5), Domain),
        ("pow(2, 1e100)", musl_math::pow(2.0, 1e100), Overflow),
        ("pow(1e100, -10)", musl_math::pow(1e100, -10.0), Underflow),
        ("pow(inf, -1)", musl_math::pow(f64::INFINITY, -1.0), None),
    ];
    let mut bad = 0;
    for (what, (_, got), want) in cases {
        if got != want {
            bad += 1;
            eprintln!("CLASS {what}: {got:?}, expected {want:?}");
        }
    }
    let f: [(&str, MathErr, MathErr); 4] = [
        ("powf(0, -1)", musl_math::powf(0.0, -1.0).1, Sing),
        ("powf(-2, 0.5)", musl_math::powf(-2.0, 0.5).1, Domain),
        ("expf(100)", musl_math::expf(100.0).1, Overflow),
        ("logf(0)", musl_math::logf(0.0).1, Sing),
    ];
    for (what, got, want) in f {
        if got != want {
            bad += 1;
            eprintln!("CLASS {what}: {got:?}, expected {want:?}");
        }
    }
    bad
}

fn main() {
    let u = diff_unary();
    let p = diff_pow();
    let r = pow10_rounding();
    let c = classification();
    println!("unary mismatches: {u}, pow/powf mismatches: {p}, pow(10,n) off: {r}/601, classification: {c}");
    assert_eq!(u, 0, "unary functions differ from the musl C");
    assert_eq!(p, 0, "pow/powf differ from the musl C");
    assert!(r <= 3, "pow(10, n) misses {r} of 601 -- musl's misses 3");
    assert_eq!(c, 0, "error classification");
    // The line `vibeos-cli difftest` counts: one row per graded comparison family.
    println!("difftest result: ok. {} passed; 0 failed", 8 + 2 + 1 + 1);
}
