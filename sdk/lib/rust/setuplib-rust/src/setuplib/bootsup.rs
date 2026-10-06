use crate::setuplib::bldrsup::AddBootStoreEntry;
use crate::setuplib::bldrsup::CloseBootStore;
use crate::setuplib::filesup::CombinePaths;
use crate::setuplib::filesup::DoesFileExist_2;
use crate::setuplib::bldrsup::EnumerateBootStoreEntries;
use crate::setuplib::fsutil::FormatFileSystem;
use crate::setuplib::fsutil::FormatFileSystem_UStr;
use crate::setuplib::bootcode::FreeBootCode;
use crate::setuplib::fsrec::GetFileSystemName;
use crate::setuplib::fsrec::GetFileSystemName_UStr;
use crate::setuplib::fsutil::InstallFat32BootCode;
use crate::setuplib::partlist::IsDiskSuperFloppy2;
use crate::setuplib::setuplib::IsUnattendedSetup;
use crate::setuplib::filesup::NtPathToDiskPartComponents;
use crate::setuplib::bldrsup::OpenBootStore;
use crate::setuplib::bootcode::ReadBootCodeByHandle;
use crate::setuplib::bootcode::ReadBootCodeFromFile;
use crate::setuplib::bldrsup::SetBootStoreOptions;
use crate::setuplib::filesup::SetupCopyFile;
use crate::setuplib::devutils::pOpenDeviceEx_UStr;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_007B75529801: [u16; 12] = [
    0x0066u16, 0x0072, 0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0073, 0x0079,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_01AC0DF2395A: [u16; 12] = [
    0x0066u16, 0x0072, 0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0069, 0x006E,
    0x0069, 0x0000,
];
pub static RUSTCONV_WSTR_0B94B1551D26: [u16; 12] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0025, 0x006C,
    0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_0F012B9C99B6: [u16; 13] = [
    0x0062u16, 0x006F, 0x006F, 0x0074, 0x0073, 0x0065, 0x0063, 0x0074, 0x002E, 0x0072,
    0x006F, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_14336875ADDD: [u16; 13] = [
    0x0042u16, 0x004F, 0x004F, 0x0054, 0x0053, 0x0045, 0x0043, 0x0054, 0x002E, 0x004F,
    0x004C, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_170DB3229921: [u16; 6] = [0x004Du16, 0x0053, 0x0044, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_182039E8C7BB: [u16; 28] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0046, 0x0049,
    0x004C, 0x0045, 0x0020, 0x002F, 0x0053, 0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_24299CDCC66A: [u16; 16] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x004C, 0x006F,
    0x0067, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_25CD2C0F25D2: [u16; 20] = [
    0x005Cu16, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072, 0x005C, 0x0066, 0x0072,
    0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0073, 0x0079, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_2818D7AC989B: [u16; 6] = [0x004Eu16, 0x0054, 0x004C, 0x0044, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_28BF590FFFF8: [u16; 8] = [0x004Fu16, 0x0053, 0x0032, 0x004B, 0x0052, 0x004E, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_295882AE2BC3: [u16; 18] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x0044, 0x0065, 0x0062, 0x0075, 0x0067, 0x0029, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_29D2B212C703: [u16; 12] = [
    0x0072u16, 0x006F, 0x0073, 0x006C, 0x006F, 0x0061, 0x0064, 0x002E, 0x0065, 0x0078,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_323F4BBDA7DF: [u16; 11] = [
    0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x002E, 0x0073, 0x0079, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_33A09EA265C8: [u16; 9] = [0x0042u16, 0x004F, 0x004F, 0x0054, 0x002E, 0x0049, 0x004E, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_33FB62474A78: [u16; 8] = [0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_37F85C4F85C4: [u16; 13] = [
    0x0042u16, 0x004F, 0x004F, 0x0054, 0x0053, 0x0045, 0x0043, 0x0054, 0x002E, 0x004F,
    0x0053, 0x0032, 0x0000,
];
pub static RUSTCONV_WSTR_39B8380085DF: [u16; 14] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0025,
    0x006C, 0x0075, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_39CDE12564AA: [u16; 10] = [0x004Du16, 0x0053, 0x0044, 0x004F, 0x0053, 0x002E, 0x0043, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_3D981F870316: [u16; 10] = [0x004Du16, 0x0053, 0x0044, 0x004F, 0x0053, 0x002E, 0x0053, 0x0059, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_3D9AC6CADA36: [u16; 10] = [0x0049u16, 0x004F, 0x0053, 0x0059, 0x0053, 0x002E, 0x0043, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_446D096AE784: [u16; 21] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x0052, 0x0041, 0x004D, 0x0020, 0x0044, 0x0069, 0x0073, 0x006B, 0x0029, 0x0022,
    0x0000,
];
pub static RUSTCONV_WSTR_4A0C3CE0D885: [u16; 12] = [
    0x004Fu16, 0x0053, 0x0032, 0x0042, 0x004F, 0x004F, 0x0054, 0x002E, 0x0043, 0x004F,
    0x004D, 0x0000,
];
pub static RUSTCONV_WSTR_4F3D11B0AF9C: [u16; 17] = [
    0x0022u16, 0x004D, 0x0053, 0x002D, 0x0044, 0x004F, 0x0053, 0x002F, 0x0057, 0x0069,
    0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_4FE6EDBB76D2: [u16; 85] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0043, 0x004F,
    0x004D, 0x0031, 0x0020, 0x002F, 0x0042, 0x0041, 0x0055, 0x0044, 0x0052, 0x0041,
    0x0054, 0x0045, 0x003D, 0x0031, 0x0031, 0x0035, 0x0032, 0x0030, 0x0030, 0x0020,
    0x002F, 0x0053, 0x004F, 0x0053, 0x0020, 0x002F, 0x0072, 0x0065, 0x0064, 0x0069,
    0x0072, 0x0065, 0x0063, 0x0074, 0x003D, 0x0063, 0x006F, 0x006D, 0x0032, 0x0020,
    0x002F, 0x0072, 0x0065, 0x0064, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x0062,
    0x0061, 0x0075, 0x0064, 0x0072, 0x0061, 0x0074, 0x0065, 0x003D, 0x0031, 0x0031,
    0x0035, 0x0032, 0x0030, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_57C740ED105F: [u16; 7] = [0x0049u16, 0x0042, 0x004D, 0x004F, 0x0053, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_5EDB4DCF1AD2: [u16; 13] = [
    0x0022u16, 0x0044, 0x0052, 0x002D, 0x0044, 0x004F, 0x0053, 0x0020, 0x0033, 0x002E,
    0x0078, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_627AF3915AC4: [u16; 17] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x004B, 0x0064,
    0x0053, 0x0065, 0x0072, 0x0069, 0x0061, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_6BE1F9B0A9D1: [u16; 13] = [
    0x0042u16, 0x004F, 0x004F, 0x0054, 0x0053, 0x0045, 0x0043, 0x0054, 0x002E, 0x0044,
    0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_6C8CC80395D9: [u16; 6] = [0x004Du16, 0x0053, 0x004F, 0x0053, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_6EDDCF3F8D25: [u16; 10] = [0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_72F88E09404E: [u16; 45] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0043, 0x004F,
    0x004D, 0x0031, 0x0020, 0x002F, 0x0042, 0x0041, 0x0055, 0x0044, 0x0052, 0x0041,
    0x0054, 0x0045, 0x003D, 0x0031, 0x0031, 0x0035, 0x0032, 0x0030, 0x0030, 0x0020,
    0x002F, 0x0053, 0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_76D722070308: [u16; 19] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x0052, 0x006F, 0x0073, 0x0044, 0x0062, 0x0067, 0x0029, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_7707EC4899BC: [u16; 12] = [
    0x0044u16, 0x0045, 0x004C, 0x004C, 0x0042, 0x0049, 0x004F, 0x002E, 0x0042, 0x0049,
    0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_7BB0599F77FA: [u16; 27] = [
    0x0022u16, 0x0055, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0020, 0x004F,
    0x0070, 0x0065, 0x0072, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_81AF36117C81: [u16; 40] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0072, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_821ADD57A26F: [u16; 6] = [0x0044u16, 0x0052, 0x0044, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_8568F616184E: [u16; 11] = [
    0x0044u16, 0x0052, 0x0042, 0x0044, 0x004F, 0x0053, 0x002E, 0x0053, 0x0059, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_8CDC2448B4AB: [u16; 5] = [0x0044u16, 0x0052, 0x004D, 0x004B, 0x0000];
pub static RUSTCONV_WSTR_8D6F0929D666: [u16; 34] = [
    0x0022u16, 0x0043, 0x004F, 0x004D, 0x0050, 0x0041, 0x0051, 0x0020, 0x004D, 0x0053,
    0x002D, 0x0044, 0x004F, 0x0053, 0x0020, 0x0031, 0x002E, 0x0078, 0x0020, 0x002F,
    0x0020, 0x004D, 0x0053, 0x002D, 0x0044, 0x004F, 0x0053, 0x0020, 0x0031, 0x002E,
    0x0032, 0x0035, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_8DE000CD0DDF: [u16; 8] = [0x004Fu16, 0x0053, 0x0032, 0x0042, 0x004F, 0x004F, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_8E77B0BC7D5C: [u16; 7] = [0x0049u16, 0x004F, 0x002E, 0x0053, 0x0059, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_9A1AF075FEE2: [u16; 7] = [0x004Fu16, 0x0053, 0x0032, 0x004C, 0x0044, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_9A6F5C97B537: [u16; 16] = [
    0x0043u16, 0x003A, 0x005C, 0x0062, 0x006F, 0x006F, 0x0074, 0x0073, 0x0065, 0x0063,
    0x0074, 0x002E, 0x0072, 0x006F, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_9D4ECCA76B56: [u16; 15] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0053, 0x0063,
    0x0072, 0x0065, 0x0065, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_A33FA6D76021: [u16; 12] = [
    0x0044u16, 0x0045, 0x004C, 0x004C, 0x0052, 0x004D, 0x004B, 0x002E, 0x0042, 0x0049,
    0x004E, 0x0000,
];
pub static RUSTCONV_WSTR_A3FE07226E20: [u16; 20] = [
    0x005Cu16, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072, 0x005C, 0x0072, 0x006F,
    0x0073, 0x006C, 0x006F, 0x0061, 0x0064, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_A40D783883EA: [u16; 10] = [0x0022u16, 0x0046, 0x0072, 0x0065, 0x0065, 0x0044, 0x004F, 0x0053, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_A5CD143534FA: [u16; 8] = [0x006Du16, 0x0062, 0x0072, 0x002E, 0x006F, 0x006C, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_AA87DD449796: [u16; 55] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0043, 0x004F,
    0x004D, 0x0031, 0x0020, 0x002F, 0x0042, 0x0041, 0x0055, 0x0044, 0x0052, 0x0041,
    0x0054, 0x0045, 0x003D, 0x0031, 0x0031, 0x0035, 0x0032, 0x0030, 0x0030, 0x0020,
    0x002F, 0x0053, 0x004F, 0x0053, 0x0020, 0x002F, 0x004B, 0x0044, 0x0053, 0x0045,
    0x0052, 0x0049, 0x0041, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_AFE3EF446700: [u16; 30] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x0064, 0x005C, 0x0050,
    0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_B3DA9F747F15: [u16; 21] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x004C, 0x006F, 0x0067, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0029, 0x0022,
    0x0000,
];
pub static RUSTCONV_WSTR_B765CB4DE9F0: [u16; 11] = [
    0x0049u16, 0x0042, 0x004D, 0x0042, 0x0049, 0x004F, 0x002E, 0x0043, 0x004F, 0x004D,
    0x0000,
];
pub static RUSTCONV_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_BBB75BE65CBC: [u16; 86] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0043, 0x004F,
    0x004D, 0x0031, 0x0020, 0x002F, 0x0042, 0x0041, 0x0055, 0x0044, 0x0052, 0x0041,
    0x0054, 0x0045, 0x003D, 0x0031, 0x0031, 0x0035, 0x0032, 0x0030, 0x0030, 0x0020,
    0x002F, 0x0053, 0x004F, 0x0053, 0x0020, 0x002F, 0x0052, 0x0044, 0x0050, 0x0041,
    0x0054, 0x0048, 0x003D, 0x0072, 0x0065, 0x0061, 0x0063, 0x0074, 0x006F, 0x0073,
    0x002E, 0x0069, 0x006D, 0x0067, 0x0020, 0x002F, 0x0052, 0x0044, 0x0049, 0x004D,
    0x0041, 0x0047, 0x0045, 0x004F, 0x0046, 0x0046, 0x0053, 0x0045, 0x0054, 0x003D,
    0x0033, 0x0032, 0x0032, 0x0035, 0x0036, 0x0000,
];
pub static RUSTCONV_WSTR_C79D33CDA43F: [u16; 19] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x0053, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0029, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_CAE4D4B9BFD3: [u16; 12] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0052, 0x0061,
    0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_CB94D5A2FA8B: [u16; 15] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_CD3785CE0331: [u16; 30] = [
    0x002Fu16, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047, 0x0020, 0x002F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x0050, 0x004F, 0x0052, 0x0054, 0x003D, 0x0053, 0x0043,
    0x0052, 0x0045, 0x0045, 0x004E, 0x0020, 0x002F, 0x0053, 0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_CDE2B3AE5C9B: [u16; 10] = [0x0049u16, 0x0042, 0x004D, 0x0049, 0x004F, 0x002E, 0x0043, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_CFCA248987ED: [u16; 19] = [
    0x0072u16, 0x0061, 0x006D, 0x0064, 0x0069, 0x0073, 0x006B, 0x0028, 0x0030, 0x0029,
    0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_D2153B740B1B: [u16; 16] = [
    0x005Cu16, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072, 0x005C, 0x0066, 0x0061,
    0x0074, 0x002E, 0x0062, 0x0069, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_D42A0ABB3863: [u16; 19] = [
    0x005Cu16, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072, 0x005C, 0x0064, 0x006F,
    0x0073, 0x006D, 0x0062, 0x0072, 0x002E, 0x0062, 0x0069, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_D4D901B6B53E: [u16; 14] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0044, 0x0065,
    0x0062, 0x0075, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_D4F8A1247F54: [u16; 11] = [
    0x0049u16, 0x0042, 0x004D, 0x0044, 0x004F, 0x0053, 0x002E, 0x0043, 0x004F, 0x004D,
    0x0000,
];
pub static RUSTCONV_WSTR_D846342559FB: [u16; 11] = [
    0x004Fu16, 0x0053, 0x0032, 0x0044, 0x004F, 0x0053, 0x002E, 0x0043, 0x004F, 0x004D,
    0x0000,
];
pub static RUSTCONV_WSTR_DB083D81CBEC: [u16; 27] = [
    0x0022u16, 0x0044, 0x0065, 0x006C, 0x006C, 0x0020, 0x0052, 0x0065, 0x0061, 0x006C,
    0x002D, 0x004D, 0x006F, 0x0064, 0x0065, 0x0020, 0x004B, 0x0065, 0x0072, 0x006E,
    0x0065, 0x006C, 0x0020, 0x004F, 0x0053, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_DBA9AF0680E0: [u16; 7] = [0x0049u16, 0x0042, 0x004D, 0x0044, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_DBB0C398F223: [u16; 18] = [
    0x005Cu16, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072, 0x005C, 0x0066, 0x0061,
    0x0074, 0x0033, 0x0032, 0x002E, 0x0062, 0x0069, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_DD1FFD40389F: [u16; 12] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0045, 0x004D,
    0x0053, 0x0000,
];
pub static RUSTCONV_WSTR_DDA9DE3FD97C: [u16; 14] = [
    0x0022u16, 0x004D, 0x0053, 0x0020, 0x004F, 0x0053, 0x002F, 0x0032, 0x0020, 0x0031,
    0x002E, 0x0078, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_E121F9CA1380: [u16; 43] = [
    0x0022u16, 0x0049, 0x0042, 0x004D, 0x0020, 0x0050, 0x0043, 0x002D, 0x0044, 0x004F,
    0x0053, 0x0020, 0x006F, 0x0072, 0x0020, 0x0044, 0x0052, 0x002D, 0x0044, 0x004F,
    0x0053, 0x0020, 0x0035, 0x002E, 0x0078, 0x0020, 0x006F, 0x0072, 0x0020, 0x0049,
    0x0042, 0x004D, 0x0020, 0x004F, 0x0053, 0x002F, 0x0032, 0x0020, 0x0031, 0x002E,
    0x0030, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_E9D409C4F426: [u16; 4] = [0x0025u16, 0x0077, 0x005A, 0x0000];
pub static RUSTCONV_WSTR_EBD24329D680: [u16; 7] = [0x0043u16, 0x0050, 0x0051, 0x0044, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_F3CDEC9800CB: [u16; 8] = [0x0055u16, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_F6F6FA41222F: [u16; 11] = [
    0x0044u16, 0x0052, 0x0042, 0x0049, 0x004F, 0x0053, 0x002E, 0x0053, 0x0059, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_F7B88D7F0064: [u16; 11] = [
    0x004Fu16, 0x0053, 0x0032, 0x0042, 0x0049, 0x004F, 0x002E, 0x0043, 0x004F, 0x004D,
    0x0000,
];
pub static RUSTCONV_WSTR_F7E5103E0F57: [u16; 5] = [0x0046u16, 0x0044, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_FA503C66EA7F: [u16; 14] = [
    0x0022u16, 0x004D, 0x0053, 0x002F, 0x0049, 0x0042, 0x004D, 0x0020, 0x004F, 0x0053,
    0x002F, 0x0032, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_FDD69F3CB5F1: [u16; 26] = [
    0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0028, 0x0030, 0x0029, 0x0064, 0x0069,
    0x0073, 0x006B, 0x0028, 0x0030, 0x0029, 0x0066, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0028, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_FFCEBC1E6185: [u16; 42] = [
    0x0022u16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0028,
    0x0045, 0x006D, 0x0065, 0x0072, 0x0067, 0x0065, 0x006E, 0x0063, 0x0079, 0x0020,
    0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074,
    0x0020, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0029,
    0x0022, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert(message: *mut core::ffi::c_void, file: *mut core::ffi::c_void, line: u32, dummy: *mut i8);
}
extern "C" {
    fn DbgPrint(format: *const i8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_CB94D5A2FA8B: [u16; 15] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0000,
];

static mut DeviceFloppy: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_CB94D5A2FA8B.as_ptr() as PWSTR,
    };

pub const NTOS_OPTIONS_SIGNATURE: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"NTOS_5\0\0\0") };

#[repr(C)]
pub struct _ENUM_REACTOS_ENTRIES_DATA {
    pub i: u32,
    pub UseExistingEntry: u8,
    pub ArcPath: PCWSTR,
    pub SectionName: [u16; 80],
    pub OsName: [u16; 80],
}
pub type ENUM_REACTOS_ENTRIES_DATA = _ENUM_REACTOS_ENTRIES_DATA;
pub type PENUM_REACTOS_ENTRIES_DATA = *mut _ENUM_REACTOS_ENTRIES_DATA;

unsafe extern "C" fn TrimTrailingPathSeparators_UStr(mut UnicodeString: PUNICODE_STRING) {
    while (*UnicodeString).Length as usize >= core::mem::size_of::<u16>()
        && *(*UnicodeString).Buffer.offset(
            ((*UnicodeString).Length as usize)
                .wrapping_div(core::mem::size_of::<u16>())
                .wrapping_sub(1) as isize,
        ) as i32
            == OBJ_NAME_PATH_SEPARATOR as i32
    {
        (*UnicodeString).Length = ((*UnicodeString).Length as u64)
            .wrapping_sub(core::mem::size_of::<u16>() as u64) as u16;
    }
}

unsafe extern "C" fn CreateFreeLoaderReactOSEntries(
    mut BootStoreHandle: PVOID,
    mut ArcPath: PCWSTR,
) {
    let mut xxBootEntry: [UCHAR; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize] = [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize];
    let mut BootEntry: PBOOT_STORE_ENTRY = &raw mut xxBootEntry as PBOOT_STORE_ENTRY;
    let mut Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    let mut BootOptions: BOOT_STORE_OPTIONS = BOOT_STORE_OPTIONS {
        Timeout: 0,
        CurrentBootEntryKey: 0,
        NextBootEntryKey: 0,
    };
    (*BootEntry).Version = FreeLdr as ULONG;
    (*BootEntry).BootFilePath = core::ptr::null_mut();
    (*BootEntry).OsOptionsLength = core::mem::size_of::<NTOS_OPTIONS>() as ULONG;
    RtlCopyMemory!(
        &raw mut (*Options).Signature as *mut UCHAR as *mut core::ffi::c_void,
        (NTOS_OPTIONS_SIGNATURE).as_ptr() as *const core::ffi::c_void,
        RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as usize,
    );
    (*Options).OsLoadPath = ArcPath;
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_6EDDCF3F8D25.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_72F88E09404E.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_33FB62474A78.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_295882AE2BC3.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_72F88E09404E.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_D4D901B6B53E.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_76D722070308.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_AA87DD449796.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_627AF3915AC4.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_C79D33CDA43F.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_CD3785CE0331.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_9D4ECCA76B56.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_B3DA9F747F15.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_182039E8C7BB.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_24299CDCC66A.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_446D096AE784.as_ptr();
    (*Options).OsLoadPath = RUSTCONV_WSTR_CFCA248987ED.as_ptr();
    (*Options).OsLoadOptions = RUSTCONV_WSTR_BBB75BE65CBC.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_CAE4D4B9BFD3.as_ptr())) as _);
    (*BootEntry).FriendlyName = RUSTCONV_WSTR_FFCEBC1E6185.as_ptr();
    (*Options).OsLoadPath = ArcPath;
    (*Options).OsLoadOptions = RUSTCONV_WSTR_4FE6EDBB76D2.as_ptr();
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(RUSTCONV_WSTR_DD1FFD40389F.as_ptr())) as _);
    if IsUnattendedSetup != 0 {
        BootOptions.NextBootEntryKey = MAKESTRKEY!(RUSTCONV_WSTR_627AF3915AC4.as_ptr());
    } else {
        BootOptions.NextBootEntryKey = MAKESTRKEY!(RUSTCONV_WSTR_D4D901B6B53E.as_ptr());
    }
    if IsUnattendedSetup != 0 {
        BootOptions.Timeout = 0;
    } else {
        BootOptions.Timeout = 10;
    }
    SetBootStoreOptions(
        BootStoreHandle,
        &raw mut BootOptions,
        BOOT_OPTIONS_TIMEOUT | BOOT_OPTIONS_NEXT_BOOTENTRY_KEY,
    );
}

unsafe extern "C" fn CreateFreeLoaderIniForReactOS(
    IniPath: PCWSTR,
    ArcPath: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut BootStoreHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    Status = OpenBootStore(
        BootStoreHandle,
        IniPath,
        FreeLdr,
        BS_CreateAlways,
        BS_ReadWriteAccess,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    CreateFreeLoaderReactOSEntries(BootStoreHandle, ArcPath);

    CloseBootStore(BootStoreHandle);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn CreateFreeLoaderIniForReactOSAndBootSector(
    mut IniPath: PCWSTR,
    mut ArcPath: PCWSTR,
    mut Section: PCWSTR,
    mut Description: PCWSTR,
    mut BootPath: PCWSTR,
    mut BootSector: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut BootStoreHandle: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut xxBootEntry: [UCHAR; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<BOOTSECTOR_OPTIONS>()) as i32) as usize] = [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<BOOTSECTOR_OPTIONS>()) as i32) as usize];
    let mut BootEntry: PBOOT_STORE_ENTRY = &raw mut xxBootEntry as PBOOT_STORE_ENTRY;
    let mut Options: PBOOTSECTOR_OPTIONS = &raw mut (*BootEntry).OsOptions as PBOOTSECTOR_OPTIONS;
    let mut BootPathBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut DiskNumber: ULONG = 0;
    let mut PartitionNumber: ULONG = 0;
    let mut PathComponent: PCWSTR = core::ptr::null::<WCHAR>();

    if NtPathToDiskPartComponents(
        BootPath,
        &raw mut DiskNumber,
        &raw mut PartitionNumber,
        &raw mut PathComponent,
    ) != 0
    {
        DPRINT1!(
            b"BootPath = '%S' points to disk #%d, partition #%d, path '%S'\n\0" as *const u8 as *const i8,
            BootPath,
            DiskNumber,
            PartitionNumber,
            PathComponent,
        );

        RtlStringCchPrintfW!(
            &raw mut BootPathBuffer as NTSTRSAFE_PWSTR,
            (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
            RUSTCONV_WSTR_81AF36117C81.as_ptr(),
            DiskNumber,
            PartitionNumber,
        );
        if !PathComponent.is_null()
            && *PathComponent as core::ffi::c_int != 0
            && (*PathComponent.offset(0 as isize) as core::ffi::c_int
                != 0x005Cu16 as core::ffi::c_int
                || *PathComponent.offset(1 as isize) as core::ffi::c_int
                    != 0)
        {
            RtlStringCchCatW(
                &raw mut BootPathBuffer as NTSTRSAFE_PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                PathComponent as NTSTRSAFE_PCWSTR,
            );
        }
    } else {
        let mut Path: PCWSTR = BootPath;

        if (_wcsnicmp(
            Path as *const wchar_t,
            RUSTCONV_WSTR_CB94D5A2FA8B.as_ptr(),
            14,
        ) == 0)
            && {
                Path = Path.offset(14 as isize);
                !Path.is_null()
            }
            && iswdigit(*Path) != 0
        {
            DiskNumber = wcstoul(
                Path as *const wchar_t,
                &raw mut PathComponent as *mut *mut wchar_t,
                10,
            ) as ULONG;
            if !PathComponent.is_null()
                && *PathComponent as core::ffi::c_int != 0
                && *PathComponent as core::ffi::c_int != 0x005Cu16 as core::ffi::c_int
            {
                PathComponent = core::ptr::null::<WCHAR>();
            }

            RtlStringCchPrintfW!(
                &raw mut BootPathBuffer as NTSTRSAFE_PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                RUSTCONV_WSTR_FDD69F3CB5F1.as_ptr(),
                DiskNumber,
            );
            if !PathComponent.is_null()
                && *PathComponent as core::ffi::c_int != 0
                && (*PathComponent.offset(0 as isize) as core::ffi::c_int
                    != 0x005Cu16 as core::ffi::c_int
                    || *PathComponent.offset(1 as isize)
                        as core::ffi::c_int
                        != 0)
            {
                RtlStringCchCatW(
                    &raw mut BootPathBuffer as NTSTRSAFE_PWSTR,
                    (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                        .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                    PathComponent as NTSTRSAFE_PCWSTR,
                );
            }
        } else {
            let mut RootPartition: _UNICODE_STRING = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<WCHAR>(),
            };
            RtlInitUnicodeString(&raw mut RootPartition, BootPath);
            TrimTrailingPathSeparators_UStr(&raw mut RootPartition);

            RtlStringCchPrintfW!(
                &raw mut BootPathBuffer as NTSTRSAFE_PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as size_t),
                RUSTCONV_WSTR_E9D409C4F426.as_ptr(),
                &raw mut RootPartition,
            );

            DPRINT1!(b"Unhandled NT path '%S'\n\0" as *const u8 as *const i8, BootPath);
        }
    }

    Status = OpenBootStore(
        (&raw mut BootStoreHandle).cast(),
        IniPath,
        FreeLdr,
        BS_CreateAlways,
        BS_ReadWriteAccess,
    );
    if !(Status >= 0 as NTSTATUS) {
        return Status;
    }

    CreateFreeLoaderReactOSEntries(BootStoreHandle, ArcPath);

    (*BootEntry).Version = FreeLdr as core::ffi::c_int as ULONG;
    (*BootEntry).BootFilePath = core::ptr::null::<WCHAR>();

    (*BootEntry).OsOptionsLength = core::mem::size_of::<BOOTSECTOR_OPTIONS>() as ULONG;
    RtlCopyMemory!(
        &raw mut (*Options).Signature as *mut UCHAR as *mut core::ffi::c_void,
        BOOTSECTOR_OPTIONS_SIGNATURE.as_ptr() as *const core::ffi::c_void,
        RTL_FIELD_SIZE!(BOOTSECTOR_OPTIONS, Signature) as size_t,
    );

    (*Options).BootPath = &raw mut BootPathBuffer as *mut WCHAR as PCWSTR;
    (*Options).FileName = BootSector;

    (*BootEntry).FriendlyName = Description;
    AddBootStoreEntry(BootStoreHandle, BootEntry, (MAKESTRKEY!(Section)) as _);

    CloseBootStore(BootStoreHandle);
    return STATUS_SUCCESS;
}
unsafe extern "system" fn EnumerateReactOSEntries(
    Type: BOOT_STORE_TYPE,
    BootEntry: PBOOT_STORE_ENTRY,
    Parameter: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Data: PENUM_REACTOS_ENTRIES_DATA = Parameter as PENUM_REACTOS_ENTRIES_DATA;
    let Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    let mut SystemPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    'skip_this_entry: loop {
        if (*BootEntry).OsOptionsLength < core::mem::size_of::<NTOS_OPTIONS>() as u32
            || RtlCompareMemory(
                &raw mut (*BootEntry).OsOptions as *const core::ffi::c_void,
                NTOS_OPTIONS_SIGNATURE.as_ptr() as *const core::ffi::c_void,
                RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T,
            ) != RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature) as SIZE_T
        {
            DPRINT!(
                b"    An installation '%S' of unsupported type %lu\n\0".as_ptr() as *const i8,
                (*BootEntry).FriendlyName,
                (*BootEntry).OsOptionsLength,
            );
            break 'skip_this_entry;
        }

        if (*Options).OsLoadPath.is_null() || *(*Options).OsLoadPath == 0 {
            DPRINT1!(
                b"    A Win2k3 install '%S' without an ARC path?!\n\0".as_ptr() as *const i8,
                (*BootEntry).FriendlyName,
            );
            break 'skip_this_entry;
        }

        if _wcsicmp((*Options).OsLoadPath, (*Data).ArcPath) != 0 {
            Status = RtlStringCchPrintfW!(
                SystemPath.as_mut_ptr(),
                ARRAYSIZE!(SystemPath),
                RUSTCONV_WSTR_9896841D0A6A.as_ptr(),
                (*Data).ArcPath,
            );
            if ((NT_SUCCESS!(Status)) == 0)
                || _wcsicmp((*Options).OsLoadPath, SystemPath.as_mut_ptr())
                    != 0
            {
                break 'skip_this_entry;
            }
        }

        DPRINT!(
            b"    Found a candidate Win2k3 install '%S' with ARC path '%S'\n\0".as_ptr()
                as *const i8,
            (*BootEntry).FriendlyName,
            (*Options).OsLoadPath,
        );

        DPRINT!(
            b"EnumerateReactOSEntries: OsLoadPath: '%S'\n\0".as_ptr() as *const i8,
            (*Options).OsLoadPath,
        );

        (*Data).UseExistingEntry = (TRUE) as u8;
        RtlStringCchCopyW(
            (*Data).OsName.as_mut_ptr(),
            (ARRAYSIZE!((*Data).OsName)) as _,
            (*BootEntry).FriendlyName,
        );

        return STATUS_NO_MORE_ENTRIES;
    }

    (*Data).UseExistingEntry = (FALSE) as u8;
    if Type == FreeLdr
        && wcscmp(
            (*Data).SectionName.as_mut_ptr(),
            (*BootEntry).BootEntryKey as PWSTR,
        ) == 0
    {
        RtlStringCchPrintfW!(
            (*Data).SectionName.as_mut_ptr(),
            ARRAYSIZE!((*Data).SectionName),
            RUSTCONV_WSTR_0B94B1551D26.as_ptr(),
            (*Data).i,
        );
        RtlStringCchPrintfW!(
            (*Data).OsName.as_mut_ptr(),
            ARRAYSIZE!((*Data).OsName),
            RUSTCONV_WSTR_39B8380085DF.as_ptr(),
            (*Data).i,
        );
        (*Data).i += 1;
    }
    return STATUS_SUCCESS;
}

unsafe extern "C" fn UpdateFreeLoaderIni(
    mut IniPath: PCWSTR,
    mut ArcPath: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut BootStoreHandle: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Data: ENUM_REACTOS_ENTRIES_DATA = ENUM_REACTOS_ENTRIES_DATA {
        i: 0,
        UseExistingEntry: 0,
        ArcPath: core::ptr::null::<u16>(),
        SectionName: [0; 80],
        OsName: [0; 80],
    };
    let mut xxBootEntry: [u8; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions)
        + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize] =
        [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions)
            + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize];
    let mut BootEntry: PBOOT_STORE_ENTRY = &raw mut xxBootEntry as PBOOT_STORE_ENTRY;
    let mut Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    Status = OpenBootStore(
        (&raw mut BootStoreHandle).cast(),
        IniPath,
        FreeLdr,
        BS_OpenExisting,
        BS_ReadWriteAccess,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }
    Data.UseExistingEntry = (TRUE) as u8;
    Data.i = 1;
    Data.ArcPath = ArcPath;
    RtlStringCchCopyW(
        &raw mut Data.SectionName as *mut u16,
        (ARRAYSIZE!(Data.SectionName)) as _,
        RUSTCONV_WSTR_33FB62474A78.as_ptr(),
    );
    RtlStringCchCopyW(
        &raw mut Data.OsName as *mut u16,
        (ARRAYSIZE!(Data.OsName)) as _,
        RUSTCONV_WSTR_6EDDCF3F8D25.as_ptr(),
    );
    Status = EnumerateBootStoreEntries(
        BootStoreHandle,
        Some(EnumerateReactOSEntries),
        &raw mut Data as PVOID,
    );
    if Data.UseExistingEntry == 0 {
        (*BootEntry).Version = FreeLdr;
        (*BootEntry).BootFilePath = core::ptr::null::<u16>();
        (*BootEntry).OsOptionsLength = core::mem::size_of::<NTOS_OPTIONS>() as ULONG;
        RtlCopyMemory!(
            &raw mut (*Options).Signature as *mut u8 as *mut core::ffi::c_void,
            (NTOS_OPTIONS_SIGNATURE).as_ptr() as *const u8 as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature),
        );
        (*Options).OsLoadPath = ArcPath;
        (*BootEntry).FriendlyName = &raw mut Data.OsName as *mut u16 as PCWSTR;
        (*Options).OsLoadOptions = core::ptr::null::<u16>();
        AddBootStoreEntry(
            BootStoreHandle,
            BootEntry,
            (MAKESTRKEY!(Data.SectionName.as_ptr())) as _,
        );
    }
    CloseBootStore(BootStoreHandle);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn UpdateBootIni(
    mut IniPath: PCWSTR,
    mut EntryName: PCWSTR,
    mut EntryValue: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut BootStoreHandle: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Data: ENUM_REACTOS_ENTRIES_DATA = ENUM_REACTOS_ENTRIES_DATA {
        i: 0,
        UseExistingEntry: 0,
        ArcPath: core::ptr::null::<u16>(),
        SectionName: [0; 80],
        OsName: [0; 80],
    };
    let mut xxBootEntry: [u8; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize] = [0; (FIELD_OFFSET!(BOOT_STORE_ENTRY, OsOptions) + (core::mem::size_of::<NTOS_OPTIONS>()) as i32) as usize];
    let mut BootEntry: PBOOT_STORE_ENTRY = &raw mut xxBootEntry as PBOOT_STORE_ENTRY;
    let mut Options: PNTOS_OPTIONS = &raw mut (*BootEntry).OsOptions as PNTOS_OPTIONS;
    Status = OpenBootStore(
        (&raw mut BootStoreHandle).cast(),
        IniPath,
        NtLdr,
        BS_OpenExisting,
        BS_ReadWriteAccess,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }
    Data.UseExistingEntry = (TRUE) as u8;
    Data.ArcPath = EntryName;
    RtlStringCchCopyW(
        &raw mut Data.OsName as *mut u16,
        (ARRAYSIZE!(Data.OsName)) as _,
        RUSTCONV_WSTR_6EDDCF3F8D25.as_ptr(),
    );
    Status = EnumerateBootStoreEntries(
        BootStoreHandle,
        Some(EnumerateReactOSEntries),
        &raw mut Data as PVOID,
    );
    if Data.UseExistingEntry == 0 {
        (*BootEntry).Version = NtLdr;
        (*BootEntry).BootFilePath = core::ptr::null::<u16>();
        (*BootEntry).OsOptionsLength = core::mem::size_of::<NTOS_OPTIONS>() as u32;
        RtlCopyMemory!(
            &raw mut (*Options).Signature as *mut u8 as *mut core::ffi::c_void,
            (NTOS_OPTIONS_SIGNATURE).as_ptr() as *const core::ffi::c_void,
            RTL_FIELD_SIZE!(NTOS_OPTIONS, Signature),
        );
        (*Options).OsLoadPath = EntryName;
        (*BootEntry).FriendlyName = EntryValue;
        (*Options).OsLoadOptions = core::ptr::null::<u16>();
        AddBootStoreEntry(
            BootStoreHandle,
            BootEntry,
            (MAKESTRKEY!(0)) as _,
        );
    }
    CloseBootStore(BootStoreHandle);
    return STATUS_SUCCESS;
}

unsafe extern "C" fn IsThereAValidBootSector(mut RootPath: PCWSTR) -> BOOLEAN {
    let mut IsValid: BOOLEAN = (FALSE) as u8;
    let mut Status: NTSTATUS = 0;
    let mut RootPartition: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut BootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };

    RtlInitUnicodeString(&mut RootPartition, RootPath);
    TrimTrailingPathSeparators_UStr(&mut RootPartition);
    Status = ReadBootCodeFromFile(&mut BootSector, &mut RootPartition, SECTORSIZE);
    if !(Status >= 0) {
        return (FALSE) as u8;
    }

    IsValid = (*((BootSector.BootCode as *mut u8).add(0x1FE as usize) as *mut u16) // nolint: magic-offset (bootsup.c:544 writes `(PUCHAR)BootSector.BootCode + 0x1FE`, the boot signature)
        == 0xAA55) as BOOLEAN;
    if IsValid != 0 {
        IsValid = ((*(BootSector.BootCode as *mut u32) & 0x00FFFFFF) != 0x00000000) as BOOLEAN;
    }

    FreeBootCode(&mut BootSector);
    return IsValid;
}

unsafe extern "C" fn SaveBootSector(
    RootPath: PCWSTR,
    DstPath: PCWSTR,
    Length: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileHandle: *mut core::ffi::c_void = core::mem::zeroed();
    let mut BootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };

    RtlInitUnicodeString(&mut Name, RootPath);
    TrimTrailingPathSeparators_UStr(&mut Name);
    Status = ReadBootCodeFromFile(&mut BootSector, &mut Name, Length);
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, DstPath);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut FileHandle,
        GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_SUPERSEDE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_SEQUENTIAL_ONLY,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        FreeBootCode(&mut BootSector);
        return Status;
    }

    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        BootSector.BootCode,
        BootSector.Length,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    NtClose(FileHandle);

    FreeBootCode(&mut BootSector);
    return Status;
}

unsafe extern "C" fn InstallBootCodeToDisk(
    SrcPath: PCWSTR,
    RootPath: PCWSTR,
    InstallBootCode: PFS_INSTALL_BOOTCODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut LockStatus: NTSTATUS;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut PartitionHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, RootPath);
    TrimTrailingPathSeparators_UStr(&mut Name);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut PartitionHandle,
        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    LockStatus = NtFsControlFile(
        PartitionHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        FSCTL_LOCK_VOLUME,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(LockStatus)) == 0) {
        DPRINT1!(
            b"Unable to lock the volume before installing boot code. Status 0x%08x. Expect problems.\n\0"
                .as_ptr() as *const i8,
            LockStatus,
        );
    }

    Status = (InstallBootCode.unwrap_unchecked())(SrcPath, PartitionHandle, PartitionHandle);

    if NT_SUCCESS!(LockStatus) != 0 {
        LockStatus = NtFsControlFile(
            PartitionHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            FSCTL_DISMOUNT_VOLUME,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if ((NT_SUCCESS!(LockStatus)) == 0) {
            DPRINT1!(
                b"Unable to dismount the volume after installing boot code. Status 0x%08x. Expect problems.\n\0"
                    .as_ptr() as *const i8,
                LockStatus,
            );
        }

        LockStatus = NtFsControlFile(
            PartitionHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            FSCTL_UNLOCK_VOLUME,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        if ((NT_SUCCESS!(LockStatus)) == 0) {
            DPRINT1!(
                b"Unable to unlock the volume after installing boot code. Status 0x%08x. Expect problems.\n\0"
                    .as_ptr() as *const i8,
                LockStatus,
            );
        }
    }

    NtClose(PartitionHandle);

    Status
}

unsafe extern "C" fn InstallBootCodeToFile(
    mut SrcPath: PCWSTR,
    mut DstPath: PCWSTR,
    mut RootPath: PCWSTR,
    mut InstallBootCode: PFS_INSTALL_BOOTCODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut PartitionHandle: HANDLE = core::ptr::null_mut();
    let mut FileHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, RootPath);
    TrimTrailingPathSeparators_UStr(&mut Name);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut PartitionHandle,
        GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, DstPath);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut FileHandle,
        GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_SUPERSEDE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_SEQUENTIAL_ONLY,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtCreateFile() failed (Status %lx)\n", Status);
        NtClose(PartitionHandle);
        return Status;
    }

    Status = (InstallBootCode.unwrap_unchecked())(SrcPath, FileHandle, PartitionHandle);

    NtClose(FileHandle);
    NtClose(PartitionHandle);

    return Status;
}

unsafe extern "C" fn InstallMbrBootCode(
    mut SrcPath: PCWSTR,
    mut DstPath: HANDLE,
    mut DiskHandle: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut OrigBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };
    let mut NewBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };

    C_ASSERT!(core::mem::size_of::<PARTITION_SECTOR>() == (SECTORSIZE) as _);

    Status = ReadBootCodeByHandle(
        &mut OrigBootSector,
        DiskHandle,
        core::mem::size_of::<PARTITION_SECTOR>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, SrcPath);
    Status = ReadBootCodeFromFile(
        &mut NewBootSector,
        &mut Name,
        core::mem::size_of::<PARTITION_SECTOR>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        FreeBootCode(&mut OrigBootSector);
        return Status;
    }

    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*(NewBootSector.BootCode as *mut PARTITION_SECTOR)).Signature) as *mut _ as *mut core::ffi::c_void,
        core::ptr::addr_of!((*(OrigBootSector.BootCode as *mut PARTITION_SECTOR)).Signature) as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<PARTITION_SECTOR>() - FIELD_OFFSET!(PARTITION_SECTOR, Signature) as usize,
    );

    FreeBootCode(&mut OrigBootSector);

    FileOffset.QuadPart = 0;
    Status = NtWriteFile(
        DstPath,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        NewBootSector.BootCode,
        NewBootSector.Length,
        &mut FileOffset,
        core::ptr::null_mut(),
    );

    FreeBootCode(&mut NewBootSector);

    return Status;
}

unsafe extern "C" fn InstallMbrBootCodeToDisk(
    SystemRootPath: PCUNICODE_STRING,
    SourceRootPath: PCUNICODE_STRING,
    DestinationDevicePathBuffer: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut SourceMbrPathBuffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut DstPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    CombinePaths(
        SourceMbrPathBuffer.as_mut_ptr(),
        (ARRAYSIZE!(SourceMbrPathBuffer)) as _,
        2,
        (*SourceRootPath).Buffer,
        RUSTCONV_WSTR_D42A0ABB3863.as_ptr(),
    );

    if IsThereAValidBootSector(DestinationDevicePathBuffer) != 0 {
        CombinePaths(
            DstPath.as_mut_ptr(),
            (ARRAYSIZE!(DstPath)) as _,
            2,
            (*SystemRootPath).Buffer,
            RUSTCONV_WSTR_A5CD143534FA.as_ptr(),
        );

        DPRINT1!(
            b"Save MBR: %S ==> %S\n\0".as_ptr() as *const i8,
            DestinationDevicePathBuffer,
            DstPath.as_mut_ptr(),
        );
        Status = SaveBootSector(
            DestinationDevicePathBuffer,
            DstPath.as_mut_ptr(),
            (core::mem::size_of::<PARTITION_SECTOR>()) as ULONG,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"SaveBootSector() failed (Status %lx)\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    DPRINT1!(
        b"Install MBR bootcode: %S ==> %S\n\0".as_ptr() as *const i8,
        SourceMbrPathBuffer.as_mut_ptr(),
        DestinationDevicePathBuffer,
    );

    return InstallBootCodeToDisk(
        SourceMbrPathBuffer.as_mut_ptr(),
        DestinationDevicePathBuffer,
        Some(InstallMbrBootCode),
    );
}

unsafe extern "C" fn InstallBootloaderFiles(
    SystemRootPath: PCUNICODE_STRING,
    SourceRootPath: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SrcPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut DstPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    CombinePaths(
        SrcPath.as_mut_ptr(),
        (ARRAYSIZE!(SrcPath)) as _,
        2,
        (*SourceRootPath).Buffer,
        RUSTCONV_WSTR_25CD2C0F25D2.as_ptr(),
    );
    CombinePaths(
        DstPath.as_mut_ptr(),
        (ARRAYSIZE!(DstPath)) as _,
        2,
        (*SystemRootPath).Buffer,
        RUSTCONV_WSTR_007B75529801.as_ptr(),
    );

    DPRINT!(
        b"Copy: %S ==> %S\n\0".as_ptr() as *const i8,
        SrcPath.as_ptr(),
        DstPath.as_ptr(),
    );
    Status = SetupCopyFile(
        SrcPath.as_ptr(),
        DstPath.as_ptr(),
        (FALSE) as BOOLEAN,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"SetupCopyFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    CombinePaths(
        SrcPath.as_mut_ptr(),
        (ARRAYSIZE!(SrcPath)) as _,
        2,
        (*SourceRootPath).Buffer,
        RUSTCONV_WSTR_A3FE07226E20.as_ptr(),
    );
    CombinePaths(
        DstPath.as_mut_ptr(),
        (ARRAYSIZE!(DstPath)) as _,
        2,
        (*SystemRootPath).Buffer,
        RUSTCONV_WSTR_29D2B212C703.as_ptr(),
    );

    DPRINT!(
        b"Copy: %S ==> %S\n\0".as_ptr() as *const i8,
        SrcPath.as_ptr(),
        DstPath.as_ptr(),
    );
    Status = SetupCopyFile(
        SrcPath.as_ptr(),
        DstPath.as_ptr(),
        (FALSE) as BOOLEAN,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"SetupCopyFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn InstallFatBootcodeToPartition(
    mut SystemRootPath: PCUNICODE_STRING,
    mut SourceRootPath: PCUNICODE_STRING,
    mut DestinationArcPath: PCUNICODE_STRING,
    mut FileSystemName: PCWSTR,
) -> NTSTATUS {
    unsafe {
        let mut Status: NTSTATUS = 0;
        let mut DoesFreeLdrExist: BOOLEAN = 0;
        let mut SrcPath: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
        let mut DstPath: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];

        DPRINT!(
            b"System path: '%wZ'\n\0".as_ptr() as *const i8,
            SystemRootPath,
        );

        Status = InstallBootloaderFiles(SystemRootPath, SourceRootPath);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"InstallBootloaderFiles() failed (Status %lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            return Status;
        }

        DoesFreeLdrExist = (DoesFileExist_2(
            (*SystemRootPath).Buffer as PCWSTR,
            RUSTCONV_WSTR_01AC0DF2395A.as_ptr(),
        )) as u8;
        if DoesFreeLdrExist != 0 {
            DPRINT1!(b"Update existing 'freeldr.ini'\n\0".as_ptr() as *const i8);
            Status = UpdateFreeLoaderIni(
                (*SystemRootPath).Buffer as PCWSTR,
                (*DestinationArcPath).Buffer as PCWSTR,
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"UpdateFreeLoaderIni() failed (Status %lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                return Status;
            }
        }

        if (DoesFileExist_2(
            (*SystemRootPath).Buffer as PCWSTR,
            RUSTCONV_WSTR_2818D7AC989B.as_ptr(),
        )) as u32 == TRUE
            || (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_33A09EA265C8.as_ptr(),
            )) as u32 == TRUE
        {
            DPRINT1!(
                b"Found Microsoft Windows NT/2000/XP boot loader\n\0".as_ptr() as *const i8,
            );

            if (DoesFreeLdrExist) as u32 == FALSE {
                DPRINT1!(b"Create new 'freeldr.ini'\n\0".as_ptr() as *const i8);
                Status = CreateFreeLoaderIniForReactOS(
                    (*SystemRootPath).Buffer as PCWSTR,
                    (*DestinationArcPath).Buffer as PCWSTR,
                );
                if ((NT_SUCCESS!(Status)) == 0) {
                    DPRINT1!(
                        b"CreateFreeLoaderIniForReactOS() failed (Status %lx)\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                    return Status;
                }

                CombinePaths(
                    DstPath.as_mut_ptr(),
                    (ARRAYSIZE!(DstPath)) as _,
                    2,
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_0F012B9C99B6.as_ptr(),
                );

                if _wcsicmp(FileSystemName, RUSTCONV_WSTR_BB53EE05898F.as_ptr())
                    == 0
                {
                    CombinePaths(
                        SrcPath.as_mut_ptr(),
                        (ARRAYSIZE!(SrcPath)) as _,
                        2,
                        (*SourceRootPath).Buffer as PCWSTR,
                        RUSTCONV_WSTR_DBB0C398F223.as_ptr(),
                    );

                    DPRINT1!(
                        b"Install FAT32 bootcode: %S ==> %S\n\0".as_ptr() as *const i8,
                        SrcPath.as_ptr(),
                        DstPath.as_ptr(),
                    );
                    Status = InstallBootCodeToFile(
                        SrcPath.as_mut_ptr() as PCWSTR,
                        DstPath.as_mut_ptr() as PCWSTR,
                        (*SystemRootPath).Buffer as PCWSTR,
                        Some(
                            InstallFat32BootCode
                                as unsafe extern "C" fn(PCWSTR, HANDLE, HANDLE) -> NTSTATUS,
                        ),
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"InstallBootCodeToFile(FAT32) failed (Status %lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        return Status;
                    }
                } else {
                    CombinePaths(
                        SrcPath.as_mut_ptr(),
                        (ARRAYSIZE!(SrcPath)) as _,
                        2,
                        (*SourceRootPath).Buffer as PCWSTR,
                        RUSTCONV_WSTR_D2153B740B1B.as_ptr(),
                    );

                    DPRINT1!(
                        b"Install FAT16 bootcode: %S ==> %S\n\0".as_ptr() as *const i8,
                        SrcPath.as_ptr(),
                        DstPath.as_ptr(),
                    );
                    Status = InstallBootCodeToFile(
                        SrcPath.as_mut_ptr() as PCWSTR,
                        DstPath.as_mut_ptr() as PCWSTR,
                        (*SystemRootPath).Buffer as PCWSTR,
                        Some(
                            InstallFat16BootCode
                                as unsafe extern "C" fn(PCWSTR, HANDLE, HANDLE) -> NTSTATUS,
                        ),
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"InstallBootCodeToFile(FAT16) failed (Status %lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        return Status;
                    }
                }
            }

            DPRINT1!(b"Update 'boot.ini'\n\0".as_ptr() as *const i8);
            Status = UpdateBootIni(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_9A6F5C97B537.as_ptr(),
                RUSTCONV_WSTR_6EDDCF3F8D25.as_ptr(),
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!(
                    b"UpdateBootIni() failed (Status %lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                return Status;
            }
        } else {
            let mut Section: PCWSTR = core::ptr::null();
            let mut Description: PCWSTR = core::ptr::null();
            let mut BootSector: PCWSTR = core::ptr::null();

            if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_3D9AC6CADA36.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_39CDE12564AA.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(
                    b"Found COMPAQ MS-DOS 1.x (1.11, 1.12) / MS-DOS 1.25 boot loader\n\0"
                        .as_ptr() as *const i8,
                );

                Section = RUSTCONV_WSTR_EBD24329D680.as_ptr();
                Description = RUSTCONV_WSTR_8D6F0929D666.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_8E77B0BC7D5C.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_3D981F870316.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(
                    b"Found Microsoft DOS or Windows 9x boot loader\n\0".as_ptr() as *const i8,
                );

                Section = RUSTCONV_WSTR_170DB3229921.as_ptr();
                Description = RUSTCONV_WSTR_4F3D11B0AF9C.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_CDE2B3AE5C9B.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_B765CB4DE9F0.as_ptr(),
                )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_D4F8A1247F54.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(
                    b"Found IBM PC-DOS or DR-DOS 5.x or IBM OS/2 1.0\n\0".as_ptr() as *const i8,
                );

                Section = RUSTCONV_WSTR_DBA9AF0680E0.as_ptr();
                Description = RUSTCONV_WSTR_E121F9CA1380.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_F6F6FA41222F.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_8568F616184E.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(b"Found DR-DOS 3.x\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_821ADD57A26F.as_ptr();
                Description = RUSTCONV_WSTR_5EDB4DCF1AD2.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_7707EC4899BC.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_A33FA6D76021.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(b"Found Dell Real-Mode Kernel OS\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_8CDC2448B4AB.as_ptr();
                Description = RUSTCONV_WSTR_DB083D81CBEC.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_4A0C3CE0D885.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_F7B88D7F0064.as_ptr(),
                )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_D846342559FB.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(b"Found MS OS/2 1.x\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_6C8CC80395D9.as_ptr();
                Description = RUSTCONV_WSTR_DDA9DE3FD97C.as_ptr();
                BootSector = RUSTCONV_WSTR_37F85C4F85C4.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_8DE000CD0DDF.as_ptr(),
            )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_9A1AF075FEE2.as_ptr(),
                )) as u32 == TRUE
                || (DoesFileExist_2(
                    (*SystemRootPath).Buffer as PCWSTR,
                    RUSTCONV_WSTR_28BF590FFFF8.as_ptr(),
                )) as u32 == TRUE
            {
                DPRINT1!(b"Found MS/IBM OS/2\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_57C740ED105F.as_ptr();
                Description = RUSTCONV_WSTR_FA503C66EA7F.as_ptr();
                BootSector = RUSTCONV_WSTR_37F85C4F85C4.as_ptr();
            } else if (DoesFileExist_2(
                (*SystemRootPath).Buffer as PCWSTR,
                RUSTCONV_WSTR_323F4BBDA7DF.as_ptr(),
            )) as u32 == TRUE
            {
                DPRINT1!(b"Found FreeDOS boot loader\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_F7E5103E0F57.as_ptr();
                Description = RUSTCONV_WSTR_A40D783883EA.as_ptr();
                BootSector = RUSTCONV_WSTR_6BE1F9B0A9D1.as_ptr();
            } else {
                DPRINT1!(b"No or unknown boot loader found\n\0".as_ptr() as *const i8);

                Section = RUSTCONV_WSTR_F3CDEC9800CB.as_ptr();
                Description = RUSTCONV_WSTR_7BB0599F77FA.as_ptr();
                BootSector = RUSTCONV_WSTR_14336875ADDD.as_ptr();
            }

            if (DoesFreeLdrExist) as u32 == FALSE {
                DPRINT1!(b"Create new 'freeldr.ini'\n\0".as_ptr() as *const i8);

                if IsThereAValidBootSector((*SystemRootPath).Buffer as PCWSTR) != 0 {
                    Status = CreateFreeLoaderIniForReactOSAndBootSector(
                        (*SystemRootPath).Buffer as PCWSTR,
                        (*DestinationArcPath).Buffer as PCWSTR,
                        Section,
                        Description,
                        (*SystemRootPath).Buffer as PCWSTR,
                        BootSector,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"CreateFreeLoaderIniForReactOSAndBootSector() failed (Status %lx)\n\0"
                                .as_ptr() as *const i8,
                            Status,
                        );
                        return Status;
                    }

                    CombinePaths(
                        DstPath.as_mut_ptr(),
                        (ARRAYSIZE!(DstPath)) as _,
                        2,
                        (*SystemRootPath).Buffer as PCWSTR,
                        BootSector,
                    );

                    DPRINT1!(
                        b"Save bootsector: %S ==> %S\n\0".as_ptr() as *const i8,
                        (*SystemRootPath).Buffer,
                        DstPath.as_ptr(),
                    );
                    Status = SaveBootSector(
                        (*SystemRootPath).Buffer as PCWSTR,
                        DstPath.as_mut_ptr() as PCWSTR,
                        SECTORSIZE,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"SaveBootSector() failed (Status %lx)\n\0".as_ptr() as *const i8,
                            Status,
                        );
                        return Status;
                    }
                } else {
                    Status = CreateFreeLoaderIniForReactOS(
                        (*SystemRootPath).Buffer as PCWSTR,
                        (*DestinationArcPath).Buffer as PCWSTR,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"CreateFreeLoaderIniForReactOS() failed (Status %lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        return Status;
                    }
                }

                if _wcsicmp(FileSystemName, RUSTCONV_WSTR_BB53EE05898F.as_ptr())
                    == 0
                {
                    CombinePaths(
                        SrcPath.as_mut_ptr(),
                        (ARRAYSIZE!(SrcPath)) as _,
                        2,
                        (*SourceRootPath).Buffer as PCWSTR,
                        RUSTCONV_WSTR_DBB0C398F223.as_ptr(),
                    );

                    DPRINT1!(
                        b"Install FAT32 bootcode: %S ==> %S\n\0".as_ptr() as *const i8,
                        SrcPath.as_ptr(),
                        (*SystemRootPath).Buffer,
                    );
                    Status = InstallBootCodeToDisk(
                        SrcPath.as_mut_ptr() as PCWSTR,
                        (*SystemRootPath).Buffer as PCWSTR,
                        Some(
                            InstallFat32BootCode
                                as unsafe extern "C" fn(PCWSTR, HANDLE, HANDLE) -> NTSTATUS,
                        ),
                    );
                    DPRINT1!(b"Status: 0x%08X\n\0".as_ptr() as *const i8, Status);
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"InstallBootCodeToDisk(FAT32) failed (Status %lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        return Status;
                    }
                } else {
                    CombinePaths(
                        SrcPath.as_mut_ptr(),
                        (ARRAYSIZE!(SrcPath)) as _,
                        2,
                        (*SourceRootPath).Buffer as PCWSTR,
                        RUSTCONV_WSTR_D2153B740B1B.as_ptr(),
                    );

                    DPRINT1!(
                        b"Install FAT16 bootcode: %S ==> %S\n\0".as_ptr() as *const i8,
                        SrcPath.as_ptr(),
                        (*SystemRootPath).Buffer,
                    );
                    Status = InstallBootCodeToDisk(
                        SrcPath.as_mut_ptr() as PCWSTR,
                        (*SystemRootPath).Buffer as PCWSTR,
                        Some(
                            InstallFat16BootCode
                                as unsafe extern "C" fn(PCWSTR, HANDLE, HANDLE) -> NTSTATUS,
                        ),
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        DPRINT1!(
                            b"InstallBootCodeToDisk(FAT16) failed (Status %lx)\n\0".as_ptr()
                                as *const i8,
                            Status,
                        );
                        return Status;
                    }
                }
            }
        }

        return STATUS_SUCCESS;
    }
}

unsafe extern "C" fn InstallVBRToPartition(
    mut SystemRootPath: PCUNICODE_STRING,
    mut SourceRootPath: PCUNICODE_STRING,
    mut DestinationArcPath: PCUNICODE_STRING,
    mut FileSystemName: PCWSTR,
) -> NTSTATUS {
    if _wcsicmp(FileSystemName, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0
        || _wcsicmp(FileSystemName, RUSTCONV_WSTR_BB53EE05898F.as_ptr()) == 0
    {
        return InstallFatBootcodeToPartition(
            SystemRootPath,
            SourceRootPath,
            DestinationArcPath,
            FileSystemName,
        );
    } else {
        DPRINT1!(
            b"Unsupported file system '%S' for VBR install\n\0".as_ptr() as *const i8,
            FileSystemName,
        );
    }
    return STATUS_NOT_SUPPORTED;
}

unsafe extern "C" fn InstallBootManagerAndBootEntriesWorker(
    mut ArchType: ARCHITECTURE_TYPE,
    mut SystemRootPath: PCUNICODE_STRING,
    mut DiskNumber: ULONG,
    mut DiskStyle: PARTITION_STYLE,
    mut IsSuperFloppy: BOOLEAN,
    mut FileSystem: PCWSTR,
    mut SourceRootPath: PCUNICODE_STRING,
    mut DestinationArcPath: PCUNICODE_STRING,
    mut Options: ULONG_PTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IsBIOS: BOOLEAN = (ArchType as ::core::ffi::c_uint
        == ARCH_PcAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || ArchType as ::core::ffi::c_uint
            == ARCH_NEC98x86 as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int as BOOLEAN;
    let mut InstallType: UCHAR = (Options & 0x03 as ULONG_PTR) as UCHAR;
    if IsBIOS == 0 {
        return STATUS_NOT_SUPPORTED;
    }
    if InstallType as ::core::ffi::c_int <= 1 as ::core::ffi::c_int {
        Status = InstallVBRToPartition(
            SystemRootPath,
            SourceRootPath,
            DestinationArcPath,
            FileSystem,
        );
        if !(Status >= 0 as NTSTATUS) {
            DPRINT1!(
                b"InstallVBRToPartition() failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                Status
            );
            return (ERROR_WRITE_BOOT) as i32;
        }
        if InstallType as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && DiskStyle as ::core::ffi::c_uint
                == PARTITION_STYLE_MBR as ::core::ffi::c_int as ::core::ffi::c_uint
            && IsSuperFloppy == 0
        {
            let mut SystemDiskPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            RtlStringCchPrintfW!(
                &raw mut SystemDiskPath as NTSTRSAFE_PWSTR,
                (::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t),
                RUSTCONV_WSTR_AFE3EF446700.as_ptr(),
                DiskNumber,
            );
            Status = InstallMbrBootCodeToDisk(
                SystemRootPath,
                SourceRootPath,
                &raw mut SystemDiskPath as *mut WCHAR as PCWSTR,
            );
            if !(Status >= 0 as NTSTATUS) {
                DPRINT1!(
                    b"InstallMbrBootCodeToDisk() failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                    Status
                );
                return (ERROR_INSTALL_BOOTCODE) as i32;
            }
        }
    } else if InstallType as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        let mut SrcPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        if _wcsicmp(
            FileSystem as *const wchar_t,
            RUSTCONV_WSTR_A953DDDD78B1.as_ptr() as *const wchar_t,
        ) != 0 as ::core::ffi::c_int
        {
            return STATUS_NOT_SUPPORTED;
        }
        Status = InstallBootloaderFiles(SystemRootPath, SourceRootPath);
        if !(Status >= 0 as NTSTATUS) {
            DPRINT1!(
                b"InstallBootloaderFiles() failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                Status
            );
            return Status;
        }
        DPRINT!(b"Create new 'freeldr.ini'\n\0" as *const u8 as PCSTR);
        Status = CreateFreeLoaderIniForReactOS(
            (*SystemRootPath).Buffer as PCWSTR,
            (*DestinationArcPath).Buffer as PCWSTR,
        );
        if !(Status >= 0 as NTSTATUS) {
            DPRINT1!(
                b"CreateFreeLoaderIniForReactOS() failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                Status
            );
            return Status;
        }
        CombinePaths(
            &raw mut SrcPath as PWSTR,
            ((::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as SIZE_T)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as SIZE_T)) as _,
            2 as ULONG,
            (*SourceRootPath).Buffer,
            RUSTCONV_WSTR_D2153B740B1B.as_ptr(),
        );
        DPRINT1!(
            b"Install FAT12 bootcode: %S ==> %S\n\0" as *const u8 as PCSTR,
            &raw mut SrcPath as *mut WCHAR,
            (*SystemRootPath).Buffer
        );
        Status = InstallBootCodeToDisk(
            &raw mut SrcPath as *mut WCHAR as PCWSTR,
            (*SystemRootPath).Buffer as PCWSTR,
            Some(InstallFat12BootCode as unsafe extern "C" fn(PCWSTR, HANDLE, HANDLE) -> NTSTATUS),
        );
        if !(Status >= 0 as NTSTATUS) {
            DPRINT1!(
                b"InstallBootCodeToDisk(FAT12) failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
                Status
            );
            return Status;
        }
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceInfo_UStr(
    DeviceName: PCUNICODE_STRING,
    mut DeviceHandle: HANDLE,
    DeviceInfo: PFILE_FS_DEVICE_INFORMATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if !DeviceName.is_null() && !DeviceHandle.is_null() {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    if !DeviceName.is_null() {
        Status = pOpenDeviceEx_UStr(
            DeviceName,
            &mut DeviceHandle,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"Cannot open device '%wZ' (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                DeviceName,
                Status,
            );
            return Status;
        }
    }

    Status = NtQueryVolumeInformationFile(
        DeviceHandle,
        &mut IoStatusBlock,
        (DeviceInfo).cast(),
        (core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>()) as ULONG,
        FileFsDeviceInformation,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"FileFsDeviceInformation failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    }

    if !DeviceName.is_null() {
        NtClose(DeviceHandle);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceInfo(
    DeviceName: PCWSTR,
    DeviceHandle: HANDLE,
    DeviceInfo: PFILE_FS_DEVICE_INFORMATION,
) -> NTSTATUS {
    let mut DeviceNameU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if !DeviceName.is_null() && !DeviceHandle.is_null() {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    if !DeviceName.is_null() {
        RtlInitUnicodeString(&mut DeviceNameU, DeviceName);
    }

    return GetDeviceInfo_UStr(
        if !DeviceName.is_null() {
            &mut DeviceNameU as PCUNICODE_STRING
        } else {
            core::ptr::null_mut()
        },
        if !DeviceName.is_null() {
            core::ptr::null_mut()
        } else {
            DeviceHandle
        },
        DeviceInfo,
    );
}

#[no_mangle]
pub unsafe extern "system" fn InstallBootManagerAndBootEntries(
    ArchType: ARCHITECTURE_TYPE,
    SystemRootPath: PCUNICODE_STRING,
    SourceRootPath: PCUNICODE_STRING,
    DestinationArcPath: PCUNICODE_STRING,
    Options: usize,
) -> i32 {
    let mut Status: i32 = 0;
    let mut DeviceHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();
    let mut DiskNumber: u32 = 0;
    let mut PartitionStyle: PARTITION_STYLE = PARTITION_STYLE_MBR;
    let mut IsSuperFloppy: u8 = 0;
    let mut FileSystem: [u16; (MAX_PATH + 1) as _] = [0; (MAX_PATH + 1) as _];

    let mut RootPartition: _UNICODE_STRING = *SystemRootPath;
    TrimTrailingPathSeparators_UStr(&raw mut RootPartition);

    Status = pOpenDeviceEx_UStr(
        &raw mut RootPartition as PCUNICODE_STRING,
        &raw mut DeviceHandle,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) Cannot open %wZ for bootloader installation (Status 0x%08lx)\n\0" as *const u8 as *const i8,
            b"bootsup.c\0" as *const u8 as *const i8,
            1486 as i32,
            &raw mut RootPartition,
            Status,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"bootsup.c\0" as *const u8 as *const i8,
                1486 as i32,
            );
        }
        return Status;
    }

    // C: every failure below is `goto Quit`, which closes the device and returns Status.
    'quit: {
        Status = GetFileSystemName_UStr(
            core::ptr::null_mut::<_UNICODE_STRING>(),
            DeviceHandle,
            FileSystem.as_mut_ptr(),
            (core::mem::size_of::<[u16; (MAX_PATH + 1) as _]>()) as SIZE_T,
        );
        if !(Status >= 0) || *FileSystem.as_ptr() == 0 {
            if DbgPrint(
                b"(%s:%d) GetFileSystemName() failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
                b"bootsup.c\0" as *const u8 as *const i8,
                1495 as i32,
                Status,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1495 as i32,
                );
            }
            break 'quit;
        }

        Status = GetDeviceInfo_UStr(
            core::ptr::null_mut::<_UNICODE_STRING>(),
            DeviceHandle,
            &raw mut DeviceInfo,
        );
        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) FileFsDeviceInformation failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
                b"bootsup.c\0" as *const u8 as *const i8,
                1503 as i32,
                Status,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1503 as i32,
                );
            }
            break 'quit;
        }

        if DeviceInfo.DeviceType != FILE_DEVICE_DISK {
            if DbgPrint(
                b"(%s:%d) Invalid volume; device type %lu\n\0" as *const u8 as *const i8,
                b"bootsup.c\0" as *const u8 as *const i8,
                1511 as i32,
                DeviceInfo.DeviceType,
            ) != 0 {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1511 as i32,
                );
            }
            Status = STATUS_INVALID_DEVICE_REQUEST;
            break 'quit;
        }

        if (DeviceInfo.Characteristics & FILE_FLOPPY_DISKETTE) != 0 {
            DiskNumber = ULONG_MAX;
            PartitionStyle = PARTITION_STYLE_MBR;
            IsSuperFloppy = (TRUE) as u8;
        } else {
            let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut DeviceNumber: STORAGE_DEVICE_NUMBER = core::mem::zeroed();
            let mut DiskGeoEx: DISK_GEOMETRY_EX_INTERNAL = core::mem::zeroed();
            let mut PartitionInfo: PARTITION_INFORMATION = core::mem::zeroed();

            let _: *const _DISK_GEOMETRY_EX_INTERNAL = &DiskGeoEx as *const DISK_GEOMETRY_EX_INTERNAL as *const _DISK_GEOMETRY_EX_INTERNAL;
            let _: PDISK_GEOMETRY_EX_INTERNAL = &mut DiskGeoEx as *mut DISK_GEOMETRY_EX_INTERNAL as PDISK_GEOMETRY_EX_INTERNAL;

            Status = NtDeviceIoControlFile(
                DeviceHandle,
                core::ptr::null_mut::<core::ffi::c_void>(),
                None,
                core::ptr::null_mut::<core::ffi::c_void>(),
                &raw mut IoStatusBlock,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                core::ptr::null_mut::<core::ffi::c_void>(),
                0,
                &raw mut DeviceNumber as *mut core::ffi::c_void,
                core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
            );
            if !(Status >= 0) {
                break 'quit;
            }

            if !(DeviceNumber.DeviceType == DeviceInfo.DeviceType) {
                RtlAssert(
                    b"DeviceNumber.DeviceType == DeviceInfo.DeviceType\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                    b"bootsup.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                    1552 as u32,
                    core::ptr::null_mut::<i8>(),
                );
            }

            if DeviceNumber.DeviceNumber == ULONG_MAX {
                if DbgPrint(
                    b"(%s:%d) Invalid disk number reported, bail out\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1555 as i32,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                        b"bootsup.c\0" as *const u8 as *const i8,
                        1555 as i32,
                    );
                }
                Status = STATUS_NOT_FOUND;
                break 'quit;
            }

            Status = NtDeviceIoControlFile(
                DeviceHandle,
                core::ptr::null_mut::<core::ffi::c_void>(),
                None,
                core::ptr::null_mut::<core::ffi::c_void>(),
                &raw mut IoStatusBlock,
                IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                core::ptr::null_mut::<core::ffi::c_void>(),
                0,
                &raw mut DiskGeoEx as *mut core::ffi::c_void,
                core::mem::size_of::<DISK_GEOMETRY_EX_INTERNAL>() as u32,
            );
            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) IOCTL_DISK_GET_DRIVE_GEOMETRY_EX failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1571 as i32,
                    Status,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                        b"bootsup.c\0" as *const u8 as *const i8,
                        1571 as i32,
                    );
                }
                break 'quit;
            }

            Status = NtDeviceIoControlFile(
                DeviceHandle,
                core::ptr::null_mut::<core::ffi::c_void>(),
                None,
                core::ptr::null_mut::<core::ffi::c_void>(),
                &raw mut IoStatusBlock,
                IOCTL_DISK_GET_PARTITION_INFO,
                core::ptr::null_mut::<core::ffi::c_void>(),
                0,
                &raw mut PartitionInfo as *mut core::ffi::c_void,
                core::mem::size_of::<PARTITION_INFORMATION>() as u32,
            );
            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) IOCTL_DISK_GET_PARTITION_INFO failed (Status 0x%08lx)\n\0" as *const u8 as *const i8,
                    b"bootsup.c\0" as *const u8 as *const i8,
                    1593 as i32,
                    Status,
                ) != 0 {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                        b"bootsup.c\0" as *const u8 as *const i8,
                        1593 as i32,
                    );
                }
                break 'quit;
            }

            DiskNumber = DeviceNumber.DeviceNumber;
            PartitionStyle = DiskGeoEx.Partition.PartitionStyle;
            IsSuperFloppy = (IsDiskSuperFloppy2(
                &raw mut DiskGeoEx.Partition,
                &raw mut DiskGeoEx.DiskSize.QuadPart as PULONGLONG as *const u64,
                &raw mut PartitionInfo,
            )) as u8;
        }

        Status = InstallBootManagerAndBootEntriesWorker(
            ArchType,
            SystemRootPath,
            DiskNumber,
            PartitionStyle,
            IsSuperFloppy,
            FileSystem.as_ptr(),
            SourceRootPath,
            DestinationArcPath,
            (Options) as ULONG_PTR,
        );
    }

    NtClose(DeviceHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn InstallBootcodeToRemovable(
    ArchType: ARCHITECTURE_TYPE,
    RemovableRootPath: PCUNICODE_STRING,
    SourceRootPath: PCUNICODE_STRING,
    DestinationArcPath: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceInfo: FILE_FS_DEVICE_INFORMATION = FILE_FS_DEVICE_INFORMATION {
        DeviceType: 0,
        Characteristics: 0,
    };
    let mut FileSystemName: PCWSTR;
    let mut IsFloppy: BOOLEAN;

    let mut RootDrive: _UNICODE_STRING = *RemovableRootPath;
    TrimTrailingPathSeparators_UStr(&mut RootDrive);

    if ((DoesDirExist!(core::ptr::null_mut(), (*RemovableRootPath).Buffer)) == 0) {
        return STATUS_DEVICE_NOT_READY;
    }

    Status = GetDeviceInfo_UStr(&mut RootDrive, core::ptr::null_mut(), &mut DeviceInfo);
    if ((NT_SUCCESS!(Status)) == 0) {
        static mut DeviceFloppy: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: RUSTCONV_WSTR_CB94D5A2FA8B.as_ptr() as *mut u16,
        };

        DPRINT1!(
            b"FileFsDeviceInformation failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );

        if ((RtlPrefixUnicodeString(&DeviceFloppy, &RootDrive, (TRUE) as BOOLEAN)) == 0) {
            return Status;
        }

        DeviceInfo.DeviceType = FILE_DEVICE_DISK;
        DeviceInfo.Characteristics = FILE_REMOVABLE_MEDIA | FILE_FLOPPY_DISKETTE;
    }

    if DeviceInfo.DeviceType != FILE_DEVICE_DISK {
        DPRINT1!(
            b"Invalid volume; device type %lu\n\0".as_ptr() as *const i8,
            DeviceInfo.DeviceType,
        );
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (((DeviceInfo.Characteristics & FILE_REMOVABLE_MEDIA)) == 0) {
        DPRINT1!(b"Device is NOT removable!\n\0".as_ptr() as *const i8);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    IsFloppy = ((DeviceInfo.Characteristics & FILE_FLOPPY_DISKETTE) != 0) as u8;

    FileSystemName = if IsFloppy != 0 {
        RUSTCONV_WSTR_A953DDDD78B1.as_ptr()
    } else {
        RUSTCONV_WSTR_BB53EE05898F.as_ptr()
    };

    Status = FormatFileSystem_UStr(
        &mut RootDrive,
        FileSystemName,
        if IsFloppy != 0 {
            FMIFS_FLOPPY
        } else {
            FMIFS_REMOVABLE
        },
        core::ptr::null_mut(),
        (TRUE) as u8,
        0,
        None,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        if Status == STATUS_NOT_SUPPORTED {
            DPRINT1!(
                b"%s FS non-existent on this system!\n\0".as_ptr() as *const i8,
                FileSystemName,
            );
        } else {
            DPRINT1!(
                b"FormatFileSystem(%s) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                FileSystemName,
                Status,
            );
        }
        return Status;
    }

    Status = InstallBootManagerAndBootEntries(
        ArchType,
        RemovableRootPath,
        SourceRootPath,
        DestinationArcPath,
        2,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"InstallBootManagerAndBootEntries() failed (Status 0x%08lx)\n\0".as_ptr()
                as *const i8,
            Status,
        );
    }
    Status
}
