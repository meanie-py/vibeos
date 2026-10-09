//! mvaesalg.rs — Rijndael reference implementation (Marvell/mvAesAlg.c).
//! Cohabitation module. Public ABI: cdecl (extern "C").
//! Tables (mask, Logtable, Alogtable, S, Si, rcon) sourced from c2rust (bit-exact).
//! Helper fns (mul1, KeyAddition, ShiftRow*, Substitution, MixColumn, InvMixColumn)
//! sourced from c2rust transpilation.
//!
//! FIXES applied:
//!   - Removed garbage preamble from LLM body (failed preamble extraction).
//!   - KeyAddition: was extern "C"; now local unsafe fn (same-module calls).
//!   - Added pub to rijndaelDecrypt128 (was missing).
//!   - S/Si tables: taken from c2rust (not LLM-fabricated).
//!   - rcon: mvAesAlg version has 30 entries (different from aes.c rcon).

use crate::types::*;

// `mul` macro from mvAesAlg.c:
// #define mul(aa, bb) (mask[bb] & Alogtable[aa + Logtable[bb]])
#[inline(always)]
unsafe fn mul(aa: u8, bb: u8) -> u8 {
    mask[bb as usize] & Alogtable[(aa as usize).wrapping_add(Logtable[bb as usize] as usize)]
}

static mask: [MV_U8; 256] = [
    0 as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
    0xff as i32 as MV_U8,
];
static Logtable: [MV_U8; 256] = [
    0 as i32 as MV_U8,
    0 as i32 as MV_U8,
    25 as i32 as MV_U8,
    1 as i32 as MV_U8,
    50 as i32 as MV_U8,
    2 as i32 as MV_U8,
    26 as i32 as MV_U8,
    198 as i32 as MV_U8,
    75 as i32 as MV_U8,
    199 as i32 as MV_U8,
    27 as i32 as MV_U8,
    104 as i32 as MV_U8,
    51 as i32 as MV_U8,
    238 as i32 as MV_U8,
    223 as i32 as MV_U8,
    3 as i32 as MV_U8,
    100 as i32 as MV_U8,
    4 as i32 as MV_U8,
    224 as i32 as MV_U8,
    14 as i32 as MV_U8,
    52 as i32 as MV_U8,
    141 as i32 as MV_U8,
    129 as i32 as MV_U8,
    239 as i32 as MV_U8,
    76 as i32 as MV_U8,
    113 as i32 as MV_U8,
    8 as i32 as MV_U8,
    200 as i32 as MV_U8,
    248 as i32 as MV_U8,
    105 as i32 as MV_U8,
    28 as i32 as MV_U8,
    193 as i32 as MV_U8,
    125 as i32 as MV_U8,
    194 as i32 as MV_U8,
    29 as i32 as MV_U8,
    181 as i32 as MV_U8,
    249 as i32 as MV_U8,
    185 as i32 as MV_U8,
    39 as i32 as MV_U8,
    106 as i32 as MV_U8,
    77 as i32 as MV_U8,
    228 as i32 as MV_U8,
    166 as i32 as MV_U8,
    114 as i32 as MV_U8,
    154 as i32 as MV_U8,
    201 as i32 as MV_U8,
    9 as i32 as MV_U8,
    120 as i32 as MV_U8,
    101 as i32 as MV_U8,
    47 as i32 as MV_U8,
    138 as i32 as MV_U8,
    5 as i32 as MV_U8,
    33 as i32 as MV_U8,
    15 as i32 as MV_U8,
    225 as i32 as MV_U8,
    36 as i32 as MV_U8,
    18 as i32 as MV_U8,
    240 as i32 as MV_U8,
    130 as i32 as MV_U8,
    69 as i32 as MV_U8,
    53 as i32 as MV_U8,
    147 as i32 as MV_U8,
    218 as i32 as MV_U8,
    142 as i32 as MV_U8,
    150 as i32 as MV_U8,
    143 as i32 as MV_U8,
    219 as i32 as MV_U8,
    189 as i32 as MV_U8,
    54 as i32 as MV_U8,
    208 as i32 as MV_U8,
    206 as i32 as MV_U8,
    148 as i32 as MV_U8,
    19 as i32 as MV_U8,
    92 as i32 as MV_U8,
    210 as i32 as MV_U8,
    241 as i32 as MV_U8,
    64 as i32 as MV_U8,
    70 as i32 as MV_U8,
    131 as i32 as MV_U8,
    56 as i32 as MV_U8,
    102 as i32 as MV_U8,
    221 as i32 as MV_U8,
    253 as i32 as MV_U8,
    48 as i32 as MV_U8,
    191 as i32 as MV_U8,
    6 as i32 as MV_U8,
    139 as i32 as MV_U8,
    98 as i32 as MV_U8,
    179 as i32 as MV_U8,
    37 as i32 as MV_U8,
    226 as i32 as MV_U8,
    152 as i32 as MV_U8,
    34 as i32 as MV_U8,
    136 as i32 as MV_U8,
    145 as i32 as MV_U8,
    16 as i32 as MV_U8,
    126 as i32 as MV_U8,
    110 as i32 as MV_U8,
    72 as i32 as MV_U8,
    195 as i32 as MV_U8,
    163 as i32 as MV_U8,
    182 as i32 as MV_U8,
    30 as i32 as MV_U8,
    66 as i32 as MV_U8,
    58 as i32 as MV_U8,
    107 as i32 as MV_U8,
    40 as i32 as MV_U8,
    84 as i32 as MV_U8,
    250 as i32 as MV_U8,
    133 as i32 as MV_U8,
    61 as i32 as MV_U8,
    186 as i32 as MV_U8,
    43 as i32 as MV_U8,
    121 as i32 as MV_U8,
    10 as i32 as MV_U8,
    21 as i32 as MV_U8,
    155 as i32 as MV_U8,
    159 as i32 as MV_U8,
    94 as i32 as MV_U8,
    202 as i32 as MV_U8,
    78 as i32 as MV_U8,
    212 as i32 as MV_U8,
    172 as i32 as MV_U8,
    229 as i32 as MV_U8,
    243 as i32 as MV_U8,
    115 as i32 as MV_U8,
    167 as i32 as MV_U8,
    87 as i32 as MV_U8,
    175 as i32 as MV_U8,
    88 as i32 as MV_U8,
    168 as i32 as MV_U8,
    80 as i32 as MV_U8,
    244 as i32 as MV_U8,
    234 as i32 as MV_U8,
    214 as i32 as MV_U8,
    116 as i32 as MV_U8,
    79 as i32 as MV_U8,
    174 as i32 as MV_U8,
    233 as i32 as MV_U8,
    213 as i32 as MV_U8,
    231 as i32 as MV_U8,
    230 as i32 as MV_U8,
    173 as i32 as MV_U8,
    232 as i32 as MV_U8,
    44 as i32 as MV_U8,
    215 as i32 as MV_U8,
    117 as i32 as MV_U8,
    122 as i32 as MV_U8,
    235 as i32 as MV_U8,
    22 as i32 as MV_U8,
    11 as i32 as MV_U8,
    245 as i32 as MV_U8,
    89 as i32 as MV_U8,
    203 as i32 as MV_U8,
    95 as i32 as MV_U8,
    176 as i32 as MV_U8,
    156 as i32 as MV_U8,
    169 as i32 as MV_U8,
    81 as i32 as MV_U8,
    160 as i32 as MV_U8,
    127 as i32 as MV_U8,
    12 as i32 as MV_U8,
    246 as i32 as MV_U8,
    111 as i32 as MV_U8,
    23 as i32 as MV_U8,
    196 as i32 as MV_U8,
    73 as i32 as MV_U8,
    236 as i32 as MV_U8,
    216 as i32 as MV_U8,
    67 as i32 as MV_U8,
    31 as i32 as MV_U8,
    45 as i32 as MV_U8,
    164 as i32 as MV_U8,
    118 as i32 as MV_U8,
    123 as i32 as MV_U8,
    183 as i32 as MV_U8,
    204 as i32 as MV_U8,
    187 as i32 as MV_U8,
    62 as i32 as MV_U8,
    90 as i32 as MV_U8,
    251 as i32 as MV_U8,
    96 as i32 as MV_U8,
    177 as i32 as MV_U8,
    134 as i32 as MV_U8,
    59 as i32 as MV_U8,
    82 as i32 as MV_U8,
    161 as i32 as MV_U8,
    108 as i32 as MV_U8,
    170 as i32 as MV_U8,
    85 as i32 as MV_U8,
    41 as i32 as MV_U8,
    157 as i32 as MV_U8,
    151 as i32 as MV_U8,
    178 as i32 as MV_U8,
    135 as i32 as MV_U8,
    144 as i32 as MV_U8,
    97 as i32 as MV_U8,
    190 as i32 as MV_U8,
    220 as i32 as MV_U8,
    252 as i32 as MV_U8,
    188 as i32 as MV_U8,
    149 as i32 as MV_U8,
    207 as i32 as MV_U8,
    205 as i32 as MV_U8,
    55 as i32 as MV_U8,
    63 as i32 as MV_U8,
    91 as i32 as MV_U8,
    209 as i32 as MV_U8,
    83 as i32 as MV_U8,
    57 as i32 as MV_U8,
    132 as i32 as MV_U8,
    60 as i32 as MV_U8,
    65 as i32 as MV_U8,
    162 as i32 as MV_U8,
    109 as i32 as MV_U8,
    71 as i32 as MV_U8,
    20 as i32 as MV_U8,
    42 as i32 as MV_U8,
    158 as i32 as MV_U8,
    93 as i32 as MV_U8,
    86 as i32 as MV_U8,
    242 as i32 as MV_U8,
    211 as i32 as MV_U8,
    171 as i32 as MV_U8,
    68 as i32 as MV_U8,
    17 as i32 as MV_U8,
    146 as i32 as MV_U8,
    217 as i32 as MV_U8,
    35 as i32 as MV_U8,
    32 as i32 as MV_U8,
    46 as i32 as MV_U8,
    137 as i32 as MV_U8,
    180 as i32 as MV_U8,
    124 as i32 as MV_U8,
    184 as i32 as MV_U8,
    38 as i32 as MV_U8,
    119 as i32 as MV_U8,
    153 as i32 as MV_U8,
    227 as i32 as MV_U8,
    165 as i32 as MV_U8,
    103 as i32 as MV_U8,
    74 as i32 as MV_U8,
    237 as i32 as MV_U8,
    222 as i32 as MV_U8,
    197 as i32 as MV_U8,
    49 as i32 as MV_U8,
    254 as i32 as MV_U8,
    24 as i32 as MV_U8,
    13 as i32 as MV_U8,
    99 as i32 as MV_U8,
    140 as i32 as MV_U8,
    128 as i32 as MV_U8,
    192 as i32 as MV_U8,
    247 as i32 as MV_U8,
    112 as i32 as MV_U8,
    7 as i32 as MV_U8,
];
static Alogtable: [MV_U8; 512] = [
    1 as i32 as MV_U8,
    3 as i32 as MV_U8,
    5 as i32 as MV_U8,
    15 as i32 as MV_U8,
    17 as i32 as MV_U8,
    51 as i32 as MV_U8,
    85 as i32 as MV_U8,
    255 as i32 as MV_U8,
    26 as i32 as MV_U8,
    46 as i32 as MV_U8,
    114 as i32 as MV_U8,
    150 as i32 as MV_U8,
    161 as i32 as MV_U8,
    248 as i32 as MV_U8,
    19 as i32 as MV_U8,
    53 as i32 as MV_U8,
    95 as i32 as MV_U8,
    225 as i32 as MV_U8,
    56 as i32 as MV_U8,
    72 as i32 as MV_U8,
    216 as i32 as MV_U8,
    115 as i32 as MV_U8,
    149 as i32 as MV_U8,
    164 as i32 as MV_U8,
    247 as i32 as MV_U8,
    2 as i32 as MV_U8,
    6 as i32 as MV_U8,
    10 as i32 as MV_U8,
    30 as i32 as MV_U8,
    34 as i32 as MV_U8,
    102 as i32 as MV_U8,
    170 as i32 as MV_U8,
    229 as i32 as MV_U8,
    52 as i32 as MV_U8,
    92 as i32 as MV_U8,
    228 as i32 as MV_U8,
    55 as i32 as MV_U8,
    89 as i32 as MV_U8,
    235 as i32 as MV_U8,
    38 as i32 as MV_U8,
    106 as i32 as MV_U8,
    190 as i32 as MV_U8,
    217 as i32 as MV_U8,
    112 as i32 as MV_U8,
    144 as i32 as MV_U8,
    171 as i32 as MV_U8,
    230 as i32 as MV_U8,
    49 as i32 as MV_U8,
    83 as i32 as MV_U8,
    245 as i32 as MV_U8,
    4 as i32 as MV_U8,
    12 as i32 as MV_U8,
    20 as i32 as MV_U8,
    60 as i32 as MV_U8,
    68 as i32 as MV_U8,
    204 as i32 as MV_U8,
    79 as i32 as MV_U8,
    209 as i32 as MV_U8,
    104 as i32 as MV_U8,
    184 as i32 as MV_U8,
    211 as i32 as MV_U8,
    110 as i32 as MV_U8,
    178 as i32 as MV_U8,
    205 as i32 as MV_U8,
    76 as i32 as MV_U8,
    212 as i32 as MV_U8,
    103 as i32 as MV_U8,
    169 as i32 as MV_U8,
    224 as i32 as MV_U8,
    59 as i32 as MV_U8,
    77 as i32 as MV_U8,
    215 as i32 as MV_U8,
    98 as i32 as MV_U8,
    166 as i32 as MV_U8,
    241 as i32 as MV_U8,
    8 as i32 as MV_U8,
    24 as i32 as MV_U8,
    40 as i32 as MV_U8,
    120 as i32 as MV_U8,
    136 as i32 as MV_U8,
    131 as i32 as MV_U8,
    158 as i32 as MV_U8,
    185 as i32 as MV_U8,
    208 as i32 as MV_U8,
    107 as i32 as MV_U8,
    189 as i32 as MV_U8,
    220 as i32 as MV_U8,
    127 as i32 as MV_U8,
    129 as i32 as MV_U8,
    152 as i32 as MV_U8,
    179 as i32 as MV_U8,
    206 as i32 as MV_U8,
    73 as i32 as MV_U8,
    219 as i32 as MV_U8,
    118 as i32 as MV_U8,
    154 as i32 as MV_U8,
    181 as i32 as MV_U8,
    196 as i32 as MV_U8,
    87 as i32 as MV_U8,
    249 as i32 as MV_U8,
    16 as i32 as MV_U8,
    48 as i32 as MV_U8,
    80 as i32 as MV_U8,
    240 as i32 as MV_U8,
    11 as i32 as MV_U8,
    29 as i32 as MV_U8,
    39 as i32 as MV_U8,
    105 as i32 as MV_U8,
    187 as i32 as MV_U8,
    214 as i32 as MV_U8,
    97 as i32 as MV_U8,
    163 as i32 as MV_U8,
    254 as i32 as MV_U8,
    25 as i32 as MV_U8,
    43 as i32 as MV_U8,
    125 as i32 as MV_U8,
    135 as i32 as MV_U8,
    146 as i32 as MV_U8,
    173 as i32 as MV_U8,
    236 as i32 as MV_U8,
    47 as i32 as MV_U8,
    113 as i32 as MV_U8,
    147 as i32 as MV_U8,
    174 as i32 as MV_U8,
    233 as i32 as MV_U8,
    32 as i32 as MV_U8,
    96 as i32 as MV_U8,
    160 as i32 as MV_U8,
    251 as i32 as MV_U8,
    22 as i32 as MV_U8,
    58 as i32 as MV_U8,
    78 as i32 as MV_U8,
    210 as i32 as MV_U8,
    109 as i32 as MV_U8,
    183 as i32 as MV_U8,
    194 as i32 as MV_U8,
    93 as i32 as MV_U8,
    231 as i32 as MV_U8,
    50 as i32 as MV_U8,
    86 as i32 as MV_U8,
    250 as i32 as MV_U8,
    21 as i32 as MV_U8,
    63 as i32 as MV_U8,
    65 as i32 as MV_U8,
    195 as i32 as MV_U8,
    94 as i32 as MV_U8,
    226 as i32 as MV_U8,
    61 as i32 as MV_U8,
    71 as i32 as MV_U8,
    201 as i32 as MV_U8,
    64 as i32 as MV_U8,
    192 as i32 as MV_U8,
    91 as i32 as MV_U8,
    237 as i32 as MV_U8,
    44 as i32 as MV_U8,
    116 as i32 as MV_U8,
    156 as i32 as MV_U8,
    191 as i32 as MV_U8,
    218 as i32 as MV_U8,
    117 as i32 as MV_U8,
    159 as i32 as MV_U8,
    186 as i32 as MV_U8,
    213 as i32 as MV_U8,
    100 as i32 as MV_U8,
    172 as i32 as MV_U8,
    239 as i32 as MV_U8,
    42 as i32 as MV_U8,
    126 as i32 as MV_U8,
    130 as i32 as MV_U8,
    157 as i32 as MV_U8,
    188 as i32 as MV_U8,
    223 as i32 as MV_U8,
    122 as i32 as MV_U8,
    142 as i32 as MV_U8,
    137 as i32 as MV_U8,
    128 as i32 as MV_U8,
    155 as i32 as MV_U8,
    182 as i32 as MV_U8,
    193 as i32 as MV_U8,
    88 as i32 as MV_U8,
    232 as i32 as MV_U8,
    35 as i32 as MV_U8,
    101 as i32 as MV_U8,
    175 as i32 as MV_U8,
    234 as i32 as MV_U8,
    37 as i32 as MV_U8,
    111 as i32 as MV_U8,
    177 as i32 as MV_U8,
    200 as i32 as MV_U8,
    67 as i32 as MV_U8,
    197 as i32 as MV_U8,
    84 as i32 as MV_U8,
    252 as i32 as MV_U8,
    31 as i32 as MV_U8,
    33 as i32 as MV_U8,
    99 as i32 as MV_U8,
    165 as i32 as MV_U8,
    244 as i32 as MV_U8,
    7 as i32 as MV_U8,
    9 as i32 as MV_U8,
    27 as i32 as MV_U8,
    45 as i32 as MV_U8,
    119 as i32 as MV_U8,
    153 as i32 as MV_U8,
    176 as i32 as MV_U8,
    203 as i32 as MV_U8,
    70 as i32 as MV_U8,
    202 as i32 as MV_U8,
    69 as i32 as MV_U8,
    207 as i32 as MV_U8,
    74 as i32 as MV_U8,
    222 as i32 as MV_U8,
    121 as i32 as MV_U8,
    139 as i32 as MV_U8,
    134 as i32 as MV_U8,
    145 as i32 as MV_U8,
    168 as i32 as MV_U8,
    227 as i32 as MV_U8,
    62 as i32 as MV_U8,
    66 as i32 as MV_U8,
    198 as i32 as MV_U8,
    81 as i32 as MV_U8,
    243 as i32 as MV_U8,
    14 as i32 as MV_U8,
    18 as i32 as MV_U8,
    54 as i32 as MV_U8,
    90 as i32 as MV_U8,
    238 as i32 as MV_U8,
    41 as i32 as MV_U8,
    123 as i32 as MV_U8,
    141 as i32 as MV_U8,
    140 as i32 as MV_U8,
    143 as i32 as MV_U8,
    138 as i32 as MV_U8,
    133 as i32 as MV_U8,
    148 as i32 as MV_U8,
    167 as i32 as MV_U8,
    242 as i32 as MV_U8,
    13 as i32 as MV_U8,
    23 as i32 as MV_U8,
    57 as i32 as MV_U8,
    75 as i32 as MV_U8,
    221 as i32 as MV_U8,
    124 as i32 as MV_U8,
    132 as i32 as MV_U8,
    151 as i32 as MV_U8,
    162 as i32 as MV_U8,
    253 as i32 as MV_U8,
    28 as i32 as MV_U8,
    36 as i32 as MV_U8,
    108 as i32 as MV_U8,
    180 as i32 as MV_U8,
    199 as i32 as MV_U8,
    82 as i32 as MV_U8,
    246 as i32 as MV_U8,
    1 as i32 as MV_U8,
    3 as i32 as MV_U8,
    5 as i32 as MV_U8,
    15 as i32 as MV_U8,
    17 as i32 as MV_U8,
    51 as i32 as MV_U8,
    85 as i32 as MV_U8,
    255 as i32 as MV_U8,
    26 as i32 as MV_U8,
    46 as i32 as MV_U8,
    114 as i32 as MV_U8,
    150 as i32 as MV_U8,
    161 as i32 as MV_U8,
    248 as i32 as MV_U8,
    19 as i32 as MV_U8,
    53 as i32 as MV_U8,
    95 as i32 as MV_U8,
    225 as i32 as MV_U8,
    56 as i32 as MV_U8,
    72 as i32 as MV_U8,
    216 as i32 as MV_U8,
    115 as i32 as MV_U8,
    149 as i32 as MV_U8,
    164 as i32 as MV_U8,
    247 as i32 as MV_U8,
    2 as i32 as MV_U8,
    6 as i32 as MV_U8,
    10 as i32 as MV_U8,
    30 as i32 as MV_U8,
    34 as i32 as MV_U8,
    102 as i32 as MV_U8,
    170 as i32 as MV_U8,
    229 as i32 as MV_U8,
    52 as i32 as MV_U8,
    92 as i32 as MV_U8,
    228 as i32 as MV_U8,
    55 as i32 as MV_U8,
    89 as i32 as MV_U8,
    235 as i32 as MV_U8,
    38 as i32 as MV_U8,
    106 as i32 as MV_U8,
    190 as i32 as MV_U8,
    217 as i32 as MV_U8,
    112 as i32 as MV_U8,
    144 as i32 as MV_U8,
    171 as i32 as MV_U8,
    230 as i32 as MV_U8,
    49 as i32 as MV_U8,
    83 as i32 as MV_U8,
    245 as i32 as MV_U8,
    4 as i32 as MV_U8,
    12 as i32 as MV_U8,
    20 as i32 as MV_U8,
    60 as i32 as MV_U8,
    68 as i32 as MV_U8,
    204 as i32 as MV_U8,
    79 as i32 as MV_U8,
    209 as i32 as MV_U8,
    104 as i32 as MV_U8,
    184 as i32 as MV_U8,
    211 as i32 as MV_U8,
    110 as i32 as MV_U8,
    178 as i32 as MV_U8,
    205 as i32 as MV_U8,
    76 as i32 as MV_U8,
    212 as i32 as MV_U8,
    103 as i32 as MV_U8,
    169 as i32 as MV_U8,
    224 as i32 as MV_U8,
    59 as i32 as MV_U8,
    77 as i32 as MV_U8,
    215 as i32 as MV_U8,
    98 as i32 as MV_U8,
    166 as i32 as MV_U8,
    241 as i32 as MV_U8,
    8 as i32 as MV_U8,
    24 as i32 as MV_U8,
    40 as i32 as MV_U8,
    120 as i32 as MV_U8,
    136 as i32 as MV_U8,
    131 as i32 as MV_U8,
    158 as i32 as MV_U8,
    185 as i32 as MV_U8,
    208 as i32 as MV_U8,
    107 as i32 as MV_U8,
    189 as i32 as MV_U8,
    220 as i32 as MV_U8,
    127 as i32 as MV_U8,
    129 as i32 as MV_U8,
    152 as i32 as MV_U8,
    179 as i32 as MV_U8,
    206 as i32 as MV_U8,
    73 as i32 as MV_U8,
    219 as i32 as MV_U8,
    118 as i32 as MV_U8,
    154 as i32 as MV_U8,
    181 as i32 as MV_U8,
    196 as i32 as MV_U8,
    87 as i32 as MV_U8,
    249 as i32 as MV_U8,
    16 as i32 as MV_U8,
    48 as i32 as MV_U8,
    80 as i32 as MV_U8,
    240 as i32 as MV_U8,
    11 as i32 as MV_U8,
    29 as i32 as MV_U8,
    39 as i32 as MV_U8,
    105 as i32 as MV_U8,
    187 as i32 as MV_U8,
    214 as i32 as MV_U8,
    97 as i32 as MV_U8,
    163 as i32 as MV_U8,
    254 as i32 as MV_U8,
    25 as i32 as MV_U8,
    43 as i32 as MV_U8,
    125 as i32 as MV_U8,
    135 as i32 as MV_U8,
    146 as i32 as MV_U8,
    173 as i32 as MV_U8,
    236 as i32 as MV_U8,
    47 as i32 as MV_U8,
    113 as i32 as MV_U8,
    147 as i32 as MV_U8,
    174 as i32 as MV_U8,
    233 as i32 as MV_U8,
    32 as i32 as MV_U8,
    96 as i32 as MV_U8,
    160 as i32 as MV_U8,
    251 as i32 as MV_U8,
    22 as i32 as MV_U8,
    58 as i32 as MV_U8,
    78 as i32 as MV_U8,
    210 as i32 as MV_U8,
    109 as i32 as MV_U8,
    183 as i32 as MV_U8,
    194 as i32 as MV_U8,
    93 as i32 as MV_U8,
    231 as i32 as MV_U8,
    50 as i32 as MV_U8,
    86 as i32 as MV_U8,
    250 as i32 as MV_U8,
    21 as i32 as MV_U8,
    63 as i32 as MV_U8,
    65 as i32 as MV_U8,
    195 as i32 as MV_U8,
    94 as i32 as MV_U8,
    226 as i32 as MV_U8,
    61 as i32 as MV_U8,
    71 as i32 as MV_U8,
    201 as i32 as MV_U8,
    64 as i32 as MV_U8,
    192 as i32 as MV_U8,
    91 as i32 as MV_U8,
    237 as i32 as MV_U8,
    44 as i32 as MV_U8,
    116 as i32 as MV_U8,
    156 as i32 as MV_U8,
    191 as i32 as MV_U8,
    218 as i32 as MV_U8,
    117 as i32 as MV_U8,
    159 as i32 as MV_U8,
    186 as i32 as MV_U8,
    213 as i32 as MV_U8,
    100 as i32 as MV_U8,
    172 as i32 as MV_U8,
    239 as i32 as MV_U8,
    42 as i32 as MV_U8,
    126 as i32 as MV_U8,
    130 as i32 as MV_U8,
    157 as i32 as MV_U8,
    188 as i32 as MV_U8,
    223 as i32 as MV_U8,
    122 as i32 as MV_U8,
    142 as i32 as MV_U8,
    137 as i32 as MV_U8,
    128 as i32 as MV_U8,
    155 as i32 as MV_U8,
    182 as i32 as MV_U8,
    193 as i32 as MV_U8,
    88 as i32 as MV_U8,
    232 as i32 as MV_U8,
    35 as i32 as MV_U8,
    101 as i32 as MV_U8,
    175 as i32 as MV_U8,
    234 as i32 as MV_U8,
    37 as i32 as MV_U8,
    111 as i32 as MV_U8,
    177 as i32 as MV_U8,
    200 as i32 as MV_U8,
    67 as i32 as MV_U8,
    197 as i32 as MV_U8,
    84 as i32 as MV_U8,
    252 as i32 as MV_U8,
    31 as i32 as MV_U8,
    33 as i32 as MV_U8,
    99 as i32 as MV_U8,
    165 as i32 as MV_U8,
    244 as i32 as MV_U8,
    7 as i32 as MV_U8,
    9 as i32 as MV_U8,
    27 as i32 as MV_U8,
    45 as i32 as MV_U8,
    119 as i32 as MV_U8,
    153 as i32 as MV_U8,
    176 as i32 as MV_U8,
    203 as i32 as MV_U8,
    70 as i32 as MV_U8,
    202 as i32 as MV_U8,
    69 as i32 as MV_U8,
    207 as i32 as MV_U8,
    74 as i32 as MV_U8,
    222 as i32 as MV_U8,
    121 as i32 as MV_U8,
    139 as i32 as MV_U8,
    134 as i32 as MV_U8,
    145 as i32 as MV_U8,
    168 as i32 as MV_U8,
    227 as i32 as MV_U8,
    62 as i32 as MV_U8,
    66 as i32 as MV_U8,
    198 as i32 as MV_U8,
    81 as i32 as MV_U8,
    243 as i32 as MV_U8,
    14 as i32 as MV_U8,
    18 as i32 as MV_U8,
    54 as i32 as MV_U8,
    90 as i32 as MV_U8,
    238 as i32 as MV_U8,
    41 as i32 as MV_U8,
    123 as i32 as MV_U8,
    141 as i32 as MV_U8,
    140 as i32 as MV_U8,
    143 as i32 as MV_U8,
    138 as i32 as MV_U8,
    133 as i32 as MV_U8,
    148 as i32 as MV_U8,
    167 as i32 as MV_U8,
    242 as i32 as MV_U8,
    13 as i32 as MV_U8,
    23 as i32 as MV_U8,
    57 as i32 as MV_U8,
    75 as i32 as MV_U8,
    221 as i32 as MV_U8,
    124 as i32 as MV_U8,
    132 as i32 as MV_U8,
    151 as i32 as MV_U8,
    162 as i32 as MV_U8,
    253 as i32 as MV_U8,
    28 as i32 as MV_U8,
    36 as i32 as MV_U8,
    108 as i32 as MV_U8,
    180 as i32 as MV_U8,
    199 as i32 as MV_U8,
    82 as i32 as MV_U8,
    246 as i32 as MV_U8,
    1 as i32 as MV_U8,
    0,
];
static S: [MV_U8; 256] = [
    99 as i32 as MV_U8,
    124 as i32 as MV_U8,
    119 as i32 as MV_U8,
    123 as i32 as MV_U8,
    242 as i32 as MV_U8,
    107 as i32 as MV_U8,
    111 as i32 as MV_U8,
    197 as i32 as MV_U8,
    48 as i32 as MV_U8,
    1 as i32 as MV_U8,
    103 as i32 as MV_U8,
    43 as i32 as MV_U8,
    254 as i32 as MV_U8,
    215 as i32 as MV_U8,
    171 as i32 as MV_U8,
    118 as i32 as MV_U8,
    202 as i32 as MV_U8,
    130 as i32 as MV_U8,
    201 as i32 as MV_U8,
    125 as i32 as MV_U8,
    250 as i32 as MV_U8,
    89 as i32 as MV_U8,
    71 as i32 as MV_U8,
    240 as i32 as MV_U8,
    173 as i32 as MV_U8,
    212 as i32 as MV_U8,
    162 as i32 as MV_U8,
    175 as i32 as MV_U8,
    156 as i32 as MV_U8,
    164 as i32 as MV_U8,
    114 as i32 as MV_U8,
    192 as i32 as MV_U8,
    183 as i32 as MV_U8,
    253 as i32 as MV_U8,
    147 as i32 as MV_U8,
    38 as i32 as MV_U8,
    54 as i32 as MV_U8,
    63 as i32 as MV_U8,
    247 as i32 as MV_U8,
    204 as i32 as MV_U8,
    52 as i32 as MV_U8,
    165 as i32 as MV_U8,
    229 as i32 as MV_U8,
    241 as i32 as MV_U8,
    113 as i32 as MV_U8,
    216 as i32 as MV_U8,
    49 as i32 as MV_U8,
    21 as i32 as MV_U8,
    4 as i32 as MV_U8,
    199 as i32 as MV_U8,
    35 as i32 as MV_U8,
    195 as i32 as MV_U8,
    24 as i32 as MV_U8,
    150 as i32 as MV_U8,
    5 as i32 as MV_U8,
    154 as i32 as MV_U8,
    7 as i32 as MV_U8,
    18 as i32 as MV_U8,
    128 as i32 as MV_U8,
    226 as i32 as MV_U8,
    235 as i32 as MV_U8,
    39 as i32 as MV_U8,
    178 as i32 as MV_U8,
    117 as i32 as MV_U8,
    9 as i32 as MV_U8,
    131 as i32 as MV_U8,
    44 as i32 as MV_U8,
    26 as i32 as MV_U8,
    27 as i32 as MV_U8,
    110 as i32 as MV_U8,
    90 as i32 as MV_U8,
    160 as i32 as MV_U8,
    82 as i32 as MV_U8,
    59 as i32 as MV_U8,
    214 as i32 as MV_U8,
    179 as i32 as MV_U8,
    41 as i32 as MV_U8,
    227 as i32 as MV_U8,
    47 as i32 as MV_U8,
    132 as i32 as MV_U8,
    83 as i32 as MV_U8,
    209 as i32 as MV_U8,
    0 as i32 as MV_U8,
    237 as i32 as MV_U8,
    32 as i32 as MV_U8,
    252 as i32 as MV_U8,
    177 as i32 as MV_U8,
    91 as i32 as MV_U8,
    106 as i32 as MV_U8,
    203 as i32 as MV_U8,
    190 as i32 as MV_U8,
    57 as i32 as MV_U8,
    74 as i32 as MV_U8,
    76 as i32 as MV_U8,
    88 as i32 as MV_U8,
    207 as i32 as MV_U8,
    208 as i32 as MV_U8,
    239 as i32 as MV_U8,
    170 as i32 as MV_U8,
    251 as i32 as MV_U8,
    67 as i32 as MV_U8,
    77 as i32 as MV_U8,
    51 as i32 as MV_U8,
    133 as i32 as MV_U8,
    69 as i32 as MV_U8,
    249 as i32 as MV_U8,
    2 as i32 as MV_U8,
    127 as i32 as MV_U8,
    80 as i32 as MV_U8,
    60 as i32 as MV_U8,
    159 as i32 as MV_U8,
    168 as i32 as MV_U8,
    81 as i32 as MV_U8,
    163 as i32 as MV_U8,
    64 as i32 as MV_U8,
    143 as i32 as MV_U8,
    146 as i32 as MV_U8,
    157 as i32 as MV_U8,
    56 as i32 as MV_U8,
    245 as i32 as MV_U8,
    188 as i32 as MV_U8,
    182 as i32 as MV_U8,
    218 as i32 as MV_U8,
    33 as i32 as MV_U8,
    16 as i32 as MV_U8,
    255 as i32 as MV_U8,
    243 as i32 as MV_U8,
    210 as i32 as MV_U8,
    205 as i32 as MV_U8,
    12 as i32 as MV_U8,
    19 as i32 as MV_U8,
    236 as i32 as MV_U8,
    95 as i32 as MV_U8,
    151 as i32 as MV_U8,
    68 as i32 as MV_U8,
    23 as i32 as MV_U8,
    196 as i32 as MV_U8,
    167 as i32 as MV_U8,
    126 as i32 as MV_U8,
    61 as i32 as MV_U8,
    100 as i32 as MV_U8,
    93 as i32 as MV_U8,
    25 as i32 as MV_U8,
    115 as i32 as MV_U8,
    96 as i32 as MV_U8,
    129 as i32 as MV_U8,
    79 as i32 as MV_U8,
    220 as i32 as MV_U8,
    34 as i32 as MV_U8,
    42 as i32 as MV_U8,
    144 as i32 as MV_U8,
    136 as i32 as MV_U8,
    70 as i32 as MV_U8,
    238 as i32 as MV_U8,
    184 as i32 as MV_U8,
    20 as i32 as MV_U8,
    222 as i32 as MV_U8,
    94 as i32 as MV_U8,
    11 as i32 as MV_U8,
    219 as i32 as MV_U8,
    224 as i32 as MV_U8,
    50 as i32 as MV_U8,
    58 as i32 as MV_U8,
    10 as i32 as MV_U8,
    73 as i32 as MV_U8,
    6 as i32 as MV_U8,
    36 as i32 as MV_U8,
    92 as i32 as MV_U8,
    194 as i32 as MV_U8,
    211 as i32 as MV_U8,
    172 as i32 as MV_U8,
    98 as i32 as MV_U8,
    145 as i32 as MV_U8,
    149 as i32 as MV_U8,
    228 as i32 as MV_U8,
    121 as i32 as MV_U8,
    231 as i32 as MV_U8,
    200 as i32 as MV_U8,
    55 as i32 as MV_U8,
    109 as i32 as MV_U8,
    141 as i32 as MV_U8,
    213 as i32 as MV_U8,
    78 as i32 as MV_U8,
    169 as i32 as MV_U8,
    108 as i32 as MV_U8,
    86 as i32 as MV_U8,
    244 as i32 as MV_U8,
    234 as i32 as MV_U8,
    101 as i32 as MV_U8,
    122 as i32 as MV_U8,
    174 as i32 as MV_U8,
    8 as i32 as MV_U8,
    186 as i32 as MV_U8,
    120 as i32 as MV_U8,
    37 as i32 as MV_U8,
    46 as i32 as MV_U8,
    28 as i32 as MV_U8,
    166 as i32 as MV_U8,
    180 as i32 as MV_U8,
    198 as i32 as MV_U8,
    232 as i32 as MV_U8,
    221 as i32 as MV_U8,
    116 as i32 as MV_U8,
    31 as i32 as MV_U8,
    75 as i32 as MV_U8,
    189 as i32 as MV_U8,
    139 as i32 as MV_U8,
    138 as i32 as MV_U8,
    112 as i32 as MV_U8,
    62 as i32 as MV_U8,
    181 as i32 as MV_U8,
    102 as i32 as MV_U8,
    72 as i32 as MV_U8,
    3 as i32 as MV_U8,
    246 as i32 as MV_U8,
    14 as i32 as MV_U8,
    97 as i32 as MV_U8,
    53 as i32 as MV_U8,
    87 as i32 as MV_U8,
    185 as i32 as MV_U8,
    134 as i32 as MV_U8,
    193 as i32 as MV_U8,
    29 as i32 as MV_U8,
    158 as i32 as MV_U8,
    225 as i32 as MV_U8,
    248 as i32 as MV_U8,
    152 as i32 as MV_U8,
    17 as i32 as MV_U8,
    105 as i32 as MV_U8,
    217 as i32 as MV_U8,
    142 as i32 as MV_U8,
    148 as i32 as MV_U8,
    155 as i32 as MV_U8,
    30 as i32 as MV_U8,
    135 as i32 as MV_U8,
    233 as i32 as MV_U8,
    206 as i32 as MV_U8,
    85 as i32 as MV_U8,
    40 as i32 as MV_U8,
    223 as i32 as MV_U8,
    140 as i32 as MV_U8,
    161 as i32 as MV_U8,
    137 as i32 as MV_U8,
    13 as i32 as MV_U8,
    191 as i32 as MV_U8,
    230 as i32 as MV_U8,
    66 as i32 as MV_U8,
    104 as i32 as MV_U8,
    65 as i32 as MV_U8,
    153 as i32 as MV_U8,
    45 as i32 as MV_U8,
    15 as i32 as MV_U8,
    176 as i32 as MV_U8,
    84 as i32 as MV_U8,
    187 as i32 as MV_U8,
    22 as i32 as MV_U8,
];
static Si: [MV_U8; 256] = [
    82 as i32 as MV_U8,
    9 as i32 as MV_U8,
    106 as i32 as MV_U8,
    213 as i32 as MV_U8,
    48 as i32 as MV_U8,
    54 as i32 as MV_U8,
    165 as i32 as MV_U8,
    56 as i32 as MV_U8,
    191 as i32 as MV_U8,
    64 as i32 as MV_U8,
    163 as i32 as MV_U8,
    158 as i32 as MV_U8,
    129 as i32 as MV_U8,
    243 as i32 as MV_U8,
    215 as i32 as MV_U8,
    251 as i32 as MV_U8,
    124 as i32 as MV_U8,
    227 as i32 as MV_U8,
    57 as i32 as MV_U8,
    130 as i32 as MV_U8,
    155 as i32 as MV_U8,
    47 as i32 as MV_U8,
    255 as i32 as MV_U8,
    135 as i32 as MV_U8,
    52 as i32 as MV_U8,
    142 as i32 as MV_U8,
    67 as i32 as MV_U8,
    68 as i32 as MV_U8,
    196 as i32 as MV_U8,
    222 as i32 as MV_U8,
    233 as i32 as MV_U8,
    203 as i32 as MV_U8,
    84 as i32 as MV_U8,
    123 as i32 as MV_U8,
    148 as i32 as MV_U8,
    50 as i32 as MV_U8,
    166 as i32 as MV_U8,
    194 as i32 as MV_U8,
    35 as i32 as MV_U8,
    61 as i32 as MV_U8,
    238 as i32 as MV_U8,
    76 as i32 as MV_U8,
    149 as i32 as MV_U8,
    11 as i32 as MV_U8,
    66 as i32 as MV_U8,
    250 as i32 as MV_U8,
    195 as i32 as MV_U8,
    78 as i32 as MV_U8,
    8 as i32 as MV_U8,
    46 as i32 as MV_U8,
    161 as i32 as MV_U8,
    102 as i32 as MV_U8,
    40 as i32 as MV_U8,
    217 as i32 as MV_U8,
    36 as i32 as MV_U8,
    178 as i32 as MV_U8,
    118 as i32 as MV_U8,
    91 as i32 as MV_U8,
    162 as i32 as MV_U8,
    73 as i32 as MV_U8,
    109 as i32 as MV_U8,
    139 as i32 as MV_U8,
    209 as i32 as MV_U8,
    37 as i32 as MV_U8,
    114 as i32 as MV_U8,
    248 as i32 as MV_U8,
    246 as i32 as MV_U8,
    100 as i32 as MV_U8,
    134 as i32 as MV_U8,
    104 as i32 as MV_U8,
    152 as i32 as MV_U8,
    22 as i32 as MV_U8,
    212 as i32 as MV_U8,
    164 as i32 as MV_U8,
    92 as i32 as MV_U8,
    204 as i32 as MV_U8,
    93 as i32 as MV_U8,
    101 as i32 as MV_U8,
    182 as i32 as MV_U8,
    146 as i32 as MV_U8,
    108 as i32 as MV_U8,
    112 as i32 as MV_U8,
    72 as i32 as MV_U8,
    80 as i32 as MV_U8,
    253 as i32 as MV_U8,
    237 as i32 as MV_U8,
    185 as i32 as MV_U8,
    218 as i32 as MV_U8,
    94 as i32 as MV_U8,
    21 as i32 as MV_U8,
    70 as i32 as MV_U8,
    87 as i32 as MV_U8,
    167 as i32 as MV_U8,
    141 as i32 as MV_U8,
    157 as i32 as MV_U8,
    132 as i32 as MV_U8,
    144 as i32 as MV_U8,
    216 as i32 as MV_U8,
    171 as i32 as MV_U8,
    0 as i32 as MV_U8,
    140 as i32 as MV_U8,
    188 as i32 as MV_U8,
    211 as i32 as MV_U8,
    10 as i32 as MV_U8,
    247 as i32 as MV_U8,
    228 as i32 as MV_U8,
    88 as i32 as MV_U8,
    5 as i32 as MV_U8,
    184 as i32 as MV_U8,
    179 as i32 as MV_U8,
    69 as i32 as MV_U8,
    6 as i32 as MV_U8,
    208 as i32 as MV_U8,
    44 as i32 as MV_U8,
    30 as i32 as MV_U8,
    143 as i32 as MV_U8,
    202 as i32 as MV_U8,
    63 as i32 as MV_U8,
    15 as i32 as MV_U8,
    2 as i32 as MV_U8,
    193 as i32 as MV_U8,
    175 as i32 as MV_U8,
    189 as i32 as MV_U8,
    3 as i32 as MV_U8,
    1 as i32 as MV_U8,
    19 as i32 as MV_U8,
    138 as i32 as MV_U8,
    107 as i32 as MV_U8,
    58 as i32 as MV_U8,
    145 as i32 as MV_U8,
    17 as i32 as MV_U8,
    65 as i32 as MV_U8,
    79 as i32 as MV_U8,
    103 as i32 as MV_U8,
    220 as i32 as MV_U8,
    234 as i32 as MV_U8,
    151 as i32 as MV_U8,
    242 as i32 as MV_U8,
    207 as i32 as MV_U8,
    206 as i32 as MV_U8,
    240 as i32 as MV_U8,
    180 as i32 as MV_U8,
    230 as i32 as MV_U8,
    115 as i32 as MV_U8,
    150 as i32 as MV_U8,
    172 as i32 as MV_U8,
    116 as i32 as MV_U8,
    34 as i32 as MV_U8,
    231 as i32 as MV_U8,
    173 as i32 as MV_U8,
    53 as i32 as MV_U8,
    133 as i32 as MV_U8,
    226 as i32 as MV_U8,
    249 as i32 as MV_U8,
    55 as i32 as MV_U8,
    232 as i32 as MV_U8,
    28 as i32 as MV_U8,
    117 as i32 as MV_U8,
    223 as i32 as MV_U8,
    110 as i32 as MV_U8,
    71 as i32 as MV_U8,
    241 as i32 as MV_U8,
    26 as i32 as MV_U8,
    113 as i32 as MV_U8,
    29 as i32 as MV_U8,
    41 as i32 as MV_U8,
    197 as i32 as MV_U8,
    137 as i32 as MV_U8,
    111 as i32 as MV_U8,
    183 as i32 as MV_U8,
    98 as i32 as MV_U8,
    14 as i32 as MV_U8,
    170 as i32 as MV_U8,
    24 as i32 as MV_U8,
    190 as i32 as MV_U8,
    27 as i32 as MV_U8,
    252 as i32 as MV_U8,
    86 as i32 as MV_U8,
    62 as i32 as MV_U8,
    75 as i32 as MV_U8,
    198 as i32 as MV_U8,
    210 as i32 as MV_U8,
    121 as i32 as MV_U8,
    32 as i32 as MV_U8,
    154 as i32 as MV_U8,
    219 as i32 as MV_U8,
    192 as i32 as MV_U8,
    254 as i32 as MV_U8,
    120 as i32 as MV_U8,
    205 as i32 as MV_U8,
    90 as i32 as MV_U8,
    244 as i32 as MV_U8,
    31 as i32 as MV_U8,
    221 as i32 as MV_U8,
    168 as i32 as MV_U8,
    51 as i32 as MV_U8,
    136 as i32 as MV_U8,
    7 as i32 as MV_U8,
    199 as i32 as MV_U8,
    49 as i32 as MV_U8,
    177 as i32 as MV_U8,
    18 as i32 as MV_U8,
    16 as i32 as MV_U8,
    89 as i32 as MV_U8,
    39 as i32 as MV_U8,
    128 as i32 as MV_U8,
    236 as i32 as MV_U8,
    95 as i32 as MV_U8,
    96 as i32 as MV_U8,
    81 as i32 as MV_U8,
    127 as i32 as MV_U8,
    169 as i32 as MV_U8,
    25 as i32 as MV_U8,
    181 as i32 as MV_U8,
    74 as i32 as MV_U8,
    13 as i32 as MV_U8,
    45 as i32 as MV_U8,
    229 as i32 as MV_U8,
    122 as i32 as MV_U8,
    159 as i32 as MV_U8,
    147 as i32 as MV_U8,
    201 as i32 as MV_U8,
    156 as i32 as MV_U8,
    239 as i32 as MV_U8,
    160 as i32 as MV_U8,
    224 as i32 as MV_U8,
    59 as i32 as MV_U8,
    77 as i32 as MV_U8,
    174 as i32 as MV_U8,
    42 as i32 as MV_U8,
    245 as i32 as MV_U8,
    176 as i32 as MV_U8,
    200 as i32 as MV_U8,
    235 as i32 as MV_U8,
    187 as i32 as MV_U8,
    60 as i32 as MV_U8,
    131 as i32 as MV_U8,
    83 as i32 as MV_U8,
    153 as i32 as MV_U8,
    97 as i32 as MV_U8,
    23 as i32 as MV_U8,
    43 as i32 as MV_U8,
    4 as i32 as MV_U8,
    126 as i32 as MV_U8,
    186 as i32 as MV_U8,
    119 as i32 as MV_U8,
    214 as i32 as MV_U8,
    38 as i32 as MV_U8,
    225 as i32 as MV_U8,
    105 as i32 as MV_U8,
    20 as i32 as MV_U8,
    99 as i32 as MV_U8,
    85 as i32 as MV_U8,
    33 as i32 as MV_U8,
    12 as i32 as MV_U8,
    125 as i32 as MV_U8,
];
static rcon: [MV_U32; 30] = [
    0x1 as i32 as MV_U32,
    0x2 as i32 as MV_U32,
    0x4 as i32 as MV_U32,
    0x8 as i32 as MV_U32,
    0x10 as i32 as MV_U32,
    0x20 as i32 as MV_U32,
    0x40 as i32 as MV_U32,
    0x80 as i32 as MV_U32,
    0x1b as i32 as MV_U32,
    0x36 as i32 as MV_U32,
    0x6c as i32 as MV_U32,
    0xd8 as i32 as MV_U32,
    0xab as i32 as MV_U32,
    0x4d as i32 as MV_U32,
    0x9a as i32 as MV_U32,
    0x2f as i32 as MV_U32,
    0x5e as i32 as MV_U32,
    0xbc as i32 as MV_U32,
    0x63 as i32 as MV_U32,
    0xc6 as i32 as MV_U32,
    0x97 as i32 as MV_U32,
    0x35 as i32 as MV_U32,
    0x6a as i32 as MV_U32,
    0xd4 as i32 as MV_U32,
    0xb3 as i32 as MV_U32,
    0x7d as i32 as MV_U32,
    0xfa as i32 as MV_U32,
    0xef as i32 as MV_U32,
    0xc5 as i32 as MV_U32,
    0x91 as i32 as MV_U32,
];
#[no_mangle]


#[inline]
unsafe fn mul1(mut aa: MV_U8, mut bb: MV_U8) -> MV_U8 {
    return (mask[bb as usize] as i32
        & Alogtable
            [(aa as i32 + Logtable[bb as usize] as i32) as usize]
            as i32) as MV_U8;
}
#[no_mangle]
unsafe fn KeyAddition(
    mut a: *mut [MV_U8; 4],
    mut rk: *mut [MV_U8; 4],
    mut BC: MV_U8,
) {
    let ref mut fresh1 = *((&raw mut *a.offset(0 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8
        as *mut MV_U32)
        .offset(0 as i32 as isize);
    *fresh1 ^= *((&raw mut *rk.offset(0 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) as u32;
    let ref mut fresh2 = *((&raw mut *a.offset(1 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8
        as *mut MV_U32)
        .offset(0 as i32 as isize);
    *fresh2 ^= *((&raw mut *rk.offset(1 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) as u32;
    let ref mut fresh3 = *((&raw mut *a.offset(2 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8
        as *mut MV_U32)
        .offset(0 as i32 as isize);
    *fresh3 ^= *((&raw mut *rk.offset(2 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) as u32;
    let ref mut fresh4 = *((&raw mut *a.offset(3 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8
        as *mut MV_U32)
        .offset(0 as i32 as isize);
    *fresh4 ^= *((&raw mut *rk.offset(3 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) as u32;
}
#[no_mangle]
unsafe fn ShiftRow128Enc(mut a: *mut [MV_U8; 4]) {
    let mut tmp: [MV_U8; 4] = [0; 4];
    tmp[0 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[1 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[2 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[3 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[0 as i32 as usize];
    *((&raw mut *a.offset(1 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
    tmp[0 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[2 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[3 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[0 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[1 as i32 as usize];
    *((&raw mut *a.offset(2 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
    tmp[0 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[3 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[0 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[1 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[2 as i32 as usize];
    *((&raw mut *a.offset(3 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
}
#[no_mangle]
unsafe fn ShiftRow128Dec(mut a: *mut [MV_U8; 4]) {
    let mut tmp: [MV_U8; 4] = [0; 4];
    tmp[0 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[3 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[0 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[1 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(1 as i32 as isize))[2 as i32 as usize];
    *((&raw mut *a.offset(1 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
    tmp[0 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[2 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[3 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[0 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(2 as i32 as isize))[1 as i32 as usize];
    *((&raw mut *a.offset(2 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
    tmp[0 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[1 as i32 as usize];
    tmp[1 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[2 as i32 as usize];
    tmp[2 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[3 as i32 as usize];
    tmp[3 as i32 as usize] =
        (*a.offset(3 as i32 as isize))[0 as i32 as usize];
    *((&raw mut *a.offset(3 as i32 as isize) as *mut MV_U8)
        .offset(0 as i32 as isize) as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize) = *((&raw mut tmp as *mut MV_U8)
        .offset(0 as i32 as isize)
        as *mut MV_U8 as *mut MV_U32)
        .offset(0 as i32 as isize);
}
#[no_mangle]
unsafe fn Substitution(mut a: *mut [MV_U8; 4], mut box_0: *mut MV_U8) {
    let mut i: i32;
    let mut j: i32;
    i = 0 as i32;
    while i < 4 as i32 {
        j = 0 as i32;
        while j < 4 as i32 {
            (*a.offset(i as isize))[j as usize] =
                *box_0.offset((*a.offset(i as isize))[j as usize] as isize);
            j += 1;
        }
        i += 1;
    }
}
#[no_mangle]
unsafe fn MixColumn(mut a: *mut [MV_U8; 4], mut rk: *mut [MV_U8; 4]) {
    let mut b: [[MV_U8; 4]; 4] = [[0; 4]; 4];
    let mut i: i32;
    let mut j: i32;
    j = 0 as i32;
    while j < 4 as i32 {
        b[0 as i32 as usize][j as usize] = (mask
            [(*a.offset(0 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(25 as i32
                + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(1 as i32
                    + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ (*a.offset(2 as i32 as isize))[j as usize] as i32
            ^ (*a.offset(3 as i32 as isize))[j as usize] as i32)
            as MV_U8;
        b[1 as i32 as usize][j as usize] = (mask
            [(*a.offset(1 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(25 as i32
                + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(1 as i32
                    + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ (*a.offset(3 as i32 as isize))[j as usize] as i32
            ^ (*a.offset(0 as i32 as isize))[j as usize] as i32)
            as MV_U8;
        b[2 as i32 as usize][j as usize] = (mask
            [(*a.offset(2 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(25 as i32
                + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(1 as i32
                    + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ (*a.offset(0 as i32 as isize))[j as usize] as i32
            ^ (*a.offset(1 as i32 as isize))[j as usize] as i32)
            as MV_U8;
        b[3 as i32 as usize][j as usize] = (mask
            [(*a.offset(3 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(25 as i32
                + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(1 as i32
                    + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ (*a.offset(1 as i32 as isize))[j as usize] as i32
            ^ (*a.offset(2 as i32 as isize))[j as usize] as i32)
            as MV_U8;
        j += 1;
    }
    i = 0 as i32;
    while i < 4 as i32 {
        *((&raw mut *a.offset(i as isize) as *mut MV_U8).offset(0 as i32 as isize)
            as *mut MV_U8 as *mut MV_U32)
            .offset(0 as i32 as isize) =
            *((&raw mut *(&raw mut b as *mut [MV_U8; 4]).offset(i as isize) as *mut MV_U8)
                .offset(0 as i32 as isize) as *mut MV_U8
                as *mut MV_U32)
                .offset(0 as i32 as isize)
                ^ *((&raw mut *rk.offset(i as isize) as *mut MV_U8)
                    .offset(0 as i32 as isize) as *mut MV_U8
                    as *mut MV_U32)
                    .offset(0 as i32 as isize);
        i += 1;
    }
}
#[no_mangle]
unsafe fn InvMixColumn(mut a: *mut [MV_U8; 4]) {
    let mut b: [[MV_U8; 4]; 4] = [[0; 4]; 4];
    let mut i: i32;
    let mut j: i32;
    j = 0 as i32;
    while j < 4 as i32 {
        b[0 as i32 as usize][j as usize] = (mask
            [(*a.offset(0 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(223 as i32
                + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(104 as i32
                    + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(238 as i32
                    + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(199 as i32
                    + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32)
            as MV_U8;
        b[1 as i32 as usize][j as usize] = (mask
            [(*a.offset(1 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(223 as i32
                + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(104 as i32
                    + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(238 as i32
                    + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(199 as i32
                    + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32)
            as MV_U8;
        b[2 as i32 as usize][j as usize] = (mask
            [(*a.offset(2 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(223 as i32
                + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(104 as i32
                    + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(238 as i32
                    + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(199 as i32
                    + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32)
            as MV_U8;
        b[3 as i32 as usize][j as usize] = (mask
            [(*a.offset(3 as i32 as isize))[j as usize] as usize]
            as i32
            & Alogtable[(223 as i32
                + Logtable[(*a.offset(3 as i32 as isize))[j as usize] as usize]
                    as i32) as usize] as i32
            ^ mask[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(104 as i32
                    + Logtable[(*a.offset(0 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(238 as i32
                    + Logtable[(*a.offset(1 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32
            ^ mask[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                as i32
                & Alogtable[(199 as i32
                    + Logtable[(*a.offset(2 as i32 as isize))[j as usize] as usize]
                        as i32) as usize] as i32)
            as MV_U8;
        j += 1;
    }
    i = 0 as i32;
    while i < 4 as i32 {
        *((&raw mut *a.offset(i as isize) as *mut MV_U8).offset(0 as i32 as isize)
            as *mut MV_U8 as *mut MV_U32)
            .offset(0 as i32 as isize) =
            *((&raw mut *(&raw mut b as *mut [MV_U8; 4]).offset(i as isize) as *mut MV_U8)
                .offset(0 as i32 as isize) as *mut MV_U8
                as *mut MV_U32)
                .offset(0 as i32 as isize);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn rijndaelKeySched(
    mut k: *mut [MV_U8; 8],
    mut keyBits: core::ffi::c_int,
    mut blockBits: core::ffi::c_int,
    mut W: *mut [[MV_U8; 4]; 4],
) -> core::ffi::c_int {
    let mut KC: core::ffi::c_int;
    let mut BC: core::ffi::c_int;
    let mut ROUNDS: core::ffi::c_int;
    let mut i: core::ffi::c_int;
    let mut j: core::ffi::c_int;
    let mut t: core::ffi::c_int;
    let mut rconpointer: core::ffi::c_int = 0 as core::ffi::c_int;
    let mut tk: [[MV_U8; 8]; 4] = [[0; 8]; 4];
    match keyBits {
        128 => {
            KC = 4 as core::ffi::c_int;
        }
        192 => {
            KC = 6 as core::ffi::c_int;
        }
        256 => {
            KC = 8 as core::ffi::c_int;
        }
        _ => return -(1 as core::ffi::c_int),
    }
    match blockBits {
        128 => {
            BC = 4 as core::ffi::c_int;
        }
        192 => {
            BC = 6 as core::ffi::c_int;
        }
        256 => {
            BC = 8 as core::ffi::c_int;
        }
        _ => return -(2 as core::ffi::c_int),
    }
    match if keyBits >= blockBits {
        keyBits
    } else {
        blockBits
    } {
        128 => {
            ROUNDS = 10 as core::ffi::c_int;
        }
        192 => {
            ROUNDS = 12 as core::ffi::c_int;
        }
        256 => {
            ROUNDS = 14 as core::ffi::c_int;
        }
        _ => return -(3 as core::ffi::c_int),
    }
    j = 0 as core::ffi::c_int;
    while j < KC {
        i = 0 as core::ffi::c_int;
        while i < 4 as core::ffi::c_int {
            tk[i as usize][j as usize] = (*k.offset(i as isize))[j as usize];
            i += 1;
        }
        j += 1;
    }
    t = 0 as core::ffi::c_int;
    j = 0 as core::ffi::c_int;
    while j < KC && t < (ROUNDS + 1 as core::ffi::c_int) * BC {
        i = 0 as core::ffi::c_int;
        while i < 4 as core::ffi::c_int {
            (*W.offset((t / BC) as isize))[i as usize][(t % BC) as usize] =
                tk[i as usize][j as usize];
            i += 1;
        }
        j += 1;
        t += 1;
    }
    while t < (ROUNDS + 1 as core::ffi::c_int) * BC {
        i = 0 as core::ffi::c_int;
        while i < 4 as core::ffi::c_int {
            tk[i as usize][0 as core::ffi::c_int as usize] =
                (tk[i as usize][0 as core::ffi::c_int as usize] as core::ffi::c_int
                    ^ S[tk[((i + 1 as core::ffi::c_int) % 4 as core::ffi::c_int) as usize]
                        [(KC - 1 as core::ffi::c_int) as usize] as usize]
                        as core::ffi::c_int) as MV_U8;
            i += 1;
        }
        let fresh0 = rconpointer;
        rconpointer = rconpointer + 1;
        tk[0 as core::ffi::c_int as usize][0 as core::ffi::c_int as usize] =
            (tk[0 as core::ffi::c_int as usize][0 as core::ffi::c_int as usize]
                as core::ffi::c_ulong
                ^ rcon[fresh0 as usize] as core::ffi::c_ulong) as MV_U8;
        if KC != 8 as core::ffi::c_int {
            j = 1 as core::ffi::c_int;
            while j < KC {
                i = 0 as core::ffi::c_int;
                while i < 4 as core::ffi::c_int {
                    tk[i as usize][j as usize] = (tk[i as usize][j as usize] as core::ffi::c_int
                        ^ tk[i as usize][(j - 1 as core::ffi::c_int) as usize]
                            as core::ffi::c_int)
                        as MV_U8;
                    i += 1;
                }
                j += 1;
            }
        } else {
            j = 1 as core::ffi::c_int;
            while j < KC / 2 as core::ffi::c_int {
                i = 0 as core::ffi::c_int;
                while i < 4 as core::ffi::c_int {
                    tk[i as usize][j as usize] = (tk[i as usize][j as usize] as core::ffi::c_int
                        ^ tk[i as usize][(j - 1 as core::ffi::c_int) as usize]
                            as core::ffi::c_int)
                        as MV_U8;
                    i += 1;
                }
                j += 1;
            }
            i = 0 as core::ffi::c_int;
            while i < 4 as core::ffi::c_int {
                tk[i as usize][(KC / 2 as core::ffi::c_int) as usize] =
                    (tk[i as usize][(KC / 2 as core::ffi::c_int) as usize] as core::ffi::c_int
                        ^ S[tk[i as usize]
                            [(KC / 2 as core::ffi::c_int - 1 as core::ffi::c_int) as usize]
                            as usize] as core::ffi::c_int) as MV_U8;
                i += 1;
            }
            j = KC / 2 as core::ffi::c_int + 1 as core::ffi::c_int;
            while j < KC {
                i = 0 as core::ffi::c_int;
                while i < 4 as core::ffi::c_int {
                    tk[i as usize][j as usize] = (tk[i as usize][j as usize] as core::ffi::c_int
                        ^ tk[i as usize][(j - 1 as core::ffi::c_int) as usize]
                            as core::ffi::c_int)
                        as MV_U8;
                    i += 1;
                }
                j += 1;
            }
        }
        j = 0 as core::ffi::c_int;
        while j < KC && t < (ROUNDS + 1 as core::ffi::c_int) * BC {
            i = 0 as core::ffi::c_int;
            while i < 4 as core::ffi::c_int {
                (*W.offset((t / BC) as isize))[i as usize][(t % BC) as usize] =
                    tk[i as usize][j as usize];
                i += 1;
            }
            j += 1;
            t += 1;
        }
    }
    return 0 as core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn rijndaelEncrypt128(
    mut a: *mut [MV_U8; 4],
    mut rk: *mut [[MV_U8; 4]; 4],
    mut rounds: i32,
) -> i32 {
    let mut r: i32;
    let mut BC: i32;
    let mut ROUNDS: i32;
    BC = 4i32;
    ROUNDS = rounds;
    KeyAddition(a, &raw mut *rk.offset(0 as isize) as *mut [MV_U8; 4], BC as MV_U8);
    r = 1i32;
    while r < ROUNDS {
        Substitution(a, S.as_ptr() as *mut MV_U8);
        ShiftRow128Enc(a);
        MixColumn(a, &raw mut *rk.offset(r as isize) as *mut [MV_U8; 4]);
        r += 1;
    }
    Substitution(a, S.as_ptr() as *mut MV_U8);
    ShiftRow128Enc(a);
    KeyAddition(
        a,
        &raw mut *rk.offset(ROUNDS as isize) as *mut [MV_U8; 4],
        BC as MV_U8,
    );
    return 0i32;
}
#[no_mangle]
pub unsafe extern "C" fn rijndaelDecrypt128(
    a: *mut [u8; 4],
    rk: *mut [[u8; 4]; 4],
    rounds: i32,
) -> i32 {
    let mut r: i32;
    let mut BC: i32;
    let mut ROUNDS: i32;

    BC = 4;
    ROUNDS = rounds;

    KeyAddition(
        a,
        &raw mut *rk.offset(ROUNDS as isize) as *mut [u8; 4],
        BC as u8,
    );
    ShiftRow128Dec(a);
    Substitution(a, Si.as_ptr() as *mut u8);

    r = ROUNDS - 1;
    while r > 0 {
        KeyAddition(
            a,
            &raw mut *rk.offset(r as isize) as *mut [u8; 4],
            BC as u8,
        );
        InvMixColumn(a);
        ShiftRow128Dec(a);
        Substitution(a, Si.as_ptr() as *mut u8);
        r -= 1;
    }

    KeyAddition(
        a,
        &raw mut *rk.offset(0) as *mut [u8; 4],
        BC as u8,
    );

    return 0;
}