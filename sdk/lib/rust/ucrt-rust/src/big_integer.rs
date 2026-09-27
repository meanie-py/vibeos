//! `corecrt_internal_big_integer.h`: the fixed-size high-precision integer the
//! decimal <=> binary floating-point conversions run on.
//!
//! Wide enough for the reciprocal of the smallest denormal (2^-1074), plus the
//! 768 decimal digits the parser keeps (log2(10^768) = 2552 bits), plus 54 bits
//! of pre-division shift space: 3680 bits, 115 32-bit elements, little endian.
//!
//! Every operation that can overflow reports it and resets the value to zero,
//! as the C does, and only elements `[0, used)` are ever meaningful. The C leaves
//! the rest uninitialised; here they are zero, which no operation can observe.
//!
//! The powers-of-ten table (10^10 .. 10^380 in steps of ten) is COMPUTED at
//! compile time by `power_table()`, not transcribed: it was checked against
//! Microsoft's packed literal element for element, same offsets, same trimmed
//! zero counts, before the literal was left behind.

pub const ELEMENT_BITS: u32 = 32;
pub const ELEMENT_COUNT: usize = ((1074 + 2552 + 54 + ELEMENT_BITS - 1) / ELEMENT_BITS) as usize;

#[derive(Clone, Copy)]
pub struct BigInteger {
    pub used: u32,
    pub data: [u32; ELEMENT_COUNT],
}

impl BigInteger {
    pub const fn zero() -> Self {
        Self { used: 0, data: [0; ELEMENT_COUNT] }
    }

    /// `make_big_integer`.
    pub fn from_u64(value: u64) -> Self {
        let mut x = Self::zero();
        x.data[0] = value as u32;
        x.data[1] = (value >> 32) as u32;
        x.used = if x.data[1] == 0 { 1 } else { 2 };
        x
    }

    /// `operator<`.
    pub fn less_than(&self, rhs: &Self) -> bool {
        if self.used > rhs.used {
            return false;
        }
        if self.used < rhs.used {
            return true;
        }
        let mut i = self.used.wrapping_sub(1);
        while i != u32::MAX && self.data[i as usize] == rhs.data[i as usize] {
            i = i.wrapping_sub(1);
        }
        if i == u32::MAX {
            return false;
        }
        self.data[i as usize] <= rhs.data[i as usize]
    }
}

/// `bit_scan_reverse(uint32_t)`: the 1-based index of the highest set bit, 0 for 0.
pub fn bsr32(value: u32) -> u32 {
    32 - value.leading_zeros()
}

/// `bit_scan_reverse(uint64_t)`.
pub fn bsr64(value: u64) -> u32 {
    64 - value.leading_zeros()
}

/// `bit_scan_reverse(big_integer)`.
pub fn bsr(x: &BigInteger) -> u32 {
    if x.used == 0 {
        return 0;
    }
    (x.used - 1) * ELEMENT_BITS + bsr32(x.data[(x.used - 1) as usize])
}

/// `shift_left`. The C reads the top element through `_used - 1` without a
/// guard; every caller passes a nonzero value, and a zero one here reads as
/// "no bits" instead of indexing below the array.
pub fn shift_left(x: &mut BigInteger, n: u32) -> bool {
    let unit_shift = n / ELEMENT_BITS;
    let bit_shift = n % ELEMENT_BITS;

    let msb_bits = bit_shift;
    let lsb_bits = ELEMENT_BITS - msb_bits;
    let lsb_mask = ((1u64 << lsb_bits) - 1) as u32;
    let msb_mask = !lsb_mask;

    let top = if x.used == 0 { 0 } else { x.data[(x.used - 1) as usize] };
    let bit_shifts_into_next_unit = bit_shift > ELEMENT_BITS - bsr32(top);

    if x.used as usize + unit_shift as usize > ELEMENT_COUNT {
        *x = BigInteger::zero();
        return false;
    }
    let new_used = x.used + unit_shift + bit_shifts_into_next_unit as u32;
    if new_used as usize > ELEMENT_COUNT {
        *x = BigInteger::zero();
        return false;
    }

    let mut destination_index = new_used.wrapping_sub(1);
    while destination_index != unit_shift.wrapping_sub(1) {
        let upper_source_index = destination_index.wrapping_sub(unit_shift);
        let lower_source_index = destination_index.wrapping_sub(unit_shift).wrapping_sub(1);
        let upper_source = if upper_source_index < x.used { x.data[upper_source_index as usize] } else { 0 };
        let lower_source = if lower_source_index < x.used { x.data[lower_source_index as usize] } else { 0 };
        let shifted_upper_source = (upper_source & lsb_mask) << msb_bits;
        // With no bit shift this is a shift by 32 -- undefined in C, but its
        // operand is masked to zero first, so it is zero either way.
        let shifted_lower_source = (lower_source & msb_mask).checked_shr(lsb_bits).unwrap_or(0);
        x.data[destination_index as usize] = shifted_upper_source | shifted_lower_source;
        destination_index = destination_index.wrapping_sub(1);
    }
    for destination_index in 0..unit_shift {
        x.data[destination_index as usize] = 0;
    }
    x.used = new_used;
    true
}

/// `add(big_integer&, uint32_t)`.
pub fn add(x: &mut BigInteger, value: u32) -> bool {
    if value == 0 {
        return true;
    }
    let mut carry = value;
    for i in 0..x.used as usize {
        let result = x.data[i] as u64 + carry as u64;
        x.data[i] = result as u32;
        carry = (result >> 32) as u32;
    }
    if carry != 0 {
        if (x.used as usize) < ELEMENT_COUNT {
            x.data[x.used as usize] = carry;
            x.used += 1;
        } else {
            *x = BigInteger::zero();
            return false;
        }
    }
    true
}

/// `multiply(big_integer&, uint32_t)`.
pub fn multiply_small(multiplicand: &mut BigInteger, multiplier: u32) -> bool {
    if multiplier == 0 {
        *multiplicand = BigInteger::zero();
        return true;
    }
    if multiplier == 1 || multiplicand.used == 0 {
        return true;
    }
    let mut carry = 0u32;
    for i in 0..multiplicand.used as usize {
        let result = multiplicand.data[i] as u64 * multiplier as u64 + carry as u64;
        multiplicand.data[i] = result as u32;
        carry = (result >> 32) as u32;
    }
    if carry != 0 {
        if (multiplicand.used as usize) < ELEMENT_COUNT {
            multiplicand.data[multiplicand.used as usize] = carry;
            multiplicand.used += 1;
        } else {
            *multiplicand = BigInteger::zero();
            return false;
        }
    }
    true
}

/// `multiply(big_integer&, big_integer const&)` (after .NET's
/// BigIntegerBuilder.Mul, as the C says).
pub fn multiply(multiplicand: &mut BigInteger, multiplier: &BigInteger) -> bool {
    if multiplier.used <= 1 {
        return multiply_small(multiplicand, multiplier.data[0]);
    }
    if multiplicand.used <= 1 {
        let small_multiplier = multiplicand.data[0];
        *multiplicand = *multiplier;
        return multiply_small(multiplicand, small_multiplier);
    }

    // Read from a copy: the result replaces `multiplicand` at the end (or zeroes
    // it on overflow) while the loops still index both operands.
    let original = *multiplicand;
    let multiplier_is_shorter = multiplier.used < original.used;
    let (rgu1, rgu2, cu1, cu2) = if multiplier_is_shorter {
        (&multiplier.data, &original.data, multiplier.used, original.used)
    } else {
        (&original.data, &multiplier.data, original.used, multiplier.used)
    };

    let mut result = BigInteger::zero();
    for iu1 in 0..cu1 {
        let u_cur = rgu1[iu1 as usize];
        if u_cur == 0 {
            if iu1 == result.used {
                result.data[iu1 as usize] = 0;
                result.used = iu1 + 1;
            }
            continue;
        }

        let mut u_carry = 0u32;
        let mut iu_res = iu1;
        let mut iu2 = 0;
        while iu2 != cu2 && iu_res as usize != ELEMENT_COUNT {
            if iu_res == result.used {
                result.data[iu_res as usize] = 0;
                result.used = iu_res + 1;
            }
            // add_multiply_carry
            let slot = &mut result.data[iu_res as usize];
            let uu_res = u_cur as u64 * rgu2[iu2 as usize] as u64 + *slot as u64 + u_carry as u64;
            *slot = uu_res as u32;
            u_carry = (uu_res >> 32) as u32;
            iu2 += 1;
            iu_res += 1;
        }

        while u_carry != 0 && iu_res as usize != ELEMENT_COUNT {
            if iu_res == result.used {
                result.data[iu_res as usize] = 0;
                result.used = iu_res + 1;
            }
            // add_carry(result[iu_res++], 0, carry)
            let slot = &mut result.data[iu_res as usize];
            let uu = *slot as u64 + u_carry as u64;
            *slot = uu as u32;
            u_carry = (uu >> 32) as u32;
            iu_res += 1;
        }

        if iu_res as usize == ELEMENT_COUNT {
            *multiplicand = BigInteger::zero();
            return false;
        }
    }

    *multiplicand = result;
    true
}

/// Packed 10^10 .. 10^380: the concatenated significant elements, and per power
/// its `(offset, leading zero elements, elements present)`.
const POWER_DATA_LEN: usize = 578;
const POWER_COUNT: usize = 38;

struct PowerTable {
    data: [u32; POWER_DATA_LEN],
    indices: [(u16, u8, u8); POWER_COUNT],
}

/// The table, computed: multiply up from 1 by ten and record every tenth power
/// with its low zero elements trimmed. The const asserts are the sizes the
/// packed layout has; a wrong generator fails the build, not a conversion.
const fn power_table() -> PowerTable {
    let mut table = PowerTable { data: [0; POWER_DATA_LEN], indices: [(0, 0, 0); POWER_COUNT] };
    let mut x = [0u32; 40];
    x[0] = 1;
    let mut used = 1usize;
    let mut offset = 0usize;
    let mut power = 1;
    while power <= 380 {
        let mut carry = 0u64;
        let mut i = 0;
        while i < used {
            let r = x[i] as u64 * 10 + carry;
            x[i] = r as u32;
            carry = r >> 32;
            i += 1;
        }
        if carry != 0 {
            x[used] = carry as u32;
            used += 1;
        }
        if power % 10 == 0 {
            let mut zeroes = 0;
            while x[zeroes] == 0 {
                zeroes += 1;
            }
            let size = used - zeroes;
            table.indices[power / 10 - 1] = (offset as u16, zeroes as u8, size as u8);
            let mut j = 0;
            while j < size {
                table.data[offset + j] = x[zeroes + j];
                j += 1;
            }
            offset += size;
        }
        power += 1;
    }
    assert!(offset == POWER_DATA_LEN);
    table
}

static POWERS: PowerTable = power_table();

const SMALL_POWERS_OF_TEN: [u32; 9] =
    [10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000];

/// `multiply_by_power_of_ten`.
pub fn multiply_by_power_of_ten(x: &mut BigInteger, power: u32) -> bool {
    let mut large_power = power / 10;
    while large_power != 0 {
        let current_power = large_power.min(POWER_COUNT as u32);
        let (offset, zeroes, size) = POWERS.indices[(current_power - 1) as usize];
        let mut multiplier = BigInteger::zero();
        multiplier.used = size as u32 + zeroes as u32;
        let source = &POWERS.data[offset as usize..offset as usize + size as usize];
        multiplier.data[zeroes as usize..zeroes as usize + size as usize].copy_from_slice(source);
        if !multiply(x, &multiplier) {
            *x = BigInteger::zero();
            return false;
        }
        large_power -= current_power;
    }
    let small_power = power % 10;
    if small_power != 0 && !multiply_small(x, SMALL_POWERS_OF_TEN[(small_power - 1) as usize]) {
        return false;
    }
    true
}

/// `count_sequential_high_zeroes`.
fn count_sequential_high_zeroes(u: u32) -> u32 {
    u.leading_zeros()
}

/// `divide` (after .NET's BigIntegerBuilder.ModDivCore): the quotient comes
/// back, the remainder is left in `numerator`.
pub fn divide(numerator: &mut BigInteger, denominator: &BigInteger) -> u64 {
    if numerator.used == 0 || denominator.used == 0 {
        return 0;
    }

    let mut max_numerator_element_index = numerator.used - 1;
    let max_denominator_element_index = denominator.used - 1;

    if max_denominator_element_index == 0 {
        let small_denominator = denominator.data[0];
        if small_denominator == 1 {
            let quotient = numerator.data[0];
            *numerator = BigInteger::zero();
            return quotient as u64;
        }
        if max_numerator_element_index == 0 {
            let small_numerator = numerator.data[0];
            *numerator = BigInteger::zero();
            numerator.data[0] = small_numerator % small_denominator;
            numerator.used = if numerator.data[0] > 0 { 1 } else { 0 };
            return (small_numerator / small_denominator) as u64;
        }
        let mut quotient = 0u64;
        let mut uu = 0u64;
        let mut iv = max_numerator_element_index;
        while iv != u32::MAX {
            uu = (uu << 32) | numerator.data[iv as usize] as u64;
            quotient = (quotient << 32).wrapping_add((uu / small_denominator as u64) as u32 as u64);
            uu %= small_denominator as u64;
            iv = iv.wrapping_sub(1);
        }
        *numerator = BigInteger::zero();
        numerator.data[1] = (uu >> 32) as u32;
        numerator.data[0] = uu as u32;
        numerator.used = if numerator.data[1] > 0 { 2 } else { 1 };
        return quotient;
    }

    if max_denominator_element_index > max_numerator_element_index {
        return 0;
    }

    let cu_den = max_denominator_element_index + 1;
    let cu_diff = (max_numerator_element_index - max_denominator_element_index) as i32;

    let mut cu_quo = cu_diff;
    let mut iu = max_numerator_element_index as i32;
    loop {
        if iu < cu_diff {
            cu_quo += 1;
            break;
        }
        let d = denominator.data[(iu - cu_diff) as usize];
        let n = numerator.data[iu as usize];
        if d != n {
            if d < n {
                cu_quo += 1;
            }
            break;
        }
        iu -= 1;
    }
    if cu_quo == 0 {
        return 0;
    }

    let mut u_den = denominator.data[(cu_den - 1) as usize];
    let mut u_den_next = denominator.data[(cu_den - 2) as usize];
    let cbit_shift_left = count_sequential_high_zeroes(u_den);
    let cbit_shift_right = 32 - cbit_shift_left;
    if cbit_shift_left > 0 {
        u_den = (u_den << cbit_shift_left) | (u_den_next >> cbit_shift_right);
        u_den_next <<= cbit_shift_left;
        if cu_den > 2 {
            u_den_next |= denominator.data[(cu_den - 3) as usize] >> cbit_shift_right;
        }
    }

    let mut quotient = 0u64;
    let mut iu = cu_quo;
    loop {
        iu -= 1;
        if iu < 0 {
            break;
        }
        let iuu = iu as u32;
        let u_num_hi = if iuu + cu_den <= max_numerator_element_index {
            numerator.data[(iuu + cu_den) as usize]
        } else {
            0
        };
        let mut uu_num = ((u_num_hi as u64) << 32) | numerator.data[(iuu + cu_den - 1) as usize] as u64;
        let mut u_num_next = numerator.data[(iuu + cu_den - 2) as usize];
        if cbit_shift_left > 0 {
            uu_num = (uu_num << cbit_shift_left) | (u_num_next >> cbit_shift_right) as u64;
            u_num_next <<= cbit_shift_left;
            if iuu + cu_den >= 3 {
                u_num_next |= numerator.data[(iuu + cu_den - 3) as usize] >> cbit_shift_right;
            }
        }

        let mut uu_quo = uu_num / u_den as u64;
        let mut uu_rem = (uu_num % u_den as u64) as u32 as u64;
        if uu_quo > u32::MAX as u64 {
            uu_rem = uu_rem.wrapping_add((u_den as u64).wrapping_mul(uu_quo - u32::MAX as u64));
            uu_quo = u32::MAX as u64;
        }
        while uu_rem <= u32::MAX as u64
            && uu_quo.wrapping_mul(u_den_next as u64) > ((uu_rem << 32) | u_num_next as u64)
        {
            uu_quo -= 1;
            uu_rem += u_den as u64;
        }

        if uu_quo > 0 {
            let mut uu_borrow = 0u64;
            for iu2 in 0..cu_den {
                // multiply_64_32: the low 64 bits of the product.
                uu_borrow = uu_borrow.wrapping_add(uu_quo.wrapping_mul(denominator.data[iu2 as usize] as u64));
                let u_sub = uu_borrow as u32;
                uu_borrow >>= 32;
                let slot = &mut numerator.data[(iuu + iu2) as usize];
                if *slot < u_sub {
                    uu_borrow += 1;
                }
                *slot = slot.wrapping_sub(u_sub);
            }
            if (u_num_hi as u64) < uu_borrow {
                let mut u_carry = 0u32;
                for iu2 in 0..cu_den {
                    let slot = &mut numerator.data[(iuu + iu2) as usize];
                    let sum = *slot as u64 + denominator.data[iu2 as usize] as u64 + u_carry as u64;
                    *slot = sum as u32;
                    u_carry = (sum >> 32) as u32;
                }
                uu_quo -= 1;
            }
            max_numerator_element_index = iuu + cu_den - 1;
        }

        quotient = (quotient << 32).wrapping_add(uu_quo as u32 as u64);
    }

    for i in (max_numerator_element_index + 1)..numerator.used {
        numerator.data[i as usize] = 0;
    }
    numerator.used = max_numerator_element_index + 1;
    while numerator.used != 0 && numerator.data[(numerator.used - 1) as usize] == 0 {
        numerator.used -= 1;
    }
    quotient
}
