use crate::apphelp::hsdb::SdbTagIDToTagRef;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_04152469CE43: [u16; 9] = [0x0044u16, 0x0041, 0x0054, 0x0041, 0x0042, 0x0041, 0x0053, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_04B8EEB408B4: [u16; 8] = [0x0043u16, 0x004F, 0x004E, 0x0054, 0x0045, 0x0058, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_0872ACAF2F2A: [u16; 12] = [
    0x0044u16, 0x0041, 0x0054, 0x0041, 0x0042, 0x0041, 0x0053, 0x0045, 0x005F, 0x0049,
    0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_08BC4D02D8CD: [u16; 6] = [0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_0B06054A863C: [u16; 12] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0050, 0x0041, 0x0043, 0x004B, 0x0041, 0x0047,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_0C546AC9EAC2: [u16; 15] = [
    0x0050u16, 0x0052, 0x0045, 0x0056, 0x004F, 0x0053, 0x004D, 0x0041, 0x004A, 0x004F,
    0x0052, 0x0056, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_12D32AAF4A91: [u16; 17] = [
    0x0043u16, 0x004F, 0x004D, 0x0050, 0x0049, 0x004C, 0x0045, 0x0052, 0x005F, 0x0056,
    0x0045, 0x0052, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_1314F3F7733B: [u16; 9] = [0x0043u16, 0x0048, 0x0045, 0x0043, 0x004B, 0x0053, 0x0055, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_16160A13BD6B: [u16; 9] = [0x0041u16, 0x0050, 0x0050, 0x005F, 0x004E, 0x0041, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_17FFB8D9A3E9: [u16; 7] = [0x0045u16, 0x0058, 0x0045, 0x005F, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_18669BB540DB: [u16; 13] = [
    0x0056u16, 0x0045, 0x0052, 0x005F, 0x004C, 0x0041, 0x004E, 0x0047, 0x0055, 0x0041,
    0x0047, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_1D44D5A7D22B: [u16; 18] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x0050, 0x0052, 0x004F, 0x0043, 0x0045,
    0x0053, 0x0053, 0x0050, 0x0041, 0x0052, 0x0041, 0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_1FAD449F8979: [u16; 5] = [0x0044u16, 0x0041, 0x0054, 0x0041, 0x0000];
pub static RUSTCONV_WSTR_227C6B982831: [u16; 10] = [0x004Cu16, 0x0049, 0x004E, 0x004B, 0x005F, 0x0054, 0x0045, 0x0058, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_2333D85AE41A: [u16; 16] = [
    0x004Fu16, 0x0053, 0x005F, 0x0053, 0x0045, 0x0052, 0x0056, 0x0049, 0x0043, 0x0045,
    0x005F, 0x0050, 0x0041, 0x0043, 0x004B, 0x0000,
];
pub static RUSTCONV_WSTR_239E714E0983: [u16; 5] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_23C16E939718: [u16; 16] = [
    0x0041u16, 0x0050, 0x0050, 0x004C, 0x0059, 0x005F, 0x0041, 0x004C, 0x004C, 0x005F,
    0x0053, 0x0048, 0x0049, 0x004D, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_242AEF5C2E9D: [u16; 10] = [0x0056u16, 0x0045, 0x0052, 0x0046, 0x0049, 0x004C, 0x0045, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_2436CBC0DE61: [u16; 14] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0054, 0x0052, 0x0041, 0x004E, 0x0053, 0x0046,
    0x004F, 0x0052, 0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_25C0EB9DBCCB: [u16; 6] = [0x0046u16, 0x004C, 0x0041, 0x0047, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_273396E20ECA: [u16; 11] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x0054, 0x0041, 0x0047, 0x0049, 0x0044,
    0x0000,
];
pub static RUSTCONV_WSTR_27EE4809B6BF: [u16; 11] = [
    0x0048u16, 0x0054, 0x004D, 0x004C, 0x0048, 0x0045, 0x004C, 0x0050, 0x0049, 0x0044,
    0x0000,
];
pub static RUSTCONV_WSTR_289950D9E528: [u16; 17] = [
    0x0052u16, 0x0055, 0x004E, 0x0054, 0x0049, 0x004D, 0x0045, 0x005F, 0x0050, 0x004C,
    0x0041, 0x0054, 0x0046, 0x004F, 0x0052, 0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_2B241FEDD36E: [u16; 22] = [
    0x004Cu16, 0x0041, 0x0059, 0x0045, 0x0052, 0x005F, 0x0050, 0x0052, 0x004F, 0x0050,
    0x0041, 0x0047, 0x0041, 0x0054, 0x0049, 0x004F, 0x004E, 0x005F, 0x004F, 0x0046,
    0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_2C84819711BE: [u16; 14] = [
    0x0049u16, 0x004E, 0x0054, 0x0045, 0x0052, 0x004E, 0x0041, 0x004C, 0x005F, 0x004E,
    0x0041, 0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_2D1087318FA4: [u16; 11] = [
    0x0044u16, 0x0041, 0x0054, 0x0041, 0x005F, 0x0051, 0x0057, 0x004F, 0x0052, 0x0044,
    0x0000,
];
pub static RUSTCONV_WSTR_2D28F26CDB5C: [u16; 18] = [
    0x0043u16, 0x004F, 0x004E, 0x0054, 0x0045, 0x0058, 0x0054, 0x005F, 0x0042, 0x0052,
    0x0041, 0x004E, 0x0043, 0x0048, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_328694CCA74D: [u16; 9] = [0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x004C, 0x0055, 0x0041, 0x0000];
pub static RUSTCONV_WSTR_36C918ADF93D: [u16; 18] = [
    0x0031u16, 0x0036, 0x0042, 0x0049, 0x0054, 0x005F, 0x0044, 0x0045, 0x0053, 0x0043,
    0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_373CD992ADB9: [u16; 11] = [
    0x004Du16, 0x0041, 0x0054, 0x0043, 0x0048, 0x005F, 0x004D, 0x004F, 0x0044, 0x0045,
    0x0000,
];
pub static RUSTCONV_WSTR_3801BF26E057: [u16; 14] = [
    0x0057u16, 0x0049, 0x004C, 0x0044, 0x0043, 0x0041, 0x0052, 0x0044, 0x005F, 0x004E,
    0x0041, 0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_38283C14387D: [u16; 15] = [
    0x0041u16, 0x0050, 0x0050, 0x005F, 0x004E, 0x0041, 0x004D, 0x0045, 0x005F, 0x0052,
    0x0043, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_38334143D9FA: [u16; 6] = [0x0054u16, 0x0041, 0x0047, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_3AEE8AFFC4B0: [u16; 19] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0054, 0x0052, 0x0041, 0x004E, 0x0053, 0x0046,
    0x004F, 0x0052, 0x004D, 0x005F, 0x0046, 0x0049, 0x004C, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_3BD4CDDA752C: [u16; 4] = [0x0053u16, 0x0050, 0x0043, 0x0000];
pub static RUSTCONV_WSTR_3D9F7780DD3F: [u16; 18] = [
    0x0031u16, 0x0036, 0x0042, 0x0049, 0x0054, 0x005F, 0x004D, 0x004F, 0x0044, 0x0055,
    0x004C, 0x0045, 0x005F, 0x004E, 0x0041, 0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_3E694BB9A2D4: [u16; 16] = [
    0x0050u16, 0x0052, 0x004F, 0x0044, 0x0055, 0x0043, 0x0054, 0x005F, 0x0056, 0x0045,
    0x0052, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_3EF99EF5AAFA: [u16; 12] = [
    0x004Du16, 0x004F, 0x0044, 0x0055, 0x004C, 0x0045, 0x005F, 0x0054, 0x0059, 0x0050,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_3F17A6C8DD89: [u16; 7] = [0x004Cu16, 0x0041, 0x004E, 0x0047, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_3FAE7E59DC04: [u16; 17] = [
    0x0050u16, 0x0052, 0x0045, 0x0056, 0x004F, 0x0053, 0x0050, 0x004C, 0x0041, 0x0054,
    0x0046, 0x004F, 0x0052, 0x004D, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_421D6A43470F: [u16; 11] = [
    0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x005F, 0x0042, 0x0049, 0x0054, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_42A1743682CA: [u16; 13] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x0053, 0x005F, 0x004E, 0x0054, 0x0056, 0x0044,
    0x004D, 0x0033, 0x0000,
];
pub static RUSTCONV_WSTR_454A87B5217E: [u16; 4] = [0x0054u16, 0x0041, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_47395966419C: [u16; 12] = [
    0x0045u16, 0x0058, 0x0045, 0x005F, 0x0057, 0x0052, 0x0041, 0x0050, 0x0050, 0x0045,
    0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_4AED236F221B: [u16; 11] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0054, 0x0041, 0x0047, 0x0049, 0x0044,
    0x0000,
];
pub static RUSTCONV_WSTR_4E0F1B69A26A: [u16; 10] = [0x004Cu16, 0x0049, 0x004E, 0x004B, 0x005F, 0x0044, 0x0041, 0x0054, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_4E11801F47A5: [u16; 14] = [
    0x0052u16, 0x0041, 0x0043, 0x005F, 0x0045, 0x0056, 0x0045, 0x004E, 0x0054, 0x005F,
    0x004F, 0x0046, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_4E6156A8C01F: [u16; 7] = [0x0041u16, 0x0043, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_4F3A9B3AE4CB: [u16; 25] = [
    0x0055u16, 0x0050, 0x0054, 0x004F, 0x005F, 0x0042, 0x0049, 0x004E, 0x005F, 0x0050,
    0x0052, 0x004F, 0x0044, 0x0055, 0x0043, 0x0054, 0x005F, 0x0056, 0x0045, 0x0052,
    0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_5017923A686F: [u16; 20] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0054, 0x0052, 0x0041, 0x004E, 0x0053, 0x0046,
    0x004F, 0x0052, 0x004D, 0x005F, 0x0054, 0x0041, 0x0047, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_51845E867197: [u16; 18] = [
    0x004Cu16, 0x0041, 0x0059, 0x0045, 0x0052, 0x005F, 0x0044, 0x0049, 0x0053, 0x0050,
    0x004C, 0x0041, 0x0059, 0x004E, 0x0041, 0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_51959BA57352: [u16; 18] = [
    0x0044u16, 0x0045, 0x0053, 0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F,
    0x004E, 0x005F, 0x0052, 0x0043, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_52914B58A62F: [u16; 10] = [0x0050u16, 0x0041, 0x0054, 0x0043, 0x0048, 0x005F, 0x0052, 0x0045, 0x0046, 0x0000];
pub static RUSTCONV_WSTR_57FECFF414E5: [u16; 17] = [
    0x0042u16, 0x0049, 0x004E, 0x005F, 0x0046, 0x0049, 0x004C, 0x0045, 0x005F, 0x0056,
    0x0045, 0x0052, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_5898F073B17E: [u16; 16] = [
    0x0041u16, 0x0050, 0x0050, 0x0048, 0x0045, 0x004C, 0x0050, 0x005F, 0x0043, 0x004F,
    0x004E, 0x0054, 0x0041, 0x0043, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_59971EDD22BE: [u16; 12] = [
    0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x005F, 0x0046, 0x004C, 0x0041, 0x0047,
    0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_5A1067373017: [u16; 16] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0045, 0x004E, 0x0047, 0x0049, 0x004E,
    0x0045, 0x005F, 0x004F, 0x0046, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_5B0F7F262758: [u16; 5] = [0x0053u16, 0x0049, 0x005A, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_5E9D9DEA8D61: [u16; 6] = [0x004Cu16, 0x0041, 0x0059, 0x0045, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_60BA521D1CA1: [u16; 13] = [
    0x0043u16, 0x004F, 0x004D, 0x0050, 0x0041, 0x004E, 0x0059, 0x005F, 0x004E, 0x0041,
    0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_629A0B15E864: [u16; 12] = [
    0x0044u16, 0x0045, 0x0053, 0x0043, 0x0052, 0x0049, 0x0050, 0x0054, 0x0049, 0x004F,
    0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_65C94AC49DBF: [u16; 18] = [
    0x004Fu16, 0x0052, 0x0049, 0x0047, 0x0049, 0x004E, 0x0041, 0x004C, 0x005F, 0x0046,
    0x0049, 0x004C, 0x0045, 0x004E, 0x0041, 0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_66888A2B7130: [u16; 15] = [
    0x0055u16, 0x0050, 0x0054, 0x004F, 0x005F, 0x004C, 0x0049, 0x004E, 0x004B, 0x005F,
    0x0044, 0x0041, 0x0054, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_67DE5C266FFF: [u16; 18] = [
    0x0053u16, 0x0055, 0x004D, 0x004D, 0x0041, 0x0052, 0x0059, 0x005F, 0x004D, 0x0053,
    0x0047, 0x005F, 0x0052, 0x0043, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_697A667BA010: [u16; 8] = [0x0049u16, 0x004E, 0x0043, 0x004C, 0x0055, 0x0044, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_69A11BC4DD07: [u16; 10] = [0x0044u16, 0x0041, 0x0054, 0x0041, 0x005F, 0x0042, 0x0049, 0x0054, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_6BE264DC8A4F: [u16; 12] = [
    0x0056u16, 0x0045, 0x0052, 0x0046, 0x0049, 0x004C, 0x0045, 0x0054, 0x0059, 0x0050,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_6C65909185A4: [u16; 16] = [
    0x0041u16, 0x0050, 0x0050, 0x0048, 0x0045, 0x004C, 0x0050, 0x005F, 0x0044, 0x0045,
    0x0054, 0x0041, 0x0049, 0x004C, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_6D2E68F65B35: [u16; 13] = [
    0x0043u16, 0x004F, 0x004D, 0x004D, 0x0041, 0x004E, 0x0044, 0x005F, 0x004C, 0x0049,
    0x004E, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_721C10FFCE5B: [u16; 12] = [
    0x0050u16, 0x0045, 0x005F, 0x0043, 0x0048, 0x0045, 0x0043, 0x004B, 0x0053, 0x0055,
    0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_75D8E40A76D2: [u16; 12] = [
    0x004Fu16, 0x0053, 0x005F, 0x0050, 0x004C, 0x0041, 0x0054, 0x0046, 0x004F, 0x0052,
    0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_77C3E0CF2A18: [u16; 18] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0043, 0x0055, 0x0053, 0x0054, 0x004F, 0x004D,
    0x005F, 0x0041, 0x0043, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_7A16028390BB: [u16; 12] = [
    0x0053u16, 0x0054, 0x0052, 0x0049, 0x004E, 0x0047, 0x0054, 0x0041, 0x0042, 0x004C,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_7A76B31D3E31: [u16; 10] = [0x0056u16, 0x0045, 0x0052, 0x0044, 0x0041, 0x0054, 0x0045, 0x004C, 0x004F, 0x0000];
pub static RUSTCONV_WSTR_7AC1B2A1871C: [u16; 11] = [
    0x0044u16, 0x0041, 0x0054, 0x0041, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044,
    0x0000,
];
pub static RUSTCONV_WSTR_7B1C0D1397C8: [u16; 9] = [0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x0052, 0x0045, 0x0046, 0x0000];
pub static RUSTCONV_WSTR_7B2767ABD848: [u16; 18] = [
    0x0056u16, 0x0045, 0x004E, 0x0044, 0x004F, 0x0052, 0x005F, 0x004E, 0x0041, 0x004D,
    0x0045, 0x005F, 0x0052, 0x0043, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_7DE7F613EDA8: [u16; 17] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x004D, 0x0041, 0x0053, 0x004B, 0x005F,
    0x004B, 0x0045, 0x0052, 0x004E, 0x0045, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_7E591A73C957: [u16; 10] = [0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x005F, 0x004B, 0x0045, 0x0059, 0x0000];
pub static RUSTCONV_WSTR_815CBF3E87D0: [u16; 13] = [
    0x0046u16, 0x0049, 0x004C, 0x0045, 0x005F, 0x0056, 0x0045, 0x0052, 0x0053, 0x0049,
    0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_82D1C5E5AAA8: [u16; 12] = [
    0x0044u16, 0x0041, 0x0054, 0x0041, 0x005F, 0x0053, 0x0054, 0x0052, 0x0049, 0x004E,
    0x0047, 0x0000,
];
pub static RUSTCONV_WSTR_85289DCA13CE: [u16; 5] = [0x0054u16, 0x0049, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_873D19CF8F2F: [u16; 23] = [
    0x0055u16, 0x0053, 0x0045, 0x005F, 0x0053, 0x0045, 0x0052, 0x0056, 0x0049, 0x0043,
    0x0045, 0x005F, 0x0050, 0x0041, 0x0043, 0x004B, 0x005F, 0x0046, 0x0049, 0x004C,
    0x0045, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_879DAA402051: [u16; 6] = [0x0050u16, 0x0041, 0x0054, 0x0043, 0x0048, 0x0000];
pub static RUSTCONV_WSTR_89F718515BCC: [u16; 10] = [0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x005F, 0x0054, 0x0041, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_8C7CF1E9BC62: [u16; 4] = [0x0045u16, 0x0058, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_8E2BC5ABC2F3: [u16; 4] = [0x0041u16, 0x0050, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_900F069F555C: [u16; 12] = [
    0x0041u16, 0x0043, 0x0054, 0x0049, 0x004F, 0x004E, 0x005F, 0x0054, 0x0059, 0x0050,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_90B53DCCB63A: [u16; 10] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x005F, 0x0042, 0x0049, 0x0054, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_91A200970320: [u16; 16] = [
    0x004Cu16, 0x0045, 0x0047, 0x0041, 0x004C, 0x005F, 0x0043, 0x004F, 0x0050, 0x0059,
    0x0052, 0x0049, 0x0047, 0x0048, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_93C8910194A5: [u16; 5] = [0x004Eu16, 0x0041, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_93FF1A47AC17: [u16; 17] = [
    0x0046u16, 0x0049, 0x004C, 0x0045, 0x005F, 0x0044, 0x0045, 0x0053, 0x0043, 0x0052,
    0x0049, 0x0050, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_96504F537330: [u16; 11] = [
    0x0050u16, 0x0041, 0x0054, 0x0043, 0x0048, 0x005F, 0x0042, 0x0049, 0x0054, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_97B9270952CE: [u16; 8] = [0x004Du16, 0x004F, 0x0044, 0x0054, 0x0049, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_986B64017E94: [u16; 18] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0054, 0x0052, 0x0041, 0x004E, 0x0053, 0x0046,
    0x004F, 0x0052, 0x004D, 0x005F, 0x0052, 0x0045, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_9AA69845B279: [u16; 7] = [0x004Cu16, 0x004F, 0x004F, 0x004B, 0x0055, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_9D8DE29A0BF8: [u16; 14] = [
    0x004Du16, 0x0049, 0x0054, 0x0049, 0x0047, 0x0041, 0x0054, 0x0049, 0x004F, 0x004E,
    0x005F, 0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_9E3130242CB2: [u16; 8] = [0x0047u16, 0x0045, 0x004E, 0x0045, 0x0052, 0x0041, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_9FDE1970188E: [u16; 10] = [0x0056u16, 0x0045, 0x0052, 0x0044, 0x0041, 0x0054, 0x0045, 0x0048, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_A69F9601B2C4: [u16; 11] = [
    0x0049u16, 0x006E, 0x0076, 0x0061, 0x006C, 0x0069, 0x0064, 0x0054, 0x0061, 0x0067,
    0x0000,
];
pub static RUSTCONV_WSTR_A79AA56FBE1B: [u16; 9] = [0x004Cu16, 0x0049, 0x004E, 0x004B, 0x005F, 0x0055, 0x0052, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_A8AB8C945814: [u16; 15] = [
    0x0044u16, 0x0041, 0x0054, 0x0041, 0x005F, 0x0056, 0x0041, 0x004C, 0x0055, 0x0045,
    0x0054, 0x0059, 0x0050, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_AB04297BDC11: [u16; 4] = [0x0055u16, 0x0052, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_ACBCC65B9C1D: [u16; 14] = [
    0x0050u16, 0x0052, 0x0045, 0x0056, 0x004F, 0x0053, 0x0042, 0x0055, 0x0049, 0x004C,
    0x0044, 0x004E, 0x004F, 0x0000,
];
pub static RUSTCONV_WSTR_AD7B0A117883: [u16; 13] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x0053, 0x005F, 0x004E, 0x0054, 0x0056, 0x0044,
    0x004D, 0x0032, 0x0000,
];
pub static RUSTCONV_WSTR_B06462BF2383: [u16; 14] = [
    0x004Du16, 0x0041, 0x0054, 0x0043, 0x0048, 0x0049, 0x004E, 0x0047, 0x005F, 0x0046,
    0x0049, 0x004C, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_B17D467E991B: [u16; 12] = [
    0x0050u16, 0x0041, 0x0054, 0x0043, 0x0048, 0x005F, 0x0054, 0x0041, 0x0047, 0x0049,
    0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_B1863B1A4495: [u16; 16] = [
    0x004Du16, 0x0041, 0x0054, 0x0043, 0x0048, 0x005F, 0x004C, 0x004F, 0x0047, 0x0049,
    0x0043, 0x005F, 0x004E, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_B552DCA322AE: [u16; 15] = [
    0x004Cu16, 0x0049, 0x004E, 0x004B, 0x0045, 0x0052, 0x005F, 0x0056, 0x0045, 0x0052,
    0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_B587DA256A81: [u16; 20] = [
    0x0042u16, 0x0049, 0x004E, 0x005F, 0x0050, 0x0052, 0x004F, 0x0044, 0x0055, 0x0043,
    0x0054, 0x005F, 0x0056, 0x0045, 0x0052, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_B671916D2EE2: [u16; 8] = [0x0049u16, 0x004E, 0x0044, 0x0045, 0x0058, 0x0045, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_B6CAE64F37AA: [u16; 13] = [
    0x0050u16, 0x0052, 0x004F, 0x0044, 0x0055, 0x0043, 0x0054, 0x005F, 0x004E, 0x0041,
    0x004D, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_B8167400E419: [u16; 15] = [
    0x0050u16, 0x0052, 0x0045, 0x0056, 0x004F, 0x0053, 0x004D, 0x0049, 0x004E, 0x004F,
    0x0052, 0x0056, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_BB1362D9E6DE: [u16; 13] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x0053, 0x005F, 0x004E, 0x0054, 0x0056, 0x0044,
    0x004D, 0x0031, 0x0000,
];
pub static RUSTCONV_WSTR_BB3D9904EA92: [u16; 7] = [0x004Fu16, 0x0046, 0x0046, 0x0053, 0x0045, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_BB5AFDE08C72: [u16; 14] = [
    0x0054u16, 0x0045, 0x004C, 0x0045, 0x004D, 0x0045, 0x0054, 0x0052, 0x0059, 0x005F,
    0x004F, 0x0046, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_C0EC3AAA1CB1: [u16; 9] = [0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0052, 0x0045, 0x0046, 0x0000];
pub static RUSTCONV_WSTR_C28852BFFB39: [u16; 17] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x004D, 0x0041, 0x0053, 0x004B, 0x005F,
    0x0046, 0x0055, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_C3896EEFC410: [u16; 5] = [0x0046u16, 0x004C, 0x0041, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_C4BD7D9AD8D3: [u16; 5] = [0x004Cu16, 0x0049, 0x004E, 0x004B, 0x0000];
pub static RUSTCONV_WSTR_C5135685EDAE: [u16; 7] = [0x004Du16, 0x004F, 0x0044, 0x0055, 0x004C, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_C919B17B4435: [u16; 12] = [
    0x004Cu16, 0x0041, 0x0059, 0x0045, 0x0052, 0x005F, 0x0054, 0x0041, 0x0047, 0x0049,
    0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_CA3F615CF4CC: [u16; 15] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x004D, 0x0041, 0x0053, 0x004B, 0x005F,
    0x0055, 0x0053, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_CE448C5F416D: [u16; 20] = [
    0x0043u16, 0x004F, 0x004E, 0x0054, 0x0045, 0x0058, 0x0054, 0x005F, 0x0050, 0x004C,
    0x0041, 0x0054, 0x0046, 0x004F, 0x0052, 0x004D, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_D3291E1A0630: [u16; 12] = [
    0x0043u16, 0x004F, 0x004E, 0x0054, 0x0045, 0x0058, 0x0054, 0x005F, 0x0052, 0x0045,
    0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_D49805BCC08B: [u16; 4] = [0x0041u16, 0x0050, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_D6E57B6014E8: [u16; 18] = [
    0x0049u16, 0x004E, 0x0043, 0x004C, 0x0055, 0x0044, 0x0045, 0x0045, 0x0058, 0x0043,
    0x004C, 0x0055, 0x0044, 0x0045, 0x0044, 0x004C, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_D76DA5FAF562: [u16; 10] = [0x0056u16, 0x0049, 0x0053, 0x0054, 0x0041, 0x005F, 0x0053, 0x004B, 0x0055, 0x0000];
pub static RUSTCONV_WSTR_D77710FDBACE: [u16; 12] = [
    0x0045u16, 0x0058, 0x0050, 0x004F, 0x0052, 0x0054, 0x005F, 0x004E, 0x0041, 0x004D,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_DB6102B2FB35: [u16; 16] = [
    0x0050u16, 0x0052, 0x004F, 0x0042, 0x004C, 0x0045, 0x004D, 0x0053, 0x0045, 0x0056,
    0x0045, 0x0052, 0x0049, 0x0054, 0x0059, 0x0000,
];
pub static RUSTCONV_WSTR_DB6DFF48431C: [u16; 10] = [0x0049u16, 0x004E, 0x0045, 0x0058, 0x0043, 0x004C, 0x0055, 0x0044, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_DC7FC9A9E687: [u16; 5] = [0x0053u16, 0x0048, 0x0049, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_DE2D8D7AC13B: [u16; 14] = [
    0x0043u16, 0x004F, 0x004E, 0x0054, 0x0045, 0x0058, 0x0054, 0x005F, 0x0054, 0x0041,
    0x0047, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_DED555D87C6E: [u16; 16] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x004D, 0x0041, 0x0053, 0x004B, 0x005F,
    0x0053, 0x0048, 0x0045, 0x004C, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_E0691A30E031: [u16; 7] = [0x004Fu16, 0x0053, 0x005F, 0x0053, 0x004B, 0x0055, 0x0000];
pub static RUSTCONV_WSTR_E2A090ED8622: [u16; 13] = [
    0x0053u16, 0x0058, 0x0053, 0x005F, 0x004D, 0x0041, 0x004E, 0x0049, 0x0046, 0x0045,
    0x0053, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_E41A40BFBF21: [u16; 8] = [0x0044u16, 0x004C, 0x004C, 0x0046, 0x0049, 0x004C, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_E48941AB8639: [u16; 7] = [0x0045u16, 0x004E, 0x0047, 0x0049, 0x004E, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_E4B0B51FACC5: [u16; 17] = [
    0x0050u16, 0x0041, 0x0052, 0x0041, 0x004D, 0x0045, 0x0054, 0x0045, 0x0052, 0x0031,
    0x005F, 0x0052, 0x0043, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_E66469B4899A: [u16; 5] = [0x004Eu16, 0x0055, 0x004C, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_E7D2A61F0DB4: [u16; 15] = [
    0x004Du16, 0x0053, 0x0049, 0x005F, 0x0050, 0x0041, 0x0043, 0x004B, 0x0041, 0x0047,
    0x0045, 0x005F, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_E9F811BC0131: [u16; 8] = [0x004Cu16, 0x0049, 0x0042, 0x0052, 0x0041, 0x0052, 0x0059, 0x0000];
pub static RUSTCONV_WSTR_EBAA476B14FC: [u16; 13] = [
    0x0046u16, 0x004C, 0x0041, 0x0047, 0x005F, 0x0049, 0x004E, 0x0053, 0x0054, 0x0041,
    0x004C, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_ECF474222B3A: [u16; 14] = [
    0x0042u16, 0x004C, 0x004F, 0x0043, 0x004B, 0x005F, 0x0055, 0x0050, 0x0047, 0x0052,
    0x0041, 0x0044, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_EDD6FD03C130: [u16; 7] = [0x0041u16, 0x0050, 0x0050, 0x005F, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_F2FCDDCD4868: [u16; 22] = [
    0x0055u16, 0x0050, 0x0054, 0x004F, 0x005F, 0x0042, 0x0049, 0x004E, 0x005F, 0x0046,
    0x0049, 0x004C, 0x0045, 0x005F, 0x0056, 0x0045, 0x0052, 0x0053, 0x0049, 0x004F,
    0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_F3DE02DAE7D2: [u16; 14] = [
    0x0041u16, 0x0050, 0x0050, 0x0048, 0x0045, 0x004C, 0x0050, 0x005F, 0x0054, 0x0049,
    0x0054, 0x004C, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_F5DD91F8FDAE: [u16; 8] = [0x0041u16, 0x0050, 0x0050, 0x0048, 0x0045, 0x004C, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_F7B12ACC25D7: [u16; 17] = [
    0x0053u16, 0x0054, 0x0052, 0x0049, 0x004E, 0x0047, 0x0054, 0x0041, 0x0042, 0x004C,
    0x0045, 0x005F, 0x0049, 0x0054, 0x0045, 0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_F870827E97A5: [u16; 7] = [0x0055u16, 0x0052, 0x004C, 0x005F, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_F94C502C67CB: [u16; 7] = [0x0056u16, 0x0045, 0x004E, 0x0044, 0x004F, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_FC353EB66ACC: [u16; 18] = [
    0x0052u16, 0x0045, 0x0049, 0x004E, 0x0053, 0x0054, 0x0041, 0x004C, 0x004C, 0x005F,
    0x0055, 0x0050, 0x0047, 0x0052, 0x0041, 0x0044, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_FCCF0BD8C37D: [u16; 7] = [0x0046u16, 0x0049, 0x0058, 0x005F, 0x0049, 0x0044, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
}
extern "C" {
    fn ShimDbgPrint(level: i32, func: PCSTR, fmt: PCSTR, ...);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut GUID_DATABASE_MSI: GUID = _GUID {
    Data1: 0xd8ff6d16 as ::core::ffi::c_uint,
    Data2: 0x6a3a as ::core::ffi::c_ushort,
    Data3: 0x468a as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut GUID_DATABASE_SHIM: GUID = _GUID {
    Data1: 0x11111111 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1111 as ::core::ffi::c_ushort,
    Data3: 0x1111 as ::core::ffi::c_ushort,
    Data4: [
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut GUID_DATABASE_DRIVERS: GUID = _GUID {
    Data1: 0xf9ab2228 as ::core::ffi::c_uint,
    Data2: 0x3312 as ::core::ffi::c_ushort,
    Data3: 0x4a73 as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut g_Heap: HANDLE = ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn SdbpHeapInit() {
    g_Heap = RtlCreateHeap(
        HEAP_GROWABLE as ULONG,
        core::ptr::null_mut(),
        0 as SIZE_T,
        0x10000 as SIZE_T,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as HANDLE;
    // SDBAPI_DEBUG_ALLOC is 0: SdbpDebugHeapInit() is compiled out in the C.
}
#[no_mangle]
pub unsafe extern "C" fn SdbpHeapDeinit() {
    // SDBAPI_DEBUG_ALLOC is 0: SdbpDebugHeapDeinit() is compiled out in the C.
    RtlDestroyHeap(g_Heap);
}

unsafe extern "C" fn SdbpHeap() -> HANDLE {
    return g_Heap;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpAlloc(size: SIZE_T) -> LPVOID {
    let mut mem: LPVOID = RtlAllocateHeap(SdbpHeap() as PVOID, HEAP_ZERO_MEMORY as ULONG, size) as LPVOID;
    return mem;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpReAlloc(mem: LPVOID, size: SIZE_T, oldSize: SIZE_T) -> LPVOID {
    let mut newmem: LPVOID = RtlReAllocateHeap(
        SdbpHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        mem as PVOID,
        size,
    ) as LPVOID;
    return newmem;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpFree(mem: LPVOID) {
    // SDBAPI_DEBUG_ALLOC is 0: SdbpRemoveAllocation() is compiled out in the C.
    RtlFreeHeap(SdbpHeap(), 0, mem);
}

#[no_mangle]
pub unsafe extern "system" fn SdbpCreate(path: LPCWSTR, type_: PATH_TYPE, write: BOOL) -> PDB {
    let mut Status: NTSTATUS;
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut attr: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut str: _UNICODE_STRING = core::mem::zeroed();
    let mut pdb: PDB;

    if type_ == DOS_PATH {
        if RtlDosPathNameToNtPathName_U(
            path as PCWSTR,
            core::ptr::addr_of_mut!(str),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            return core::ptr::null_mut();
        }
    } else {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(str), path as PCWSTR);
    }

    /* SdbAlloc zeroes the memory. */
    pdb = SdbAlloc!(core::mem::size_of::<DB>()) as PDB;
    if pdb.is_null() {
        SHIM_ERR!(b"Failed to allocate memory for shim database\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    // InitializeObjectAttributes(&attr, &str, OBJ_CASE_INSENSITIVE, NULL, NULL), expanded inline
    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = core::ptr::null_mut();
    attr.ObjectName = core::ptr::addr_of_mut!(str);
    attr.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        core::ptr::addr_of_mut!((*pdb).file),
        ((if write != 0 {
            FILE_GENERIC_WRITE
        } else {
            FILE_GENERIC_READ
        }) | SYNCHRONIZE) as ACCESS_MASK,
        core::ptr::addr_of_mut!(attr),
        core::ptr::addr_of_mut!(io),
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL as ULONG,
        FILE_SHARE_READ as ULONG,
        (if write != 0 {
            FILE_SUPERSEDE
        } else {
            FILE_OPEN
        }) as ULONG,
        (FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT) as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );

    (*pdb).for_write = write;

    if type_ == DOS_PATH {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(str));
    }

    if !NT_SUCCESS!(Status) {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(
            b"Failed to create shim database file: %lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return core::ptr::null_mut();
    }

    return pdb;
}
#[no_mangle]
pub unsafe extern "system" fn SdbpFlush(mut pdb: PDB) {
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    if (*pdb).for_write == 0 {
        RtlAssert(
            b"pdb->for_write\0".as_ptr() as *const i8 as PVOID,
            b"dll/appcompat/apphelp/sdbapi.c\0".as_ptr() as *const i8 as PVOID,
            145 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    Status = NtWriteFile(
        (*pdb).file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io,
        (*pdb).data as PVOID,
        (*pdb).write_iter as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        SHIM_WARN!(
            b"failed with 0x%lx\n\0".as_ptr() as *const i8,
            Status,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn SdbpStrlen(string: PCWSTR) -> DWORD {
    return wcslen(string) as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn SdbpStrsize(string: PCWSTR) -> DWORD {
    return (SdbpStrlen(string).wrapping_add(1))
        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn SdbpStrDup(string: LPCWSTR) -> PWSTR {
    let mut ret: PWSTR = SdbAlloc!(SdbpStrsize(string)) as PWSTR;
    wcscpy(ret, string);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SdbpOpenMemMappedFile(path: LPCWSTR, mapping: PMEMMAPPED) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    core::ptr::write_bytes(mapping, 0u8, 1);

    RtlInitUnicodeString(core::ptr::addr_of_mut!(FileName), path);

    // InitializeObjectAttributes(&ObjectAttributes, &FileName, COMMENT_TOKEN_ECHO____, NULL, NULL)
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FileName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!((*mapping).file),
        GENERIC_READ | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    if Status == STATUS_OBJECT_NAME_INVALID || Status == STATUS_OBJECT_PATH_SYNTAX_BAD {
        if RtlDosPathNameToNtPathName_U(
            path,
            core::ptr::addr_of_mut!(FileName),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            SHIM_ERR!(b"Failed to convert %S to Nt path: 0x%lx\n\0".as_ptr() as *const i8,
                path,
                Status);
            return FALSE as BOOL;
        }
        // InitializeObjectAttributes(&ObjectAttributes, &FileName, COMMENT_TOKEN_ECHO____, NULL, NULL)
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FileName);
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            core::ptr::addr_of_mut!((*mapping).file),
            GENERIC_READ | SYNCHRONIZE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            FILE_SHARE_READ,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(FileName));
    }

    if !NT_SUCCESS!(Status) {
        SHIM_ERR!(b"Failed to open file %S: 0x%lx\n\0".as_ptr() as *const i8,
            path,
            Status);
        return FALSE as BOOL;
    }

    'err_out: {
        Status = NtCreateSection(
            core::ptr::addr_of_mut!((*mapping).section),
            STANDARD_RIGHTS_REQUIRED | SECTION_QUERY | SECTION_MAP_READ,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            PAGE_READONLY,
            SEC_COMMIT,
            (*mapping).file,
        );
        if !NT_SUCCESS!(Status) {
            /* Special case */
            if Status == STATUS_MAPPED_FILE_SIZE_ZERO {
                NtClose((*mapping).file);
                (*mapping).section = core::ptr::null_mut();
                (*mapping).file = (*mapping).section;
                return TRUE as BOOL;
            }
            SHIM_ERR!(b"Failed to create mapping for file: 0x%lx\n\0".as_ptr() as *const i8,
                Status);
            break 'err_out;
        }

        Status = NtQueryInformationFile(
            (*mapping).file,
            core::ptr::addr_of_mut!(IoStatusBlock),
            core::ptr::addr_of_mut!(FileStandard) as PVOID,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
            FileStandardInformation,
        );
        if !NT_SUCCESS!(Status) {
            SHIM_ERR!(b"Failed to read file info for file: 0x%lx\n\0".as_ptr() as *const i8,
                Status);
            break 'err_out;
        }

        (*mapping).mapped_size = FileStandard.EndOfFile.__bindgen_anon_1.LowPart as SIZE_T;
        (*mapping).size = (*mapping).mapped_size;
        Status = NtMapViewOfSection(
            (*mapping).section,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!((*mapping).view) as *mut PVOID,
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*mapping).mapped_size),
            ViewUnmap,
            0,
            PAGE_READONLY,
        );
        if !NT_SUCCESS!(Status) {
            SHIM_ERR!(b"Failed to map view of file: 0x%lx\n\0".as_ptr() as *const i8,
                Status);
            break 'err_out;
        }

        return TRUE as BOOL;
    }

    // err_out:
    if (*mapping).view.is_null() {
        if !(*mapping).section.is_null() {
            NtClose((*mapping).section);
        }
        NtClose((*mapping).file);
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbpCloseMemMappedFile(mapping: PMEMMAPPED) {
    /* Prevent a VAD warning */
    if !(*mapping).view.is_null() {
        NtUnmapViewOfSection(NtCurrentProcess(), (*mapping).view as PVOID);
    }
    NtClose((*mapping).section);
    NtClose((*mapping).file);
    core::ptr::write_bytes(mapping, 0, 1); // RtlZeroMemory(mapping, sizeof(*mapping))
}
#[no_mangle]
pub unsafe extern "system" fn SdbpCheckTagType(mut tag: TAG, mut type_0: WORD) -> BOOL {
    if (tag as u32 & TAG_TYPE_MASK) != type_0 as u32 {
        return FALSE as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn SdbpCheckTagIDType(mut pdb: PDB, mut tagid: TAGID, mut type_0: WORD) -> BOOL {
    let mut tag: TAG = SdbGetTagFromTagID(pdb, tagid);
    if tag == (TAG_NULL as u16) {
        return FALSE as BOOL;
    }
    return SdbpCheckTagType(tag, type_0);
}
#[no_mangle]
pub unsafe extern "C" fn SdbpOpenDatabase(path: LPCWSTR, type_: PATH_TYPE) -> PDB {
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut fsi: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut pdb: PDB = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut header: [BYTE; 12] = [0; 12];

    pdb = SdbpCreate(path, type_, FALSE as BOOL);
    if pdb.is_null() {
        return core::ptr::null_mut();
    }

    Status = NtQueryInformationFile(
        (*pdb).file,
        core::ptr::addr_of_mut!(io),
        core::ptr::addr_of_mut!(fsi) as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        FileStandardInformation,
    );
    if !NT_SUCCESS!(Status) {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(b"Failed to get shim database size: 0x%lx\n\0".as_ptr() as *const i8, Status);
        return core::ptr::null_mut();
    }

    (*pdb).size = fsi.EndOfFile.u.LowPart;
    (*pdb).data = SdbAlloc!((*pdb).size) as *mut BYTE;
    Status = NtReadFile(
        (*pdb).file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(io),
        (*pdb).data as PVOID,
        (*pdb).size as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS!(Status) {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(b"Failed to open shim database file: 0x%lx\n\0".as_ptr() as *const i8, Status);
        return core::ptr::null_mut();
    }

    if SdbpReadData(
        pdb,
        core::ptr::addr_of_mut!(header) as PVOID,
        0,
        12,
    ) == 0
    {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(b"Failed to read shim database header\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    if memcmp(
        (core::ptr::addr_of_mut!(header) as *mut BYTE).offset(8 as isize)
            as *const core::ffi::c_void,
        b"sdbf\0".as_ptr() as *const core::ffi::c_void,
        4,
    ) != 0
    {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(b"Shim database header is invalid\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    (*pdb).major = *((core::ptr::addr_of_mut!(header) as *mut BYTE)
        .offset(0 as isize) as *mut DWORD);
    (*pdb).minor = *((core::ptr::addr_of_mut!(header) as *mut BYTE)
        .offset(4 as isize) as *mut DWORD);

    return pdb;
}

#[no_mangle]
pub unsafe extern "system" fn SdbOpenDatabase(path: LPCWSTR, type_: PATH_TYPE) -> PDB {
    let mut pdb: PDB;
    let mut root: TAGID;
    let mut name: TAGID;

    pdb = SdbpOpenDatabase(path, type_);
    if pdb.is_null() {
        return core::ptr::null_mut();
    }

    if (*pdb).major != 2 && (*pdb).major != 3 {
        SdbCloseDatabase(pdb);
        SHIM_ERR!(b"Invalid shim database version\n\0".as_ptr() as PCSTR);
        return core::ptr::null_mut();
    }

    (*pdb).stringtable = SdbFindFirstTag(pdb, TAGID_ROOT, TAG_STRINGTABLE as TAG);
    if SdbGetDatabaseID(pdb, core::ptr::addr_of_mut!((*pdb).database_id)) == 0 {
        SHIM_INFO!(b"Failed to get the database id\n\0".as_ptr() as PCSTR);
    }

    root = SdbFindFirstTag(pdb, TAGID_ROOT, TAG_DATABASE as TAG);
    if root != TAGID_NULL {
        name = SdbFindFirstTag(pdb, root, TAG_NAME as TAG);
        if name != TAGID_NULL {
            (*pdb).database_name = SdbGetStringTagPtr(pdb, name) as PCWSTR;
        }
    }
    if (*pdb).database_name.is_null() {
        SHIM_INFO!(b"Failed to get the database name\n\0".as_ptr() as PCSTR);
    }

    return pdb;
}

#[no_mangle]
pub unsafe extern "system" fn SdbCloseDatabase(pdb: PDB) {
    if pdb.is_null() {
        return;
    }

    if !(*pdb).file.is_null() {
        NtClose((*pdb).file);
    }
    if !(*pdb).string_buffer.is_null() {
        SdbCloseDatabase((*pdb).string_buffer);
    }
    if !(*pdb).string_lookup.is_null() {
        SdbpTableDestroy(core::ptr::addr_of_mut!((*pdb).string_lookup));
    }
    SdbFree!((*pdb).data);
    SdbFree!(pdb);
}

#[no_mangle]
pub unsafe extern "system" fn SdbGUIDFromString(GuidString: PCWSTR, Guid: *mut GUID) -> BOOL {
    let mut GuidString_u: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(core::ptr::addr_of_mut!(GuidString_u), GuidString);
    return NT_SUCCESS!(RtlGUIDFromString(core::ptr::addr_of_mut!(GuidString_u), Guid)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGUIDToString(Guid: *const GUID, GuidString: PWSTR, Length: SIZE_T) -> BOOL {
    let mut GuidString_u: _UNICODE_STRING = core::mem::zeroed();
    if NT_SUCCESS!(RtlStringFromGUID(Guid, core::ptr::addr_of_mut!(GuidString_u))) {
        let mut hr: HRESULT = StringCchCopyNW(
            GuidString,
            Length as usize,
            GuidString_u.Buffer,
            (GuidString_u.Length as usize) / core::mem::size_of::<WCHAR>(),
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(GuidString_u));
        return (hr >= 0) as BOOL;
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbIsNullGUID(Guid: *const GUID) -> BOOL {
    static mut NullGuid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    return (Guid.is_null() || IsEqualGUID!(core::ptr::addr_of!(NullGuid), Guid)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetStandardDatabaseGUID(Flags: DWORD, Guid: *mut GUID) -> BOOL {
    let mut copy_from: *const GUID = core::ptr::null();
    match Flags & HID_DATABASE_TYPE_MASK {
        SDB_DATABASE_MAIN_MSI => {
            copy_from = core::ptr::addr_of!(GUID_DATABASE_MSI);
        }
        SDB_DATABASE_MAIN_SHIM => {
            copy_from = core::ptr::addr_of!(GUID_DATABASE_SHIM);
        }
        SDB_DATABASE_MAIN_DRIVERS => {
            copy_from = core::ptr::addr_of!(GUID_DATABASE_DRIVERS);
        }
        _ => {
            SHIM_ERR!(b"Cannot obtain database guid for databases other than main\n\0".as_ptr() as *const i8);
            return (FALSE) as BOOL;
        }
    }
    if !Guid.is_null() {
        memcpy(
            Guid as *mut core::ffi::c_void,
            copy_from as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetDatabaseVersion(database: LPCWSTR, VersionHi: PDWORD, VersionLo: PDWORD) -> BOOL {
    let mut pdb: PDB;
    pdb = SdbpOpenDatabase(database, DOS_PATH);
    if !pdb.is_null() {
        *VersionHi = (*pdb).major;
        *VersionLo = (*pdb).minor;
        SdbCloseDatabase(pdb);
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetDatabaseInformation(pdb: PDB, information: PDB_INFORMATION) -> BOOL {
    RtlZeroMemory!(information as *mut core::ffi::c_void, core::mem::size_of::<_DB_INFORMATION>());

    if !pdb.is_null() {
        (*information).dwFlags = 0;
        (*information).dwMajor = (*pdb).major;
        (*information).dwMinor = (*pdb).minor;
        (*information).Description = (*pdb).database_name as LPCWSTR;
        if SdbIsNullGUID(core::ptr::addr_of_mut!((*pdb).database_id)) == 0 {
            (*information).dwFlags |= DB_INFO_FLAGS_VALID_GUID;
            (*information).Id = (*pdb).database_id;
        }
        return TRUE as BOOL;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetDatabaseInformationByName(lpwszFileName: LPCWSTR, ppAttrInfo: *mut PDB_INFORMATION) -> BOOL {
    SHIM_ERR!(b"Unimplemented\n\0".as_ptr() as *const i8);
    *ppAttrInfo = core::ptr::null_mut();
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbFreeDatabaseInformation(mut information: PDB_INFORMATION) {
    if g_ShimDebugLevel != 0 {
        ShimDbgPrint(
            SHIM_ERR as i32,
            b"SdbFreeDatabaseInformation\0".as_ptr() as PCSTR,
            b"Unimplemented\n\0".as_ptr() as PCSTR,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn SdbFindFirstNamedTag(pdb: PDB, root: TAGID, find: TAGID, nametag: TAGID, find_name: LPCWSTR) -> TAGID {
    let mut iter: TAGID;

    iter = SdbFindFirstTag(pdb, root, find as TAG);

    while iter != TAGID_NULL as TAGID {
        let tmp: TAGID = SdbFindFirstTag(pdb, iter, nametag as TAG);
        if tmp != TAGID_NULL as TAGID {
            let name: LPCWSTR = SdbGetStringTagPtr(pdb, tmp) as LPCWSTR;
            if !name.is_null() && _wcsicmp(name, find_name) != 0 {
                return iter;
            }
        }
        iter = SdbFindNextTag(pdb, root, iter);
    }
    return TAGID_NULL as TAGID;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetLayerTagRef(mut hsdb: HSDB, mut layerName: LPCWSTR) -> TAGREF {
    let mut pdb: PDB = (*hsdb).pdb;

    let mut database: TAGID = SdbFindFirstTag(pdb, TAGID_ROOT as TAGID, TAG_DATABASE as TAG);
    if database != TAGID_NULL as TAGID {
        let mut layer: TAGID = SdbFindFirstNamedTag(
            pdb,
            database,
            TAG_LAYER as TAGID,
            TAG_NAME as TAGID,
            layerName,
        );
        if layer != TAGID_NULL as TAGID {
            let mut tr: TAGREF = 0;
            if SdbTagIDToTagRef(hsdb, pdb, layer, core::ptr::addr_of_mut!(tr)) != 0 {
                return tr;
            }
        }
    }
    return TAGREF_NULL as TAGREF;
}

#[no_mangle]
pub unsafe extern "system" fn SdbQueryDataExTagID(mut pdb: PDB, mut tiExe: TAGID, mut lpszDataName: LPCWSTR, mut lpdwDataType: LPDWORD, mut lpBuffer: LPVOID, mut lpcbBufferSize: LPDWORD, mut ptiData: *mut TAGID) -> DWORD {
    let mut tiData: TAGID = 0;
    let mut tiValueType: TAGID = 0;
    let mut tiValue: TAGID = 0;
    let mut dwDataType: DWORD = 0;
    let mut dwSizeRequired: DWORD = 0;
    let mut dwInputSize: DWORD = 0;
    let mut lpStringData: LPCWSTR = core::ptr::null::<WCHAR>();
    if lpszDataName.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    tiData = SdbFindFirstNamedTag(
        pdb,
        tiExe,
        TAG_DATA as TAGID,
        TAG_NAME as TAGID,
        lpszDataName,
    );
    if tiData == TAGID_NULL as TAGID {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_INFO as i32,
                b"SdbQueryDataExTagID\0" as *const u8 as PCSTR,
                b"No data tag found\n\0" as *const u8 as PCSTR,
            );
        }
        return ERROR_NOT_FOUND as DWORD;
    }
    if !ptiData.is_null() {
        *ptiData = tiData;
    }
    tiValueType = SdbFindFirstTag(pdb, tiData, TAG_DATA_VALUETYPE as TAG);
    if tiValueType == TAGID_NULL as TAGID {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_WARN as i32,
                b"SdbQueryDataExTagID\0" as *const u8 as PCSTR,
                b"Data tag (0x%x) without valuetype\n\0" as *const u8 as PCSTR,
                tiData,
            );
        }
        return ERROR_INTERNAL_DB_CORRUPTION as DWORD;
    }
    dwDataType = SdbReadDWORDTag(pdb, tiValueType, 0 as DWORD);
    match dwDataType {
        REG_SZ => {
            tiValue = SdbFindFirstTag(pdb, tiData, TAG_DATA_STRING as TAG);
        }
        REG_DWORD => {
            tiValue = SdbFindFirstTag(pdb, tiData, TAG_DATA_DWORD as TAG);
        }
        REG_QWORD => {
            tiValue = SdbFindFirstTag(pdb, tiData, TAG_DATA_QWORD as TAG);
        }
        _ => {
            if g_ShimDebugLevel != 0 {
                ShimDbgPrint(
                    SHIM_WARN as i32,
                    b"SdbQueryDataExTagID\0" as *const u8 as PCSTR,
                    b"Unsupported dwDataType=0x%x\n\0" as *const u8 as PCSTR,
                    dwDataType,
                );
            }
            return ERROR_INVALID_PARAMETER as DWORD;
        }
    }
    if !lpdwDataType.is_null() {
        *lpdwDataType = dwDataType;
    }
    if tiValue == TAGID_NULL as TAGID {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_WARN as i32,
                b"SdbQueryDataExTagID\0" as *const u8 as PCSTR,
                b"Data tag (0x%x) without data\n\0" as *const u8 as PCSTR,
                tiData,
            );
        }
        return ERROR_INTERNAL_DB_CORRUPTION as DWORD;
    }
    if dwDataType != REG_SZ as DWORD {
        dwSizeRequired = SdbGetTagDataSize(pdb, tiValue);
    } else {
        lpStringData = SdbpGetString(
            pdb,
            tiValue,
            core::ptr::addr_of_mut!(dwSizeRequired),
        ) as LPCWSTR;
        if lpStringData.is_null() {
            return ERROR_INTERNAL_DB_CORRUPTION as DWORD;
        }
    }
    if lpcbBufferSize.is_null() {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }
    dwInputSize = *lpcbBufferSize;
    *lpcbBufferSize = dwSizeRequired;
    if dwInputSize < dwSizeRequired || lpBuffer.is_null() {
        if g_ShimDebugLevel != 0 {
            ShimDbgPrint(
                SHIM_WARN as i32,
                b"SdbQueryDataExTagID\0" as *const u8 as PCSTR,
                b"dwInputSize %u not sufficient to hold %u bytes\n\0" as *const u8 as PCSTR,
                dwInputSize,
                dwSizeRequired,
            );
        }
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }
    if dwDataType != REG_SZ as DWORD {
        SdbpReadData(
            pdb,
            lpBuffer,
            (tiValue as usize).wrapping_add(core::mem::size_of::<TAG>() as usize)
                as DWORD,
            dwSizeRequired,
        );
    } else {
        StringCbCopyNW(
            lpBuffer as STRSAFE_LPWSTR,
            dwInputSize as size_t,
            lpStringData as STRSAFE_LPCWSTR,
            dwSizeRequired as size_t,
        );
    }
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SdbMakeIndexKeyFromString(mut str: LPCWSTR) -> LONGLONG {
    let mut result: LONGLONG = 0;
    let mut shift: i32 = 56;
    while (*str as i32) != 0 && shift >= 0 {
        let fresh31 = str;
        str = str.offset(1);
        let mut c: WCHAR = toupper(*fresh31 as i32) as WCHAR;
        if ((c as i32) & 0xff) != 0 {
            result |= (((c as i32) & 0xff) as LONGLONG) << shift;
            shift -= 8;
        }
        if shift < 0 {
            break;
        }
        c = ((c as i32) >> 8) as WCHAR;
        if ((c as i32) & 0xff) != 0 {
            result |= (((c as i32) & 0xff) as LONGLONG) << shift;
            shift -= 8;
        }
    }
    return result;
}

#[no_mangle]
pub unsafe extern "system" fn SdbTagToString(tag: TAG) -> LPCWSTR {
    match tag as u32 {
        TAG_NULL => { return RUSTCONV_WSTR_E66469B4899A.as_ptr(); }
        TAG_INCLUDE => { return RUSTCONV_WSTR_697A667BA010.as_ptr(); }
        TAG_GENERAL => { return RUSTCONV_WSTR_9E3130242CB2.as_ptr(); }
        TAG_MATCH_LOGIC_NOT => { return RUSTCONV_WSTR_B1863B1A4495.as_ptr(); }
        TAG_APPLY_ALL_SHIMS => { return RUSTCONV_WSTR_23C16E939718.as_ptr(); }
        TAG_USE_SERVICE_PACK_FILES => { return RUSTCONV_WSTR_873D19CF8F2F.as_ptr(); }
        TAG_MITIGATION_OS => { return RUSTCONV_WSTR_9D8DE29A0BF8.as_ptr(); }
        TAG_BLOCK_UPGRADE => { return RUSTCONV_WSTR_ECF474222B3A.as_ptr(); }
        TAG_INCLUDEEXCLUDEDLL => { return RUSTCONV_WSTR_D6E57B6014E8.as_ptr(); }
        TAG_RAC_EVENT_OFF => { return RUSTCONV_WSTR_4E11801F47A5.as_ptr(); }
        TAG_TELEMETRY_OFF => { return RUSTCONV_WSTR_BB5AFDE08C72.as_ptr(); }
        TAG_SHIM_ENGINE_OFF => { return RUSTCONV_WSTR_5A1067373017.as_ptr(); }
        TAG_LAYER_PROPAGATION_OFF => { return RUSTCONV_WSTR_2B241FEDD36E.as_ptr(); }
        TAG_REINSTALL_UPGRADE => { return RUSTCONV_WSTR_FC353EB66ACC.as_ptr(); }
        TAG_MATCH_MODE => { return RUSTCONV_WSTR_373CD992ADB9.as_ptr(); }
        TAG_TAG => { return RUSTCONV_WSTR_454A87B5217E.as_ptr(); }
        TAG_INDEX_TAG => { return RUSTCONV_WSTR_89F718515BCC.as_ptr(); }
        TAG_INDEX_KEY => { return RUSTCONV_WSTR_7E591A73C957.as_ptr(); }
        TAG_SIZE => { return RUSTCONV_WSTR_5B0F7F262758.as_ptr(); }
        TAG_OFFSET => { return RUSTCONV_WSTR_BB3D9904EA92.as_ptr(); }
        TAG_CHECKSUM => { return RUSTCONV_WSTR_1314F3F7733B.as_ptr(); }
        TAG_SHIM_TAGID => { return RUSTCONV_WSTR_4AED236F221B.as_ptr(); }
        TAG_PATCH_TAGID => { return RUSTCONV_WSTR_B17D467E991B.as_ptr(); }
        TAG_MODULE_TYPE => { return RUSTCONV_WSTR_3EF99EF5AAFA.as_ptr(); }
        TAG_VERDATEHI => { return RUSTCONV_WSTR_9FDE1970188E.as_ptr(); }
        TAG_VERDATELO => { return RUSTCONV_WSTR_7A76B31D3E31.as_ptr(); }
        TAG_VERFILEOS => { return RUSTCONV_WSTR_242AEF5C2E9D.as_ptr(); }
        TAG_VERFILETYPE => { return RUSTCONV_WSTR_6BE264DC8A4F.as_ptr(); }
        TAG_PE_CHECKSUM => { return RUSTCONV_WSTR_721C10FFCE5B.as_ptr(); }
        TAG_PREVOSMAJORVER => { return RUSTCONV_WSTR_0C546AC9EAC2.as_ptr(); }
        TAG_PREVOSMINORVER => { return RUSTCONV_WSTR_B8167400E419.as_ptr(); }
        TAG_PREVOSPLATFORMID => { return RUSTCONV_WSTR_3FAE7E59DC04.as_ptr(); }
        TAG_PREVOSBUILDNO => { return RUSTCONV_WSTR_ACBCC65B9C1D.as_ptr(); }
        TAG_PROBLEMSEVERITY => { return RUSTCONV_WSTR_DB6102B2FB35.as_ptr(); }
        TAG_LANGID => { return RUSTCONV_WSTR_3F17A6C8DD89.as_ptr(); }
        TAG_VER_LANGUAGE => { return RUSTCONV_WSTR_18669BB540DB.as_ptr(); }
        TAG_ENGINE => { return RUSTCONV_WSTR_E48941AB8639.as_ptr(); }
        TAG_HTMLHELPID => { return RUSTCONV_WSTR_27EE4809B6BF.as_ptr(); }
        TAG_INDEX_FLAGS => { return RUSTCONV_WSTR_59971EDD22BE.as_ptr(); }
        TAG_FLAGS => { return RUSTCONV_WSTR_25C0EB9DBCCB.as_ptr(); }
        TAG_DATA_VALUETYPE => { return RUSTCONV_WSTR_A8AB8C945814.as_ptr(); }
        TAG_DATA_DWORD => { return RUSTCONV_WSTR_7AC1B2A1871C.as_ptr(); }
        TAG_LAYER_TAGID => { return RUSTCONV_WSTR_C919B17B4435.as_ptr(); }
        TAG_MSI_TRANSFORM_TAGID => { return RUSTCONV_WSTR_5017923A686F.as_ptr(); }
        TAG_LINKER_VERSION => { return RUSTCONV_WSTR_B552DCA322AE.as_ptr(); }
        TAG_LINK_DATE => { return RUSTCONV_WSTR_4E0F1B69A26A.as_ptr(); }
        TAG_UPTO_LINK_DATE => { return RUSTCONV_WSTR_66888A2B7130.as_ptr(); }
        TAG_OS_SERVICE_PACK => { return RUSTCONV_WSTR_2333D85AE41A.as_ptr(); }
        TAG_FLAG_TAGID => { return RUSTCONV_WSTR_273396E20ECA.as_ptr(); }
        TAG_RUNTIME_PLATFORM => { return RUSTCONV_WSTR_289950D9E528.as_ptr(); }
        TAG_OS_SKU => { return RUSTCONV_WSTR_E0691A30E031.as_ptr(); }
        TAG_OS_PLATFORM => { return RUSTCONV_WSTR_75D8E40A76D2.as_ptr(); }
        TAG_APP_NAME_RC_ID => { return RUSTCONV_WSTR_38283C14387D.as_ptr(); }
        TAG_VENDOR_NAME_RC_ID => { return RUSTCONV_WSTR_7B2767ABD848.as_ptr(); }
        TAG_SUMMARY_MSG_RC_ID => { return RUSTCONV_WSTR_67DE5C266FFF.as_ptr(); }
        TAG_VISTA_SKU => { return RUSTCONV_WSTR_D76DA5FAF562.as_ptr(); }
        TAG_DESCRIPTION_RC_ID => { return RUSTCONV_WSTR_51959BA57352.as_ptr(); }
        TAG_PARAMETER1_RC_ID => { return RUSTCONV_WSTR_E4B0B51FACC5.as_ptr(); }
        TAG_CONTEXT_TAGID => { return RUSTCONV_WSTR_DE2D8D7AC13B.as_ptr(); }
        TAG_EXE_WRAPPER => { return RUSTCONV_WSTR_47395966419C.as_ptr(); }
        TAG_URL_ID => { return RUSTCONV_WSTR_F870827E97A5.as_ptr(); }
        TAG_TAGID => { return RUSTCONV_WSTR_38334143D9FA.as_ptr(); }
        TAG_TIME => { return RUSTCONV_WSTR_85289DCA13CE.as_ptr(); }
        TAG_BIN_FILE_VERSION => { return RUSTCONV_WSTR_57FECFF414E5.as_ptr(); }
        TAG_BIN_PRODUCT_VERSION => { return RUSTCONV_WSTR_B587DA256A81.as_ptr(); }
        TAG_MODTIME => { return RUSTCONV_WSTR_97B9270952CE.as_ptr(); }
        TAG_FLAG_MASK_KERNEL => { return RUSTCONV_WSTR_7DE7F613EDA8.as_ptr(); }
        TAG_UPTO_BIN_PRODUCT_VERSION => { return RUSTCONV_WSTR_4F3A9B3AE4CB.as_ptr(); }
        TAG_DATA_QWORD => { return RUSTCONV_WSTR_2D1087318FA4.as_ptr(); }
        TAG_FLAG_MASK_USER => { return RUSTCONV_WSTR_CA3F615CF4CC.as_ptr(); }
        TAG_FLAGS_NTVDM1 => { return RUSTCONV_WSTR_BB1362D9E6DE.as_ptr(); }
        TAG_FLAGS_NTVDM2 => { return RUSTCONV_WSTR_AD7B0A117883.as_ptr(); }
        TAG_FLAGS_NTVDM3 => { return RUSTCONV_WSTR_42A1743682CA.as_ptr(); }
        TAG_FLAG_MASK_SHELL => { return RUSTCONV_WSTR_DED555D87C6E.as_ptr(); }
        TAG_UPTO_BIN_FILE_VERSION => { return RUSTCONV_WSTR_F2FCDDCD4868.as_ptr(); }
        TAG_FLAG_MASK_FUSION => { return RUSTCONV_WSTR_C28852BFFB39.as_ptr(); }
        TAG_FLAG_PROCESSPARAM => { return RUSTCONV_WSTR_1D44D5A7D22B.as_ptr(); }
        TAG_FLAG_LUA => { return RUSTCONV_WSTR_328694CCA74D.as_ptr(); }
        TAG_FLAG_INSTALL => { return RUSTCONV_WSTR_EBAA476B14FC.as_ptr(); }
        TAG_NAME => { return RUSTCONV_WSTR_93C8910194A5.as_ptr(); }
        TAG_DESCRIPTION => { return RUSTCONV_WSTR_629A0B15E864.as_ptr(); }
        TAG_MODULE => { return RUSTCONV_WSTR_C5135685EDAE.as_ptr(); }
        TAG_API => { return RUSTCONV_WSTR_D49805BCC08B.as_ptr(); }
        TAG_VENDOR => { return RUSTCONV_WSTR_F94C502C67CB.as_ptr(); }
        TAG_APP_NAME => { return RUSTCONV_WSTR_16160A13BD6B.as_ptr(); }
        TAG_COMMAND_LINE => { return RUSTCONV_WSTR_6D2E68F65B35.as_ptr(); }
        TAG_COMPANY_NAME => { return RUSTCONV_WSTR_60BA521D1CA1.as_ptr(); }
        TAG_DLLFILE => { return RUSTCONV_WSTR_E41A40BFBF21.as_ptr(); }
        TAG_WILDCARD_NAME => { return RUSTCONV_WSTR_3801BF26E057.as_ptr(); }
        TAG_PRODUCT_NAME => { return RUSTCONV_WSTR_B6CAE64F37AA.as_ptr(); }
        TAG_PRODUCT_VERSION => { return RUSTCONV_WSTR_3E694BB9A2D4.as_ptr(); }
        TAG_FILE_DESCRIPTION => { return RUSTCONV_WSTR_93FF1A47AC17.as_ptr(); }
        TAG_FILE_VERSION => { return RUSTCONV_WSTR_815CBF3E87D0.as_ptr(); }
        TAG_ORIGINAL_FILENAME => { return RUSTCONV_WSTR_65C94AC49DBF.as_ptr(); }
        TAG_INTERNAL_NAME => { return RUSTCONV_WSTR_2C84819711BE.as_ptr(); }
        TAG_LEGAL_COPYRIGHT => { return RUSTCONV_WSTR_91A200970320.as_ptr(); }
        TAG_16BIT_DESCRIPTION => { return RUSTCONV_WSTR_36C918ADF93D.as_ptr(); }
        TAG_APPHELP_DETAILS => { return RUSTCONV_WSTR_6C65909185A4.as_ptr(); }
        TAG_LINK_URL => { return RUSTCONV_WSTR_A79AA56FBE1B.as_ptr(); }
        TAG_LINK_TEXT => { return RUSTCONV_WSTR_227C6B982831.as_ptr(); }
        TAG_APPHELP_TITLE => { return RUSTCONV_WSTR_F3DE02DAE7D2.as_ptr(); }
        TAG_APPHELP_CONTACT => { return RUSTCONV_WSTR_5898F073B17E.as_ptr(); }
        TAG_SXS_MANIFEST => { return RUSTCONV_WSTR_E2A090ED8622.as_ptr(); }
        TAG_DATA_STRING => { return RUSTCONV_WSTR_82D1C5E5AAA8.as_ptr(); }
        TAG_MSI_TRANSFORM_FILE => { return RUSTCONV_WSTR_3AEE8AFFC4B0.as_ptr(); }
        TAG_16BIT_MODULE_NAME => { return RUSTCONV_WSTR_3D9F7780DD3F.as_ptr(); }
        TAG_LAYER_DISPLAYNAME => { return RUSTCONV_WSTR_51845E867197.as_ptr(); }
        TAG_COMPILER_VERSION => { return RUSTCONV_WSTR_12D32AAF4A91.as_ptr(); }
        TAG_ACTION_TYPE => { return RUSTCONV_WSTR_900F069F555C.as_ptr(); }
        TAG_EXPORT_NAME => { return RUSTCONV_WSTR_D77710FDBACE.as_ptr(); }
        TAG_URL => { return RUSTCONV_WSTR_AB04297BDC11.as_ptr(); }
        TAG_DATABASE => { return RUSTCONV_WSTR_04152469CE43.as_ptr(); }
        TAG_LIBRARY => { return RUSTCONV_WSTR_E9F811BC0131.as_ptr(); }
        TAG_INEXCLUD => { return RUSTCONV_WSTR_DB6DFF48431C.as_ptr(); }
        TAG_SHIM => { return RUSTCONV_WSTR_DC7FC9A9E687.as_ptr(); }
        TAG_PATCH => { return RUSTCONV_WSTR_879DAA402051.as_ptr(); }
        TAG_APP => { return RUSTCONV_WSTR_8E2BC5ABC2F3.as_ptr(); }
        TAG_EXE => { return RUSTCONV_WSTR_8C7CF1E9BC62.as_ptr(); }
        TAG_MATCHING_FILE => { return RUSTCONV_WSTR_B06462BF2383.as_ptr(); }
        TAG_SHIM_REF => { return RUSTCONV_WSTR_C0EC3AAA1CB1.as_ptr(); }
        TAG_PATCH_REF => { return RUSTCONV_WSTR_52914B58A62F.as_ptr(); }
        TAG_LAYER => { return RUSTCONV_WSTR_5E9D9DEA8D61.as_ptr(); }
        TAG_FILE => { return RUSTCONV_WSTR_239E714E0983.as_ptr(); }
        TAG_APPHELP => { return RUSTCONV_WSTR_F5DD91F8FDAE.as_ptr(); }
        TAG_LINK => { return RUSTCONV_WSTR_C4BD7D9AD8D3.as_ptr(); }
        TAG_DATA => { return RUSTCONV_WSTR_1FAD449F8979.as_ptr(); }
        TAG_MSI_TRANSFORM => { return RUSTCONV_WSTR_2436CBC0DE61.as_ptr(); }
        TAG_MSI_TRANSFORM_REF => { return RUSTCONV_WSTR_986B64017E94.as_ptr(); }
        TAG_MSI_PACKAGE => { return RUSTCONV_WSTR_0B06054A863C.as_ptr(); }
        TAG_FLAG => { return RUSTCONV_WSTR_C3896EEFC410.as_ptr(); }
        TAG_MSI_CUSTOM_ACTION => { return RUSTCONV_WSTR_77C3E0CF2A18.as_ptr(); }
        TAG_FLAG_REF => { return RUSTCONV_WSTR_7B1C0D1397C8.as_ptr(); }
        TAG_ACTION => { return RUSTCONV_WSTR_4E6156A8C01F.as_ptr(); }
        TAG_LOOKUP => { return RUSTCONV_WSTR_9AA69845B279.as_ptr(); }
        TAG_CONTEXT => { return RUSTCONV_WSTR_04B8EEB408B4.as_ptr(); }
        TAG_CONTEXT_REF => { return RUSTCONV_WSTR_D3291E1A0630.as_ptr(); }
        TAG_SPC => { return RUSTCONV_WSTR_3BD4CDDA752C.as_ptr(); }
        TAG_STRINGTABLE => { return RUSTCONV_WSTR_7A16028390BB.as_ptr(); }
        TAG_INDEXES => { return RUSTCONV_WSTR_B671916D2EE2.as_ptr(); }
        TAG_INDEX => { return RUSTCONV_WSTR_08BC4D02D8CD.as_ptr(); }
        TAG_STRINGTABLE_ITEM => { return RUSTCONV_WSTR_F7B12ACC25D7.as_ptr(); }
        TAG_PATCH_BITS => { return RUSTCONV_WSTR_96504F537330.as_ptr(); }
        TAG_FILE_BITS => { return RUSTCONV_WSTR_90B53DCCB63A.as_ptr(); }
        TAG_EXE_ID => { return RUSTCONV_WSTR_17FFB8D9A3E9.as_ptr(); }
        TAG_DATA_BITS => { return RUSTCONV_WSTR_69A11BC4DD07.as_ptr(); }
        TAG_MSI_PACKAGE_ID => { return RUSTCONV_WSTR_E7D2A61F0DB4.as_ptr(); }
        TAG_DATABASE_ID => { return RUSTCONV_WSTR_0872ACAF2F2A.as_ptr(); }
        TAG_CONTEXT_PLATFORM_ID => { return RUSTCONV_WSTR_CE448C5F416D.as_ptr(); }
        TAG_CONTEXT_BRANCH_ID => { return RUSTCONV_WSTR_2D28F26CDB5C.as_ptr(); }
        TAG_FIX_ID => { return RUSTCONV_WSTR_FCCF0BD8C37D.as_ptr(); }
        TAG_APP_ID => { return RUSTCONV_WSTR_EDD6FD03C130.as_ptr(); }
        TAG_INDEX_BITS => { return RUSTCONV_WSTR_421D6A43470F.as_ptr(); }
        _ => {}
    }
    return RUSTCONV_WSTR_A69F9601B2C4.as_ptr();
}
