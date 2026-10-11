use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut conv_ANGLE: [conv_t; 6] = [
    conv_t {
        unit: IDS_ANGLE_ARCMINUTES as DWORD,
        formula_from: b"$/60\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*60\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ANGLE_ARCSECONDS as DWORD,
        formula_from: b"$/3600\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*3600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ANGLE_DEGREES as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ANGLE_GRADIANS as DWORD,
        formula_from: b"$*0.9\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.9\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ANGLE_RADIANS as DWORD,
        formula_from: b"$/P*180\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*P/180\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_AREA: [conv_t; 27] = [
    conv_t {
        unit: IDS_AREA_ACRES as DWORD,
        formula_from: b"$*4046,8564224\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4046,8564224\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_ACRES_US as DWORD,
        formula_from: b"$*627264/154.99969\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/627264*154.99969\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_ACRES_SCOTS as DWORD,
        formula_from: b"$*5000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/5000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_ARES as DWORD,
        formula_from: b"$*100\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/100\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_CHOU as DWORD,
        formula_from: b"$*10800000/1089\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1089/10800000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_DANBO as DWORD,
        formula_from: b"$*991.74\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/991.74\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_HECTARES as DWORD,
        formula_from: b"$*10000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/10000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_JEONGBO as DWORD,
        formula_from: b"$*9917.4\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9917.4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_MU as DWORD,
        formula_from: b"$*(2000/3)\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/(2000/3)\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_PYEONG as DWORD,
        formula_from: b"$*400/121\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*121/400\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_RAI as DWORD,
        formula_from: b"$*1600\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SE as DWORD,
        formula_from: b"$*108000/1089\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1089/108000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_CENTIMETERS as DWORD,
        formula_from: b"$*0,0001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0,0001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_FATHOMS as DWORD,
        formula_from: b"$*3.34450944\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3.34450944\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_FATHOMS_HUNGARY as DWORD,
        formula_from: b"$*3.59665080366244\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3.59665080366244\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_FEET as DWORD,
        formula_from: b"$*0,09290304\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0,09290304\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_INCHES as DWORD,
        formula_from: b"$*0,00064516\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.00064516\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_KILOMETERS as DWORD,
        formula_from: b"$*1000000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_METER as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_MILES as DWORD,
        formula_from: b"$*2589988.110336\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/2589988.110336\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_MILLIMETERS as DWORD,
        formula_from: b"$*1000000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_SHAKU as DWORD,
        formula_from: b"$*100/1089\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1089*100\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_VA as DWORD,
        formula_from: b"$*4\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_SQUARE_YARD as DWORD,
        formula_from: b"$*0,83612736\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0,83612736\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_TAN as DWORD,
        formula_from: b"$*1080000/1089\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1089/1080000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_AREA_TSUBO as DWORD,
        formula_from: b"$*1188/1089\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1089/1188\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_CONSUMPTION: [conv_t; 5] = [
    conv_t {
        unit: IDS_CONSUMPTION_KM_PER_L as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CONSUMPTION_L_PER_100_KM as DWORD,
        formula_from: b"100/$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"100/$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CONSUMPTION_MILES_GALLON_UK as DWORD,
        formula_from: b"$*1.609344/4.54609\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.609344*4.54609\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CONSUMPTION_MILES_GALLON_US as DWORD,
        formula_from: b"$*1.609344/3.785411784\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.609344*3.785411784\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_CURRENCY: [conv_t; 21] = [
    conv_t {
        unit: IDS_CURRENCY_AUSTRIAN_SCHILLING as DWORD,
        formula_from: b"$/13,7603\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*13,7603\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_BELGIAN_FRANC as DWORD,
        formula_from: b"$/40,3399\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*40,3399\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_CYPRIOT_POUND as DWORD,
        formula_from: b"$/0,585274\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*0,585274\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_DEUTSCHE_MARK as DWORD,
        formula_from: b"$/1,95583\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1,95583\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_DUTCH_GUILDER as DWORD,
        formula_from: b"$/2,20371\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*2,20371\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_ESTONIAN_KROON as DWORD,
        formula_from: b"$/15,6466\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*15,6466\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_EURO as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_FINNISH_MARKKA as DWORD,
        formula_from: b"$/5,94573\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*5,94573\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_FRENCH_FRANC as DWORD,
        formula_from: b"$/6,55957\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*6,55957\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_GREEK_DRACHMA as DWORD,
        formula_from: b"$/340,751\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*340,751\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_IRISH_POUND as DWORD,
        formula_from: b"$/0,787564\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*0,787564\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_ITALIAN_LIRA as DWORD,
        formula_from: b"$/1936.27\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1936.27\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_LATVIAN_LATS as DWORD,
        formula_from: b"$/0,7028\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*0,7028\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_LITHUANIAN_LITAS as DWORD,
        formula_from: b"$/3,45280\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*3,45280\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_LUXEMBOURG_FRANC as DWORD,
        formula_from: b"$/40,3399\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*40,3399\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_MALTESE_LIRA as DWORD,
        formula_from: b"$/0.42930\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*0.42930\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_PORTUGUESE_ESCUDO as DWORD,
        formula_from: b"$/200,482\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*200,482\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_SLOVAK_KORUNA as DWORD,
        formula_from: b"$/30,126\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*30,126\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_SLOVENIAN_TOLAR as DWORD,
        formula_from: b"$/239,640\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*239,640\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_CURRENCY_SPANISH_PESETA as DWORD,
        formula_from: b"$/166,386\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*166,386\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_ENERGY: [conv_t; 13] = [
    conv_t {
        unit: IDS_ENERGY_15_C_CALORIES as DWORD,
        formula_from: b"$*4.1855\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.1855\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_BTUS as DWORD,
        formula_from: b"$*1055.056\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1055.056\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_ERGS as DWORD,
        formula_from: b"$*.0000001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/.0000001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_EVS as DWORD,
        formula_from: b"$*1.60217653/1X19\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.60217653*1X19\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_FOOT_POUNDS as DWORD,
        formula_from: b"$*1.3558179483314004\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.3558179483314004\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_IT_CALORIES as DWORD,
        formula_from: b"$*4.1868\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.1868\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_IT_KILOCALORIES as DWORD,
        formula_from: b"$*4186.8\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4186.8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_JOULES as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_KILOJOULES as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_KILOWATT_HOURS as DWORD,
        formula_from: b"$*3600\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_NUTRITION_CALORIES as DWORD,
        formula_from: b"$*4.182\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.182\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_ENERGY_TH_CALORIES as DWORD,
        formula_from: b"$*4.184\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.184\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_LENGTH: [conv_t; 42] = [
    conv_t {
        unit: IDS_LENGTH_ANGSTROMS as DWORD,
        formula_from: b"$/1X10\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1X10\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_ASTRONOMICAL_UNITS as DWORD,
        formula_from: b"$*149598000000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/149598000000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_BARLEYCORNS as DWORD,
        formula_from: b"$*0.9144/108\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.9144*108\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_CENTIMETERS as DWORD,
        formula_from: b"$/100\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*100\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_CHAINS_UK as DWORD,
        formula_from: b"$*20.1168\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/20.1168\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_CHI as DWORD,
        formula_from: b"$/3\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_CHOU as DWORD,
        formula_from: b"$*3600/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33/3600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_CUN as DWORD,
        formula_from: b"$/30\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*30\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_FATHOMS as DWORD,
        formula_from: b"$*1.8288\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.8288\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_FATHOMS_HUNGARY as DWORD,
        formula_from: b"$*1.8964838\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.8964838\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_FEET as DWORD,
        formula_from: b"$*0.3048\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.3048\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_FURLONGS as DWORD,
        formula_from: b"$*201.168\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/201.168\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_HANDS as DWORD,
        formula_from: b"$*0,1016\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0,1016\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_HUNH as DWORD,
        formula_from: b"$*9.144/3456\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9.144*3456\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_INCHES as DWORD,
        formula_from: b"$*0.0254\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.0254\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_KABIET as DWORD,
        formula_from: b"$*9.144/1728\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9.144*1728\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_KEN as DWORD,
        formula_from: b"$*60/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33/60\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_KEUB as DWORD,
        formula_from: b"$*9.144/36\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9.144*36\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_KILOMETERS as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_LIGHT_YEARS as DWORD,
        formula_from: b"$*9460730472580800\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9460730472580800\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_LINKS_UK as DWORD,
        formula_from: b"$*0.201168\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.201168\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_METERS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_MICRONS as DWORD,
        formula_from: b"$*0.000001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.000001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_MILES as DWORD,
        formula_from: b"$*1609.344\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1609.344\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_MILLIMETERS as DWORD,
        formula_from: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_NAUTICAL_MILES as DWORD,
        formula_from: b"$*1852\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1852\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_NIEU as DWORD,
        formula_from: b"$*9.144/432\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9.144*432\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_PARSECS as DWORD,
        formula_from: b"$*3.085678X16\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3.085678X16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_PICAS as DWORD,
        formula_from: b"$*0.9144/216\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.9144*216\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_RODS as DWORD,
        formula_from: b"$*5.0292\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/5.0292\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_RI_JAPAN as DWORD,
        formula_from: b"$*129600/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33/129600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_RI_KOREA as DWORD,
        formula_from: b"$*12960/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33/12960\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_SAWK as DWORD,
        formula_from: b"$*9.144/18\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/9.144*18\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_SEN as DWORD,
        formula_from: b"$*40.64\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/40.64\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_SHAKU as DWORD,
        formula_from: b"$*10/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33/10\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_SPAN as DWORD,
        formula_from: b"$*0.9144/4\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*4/0.9144\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_SUN as DWORD,
        formula_from: b"$*1/33\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*33\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_VA as DWORD,
        formula_from: b"$*2.032\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/2.032\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_YARDS as DWORD,
        formula_from: b"$*0.9144\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.9144\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_YOTE as DWORD,
        formula_from: b"$*16256\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/16256\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_LENGTH_ZHANG as DWORD,
        formula_from: b"$/0.3\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*0.3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_POWER: [conv_t; 7] = [
    conv_t {
        unit: IDS_POWER_BTUS_PER_MINUTE as DWORD,
        formula_from: b"$*17.5842642\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/17.5842642\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_POWER_FPS_PER_MINUTE as DWORD,
        formula_from: b"$*0.02259696580552333\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.02259696580552333\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_POWER_HORSEPOWER as DWORD,
        formula_from: b"$*745.69987158227022\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/745.69987158227022\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_POWER_KILOWATTS as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_POWER_MEGAWATTS as DWORD,
        formula_from: b"$*1000000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_POWER_WATTS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_PRESSURE: [conv_t; 8] = [
    conv_t {
        unit: IDS_PRESSURE_ATMOSPHERES as DWORD,
        formula_from: b"$*101325\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/101325\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_BARS as DWORD,
        formula_from: b"$*100000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/100000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_HECTOPASCALS as DWORD,
        formula_from: b"$*100\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/100\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_KILOPASCALS as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_MM_OF_MERCURY as DWORD,
        formula_from: b"$*133.322\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/133.322\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_PASCALS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_PRESSURE_PSI as DWORD,
        formula_from: b"$*6894.757\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/6894.757\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_TIME: [conv_t; 10] = [
    conv_t {
        unit: IDS_TIME_MINUTES as DWORD,
        formula_from: b"$*60\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/60\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_DAYS as DWORD,
        formula_from: b"$*86400\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/86400\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_HOURS as DWORD,
        formula_from: b"$*3600\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_MILLISECONDS as DWORD,
        formula_from: b"$*0.001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_MICROSECONDS as DWORD,
        formula_from: b"$*0.000001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.000001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_NANOSECONDS as DWORD,
        formula_from: b"$*0.000000001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.000000001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_SECONDS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_WEEKS as DWORD,
        formula_from: b"$*604800\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/604800\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TIME_YEARS as DWORD,
        formula_from: b"$*31556952\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/31556952\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_TEMPERATURE: [conv_t; 5] = [
    conv_t {
        unit: IDS_TEMPERATURE_CELSIUS as DWORD,
        formula_from: b"$+273.15\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$-273.15\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TEMPERATURE_FAHRENHEIT as DWORD,
        formula_from: b"($+459.67)*5/9\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*9/5-459.67\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TEMPERATURE_KELVIN as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_TEMPERATURE_RANKINE as DWORD,
        formula_from: b"$*5/9\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*9/5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_VELOCITY: [conv_t; 9] = [
    conv_t {
        unit: IDS_VELOCITY_CMS_SECOND as DWORD,
        formula_from: b"$*.01\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/.01\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_FEET_SECOND as DWORD,
        formula_from: b"$*.3048\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/.3048\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_FEET_HOUR as DWORD,
        formula_from: b"$*.0000846666667\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/.0000846666667\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_KILOMETERS_HOUR as DWORD,
        formula_from: b"$*10/36\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*36/10\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_KNOTS as DWORD,
        formula_from: b"$*18.52/36\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$*36/18.52\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_MACH as DWORD,
        formula_from: b"$*340.3\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/340.3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_METERS_SECOND as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VELOCITY_MILES_HOUR as DWORD,
        formula_from: b"$*.44704\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/.44704\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_VOLUME: [conv_t; 29] = [
    conv_t {
        unit: IDS_VOLUME_BARRELS_UK as DWORD,
        formula_from: b"$*163.65924\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/163.65924\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_BARRELS_OIL as DWORD,
        formula_from: b"$*158.987295\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/158.987295\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_BUN as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_BUSHELS_UK as DWORD,
        formula_from: b"$*36.36872\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/36.36872\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_BUSHELS_US as DWORD,
        formula_from: b"$*35.23907017\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/35.23907017\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_CUBIC_CENTIMETERS as DWORD,
        formula_from: b"$*0.001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_CUBIC_FEET as DWORD,
        formula_from: b"$*28.316846\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/28.316846\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_CUBIC_INCHES as DWORD,
        formula_from: b"$*0.016387064\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.016387064\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_CUBIC_METERS as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_CUBIC_YARDS as DWORD,
        formula_from: b"$*764.554857\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/764.554857\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_FLUID_OUNCES_UK as DWORD,
        formula_from: b"$*0.0284130625\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.0284130625\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_FLUID_OUNCES_US as DWORD,
        formula_from: b"$*0.0295735295625\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.0295735295625\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_GALLONS_UK as DWORD,
        formula_from: b"$*4.54609\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.54609\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_GALLONS_DRY_US as DWORD,
        formula_from: b"$*4.40488377086\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/4.40488377086\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_GALLONS_LIQUID_US as DWORD,
        formula_from: b"$*3.785411784\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3.785411784\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_GOU as DWORD,
        formula_from: b"$*0.1809\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.1809\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_KWIAN as DWORD,
        formula_from: b"$*2000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/2000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_LITERS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_MILLILITERS as DWORD,
        formula_from: b"$*0.001\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.001\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_PINTS_UK as DWORD,
        formula_from: b"$*0.56826125\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.56826125\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_PINTS_DRY_US as DWORD,
        formula_from: b"$*0.5506104713575\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.5506104713575\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_PINTS_LIQUID_US as DWORD,
        formula_from: b"$*0.473176473\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.473176473\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_QUARTS_UK as DWORD,
        formula_from: b"$*1.1365225\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.1365225\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_QUARTS_DRY_US as DWORD,
        formula_from: b"$*1.101220942715\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1.101220942715\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_QUARTS_LIQUID_US as DWORD,
        formula_from: b"$*0.946352946\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.946352946\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_TANANLOUNG as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_TANG as DWORD,
        formula_from: b"$*20\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/20\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_VOLUME_TO as DWORD,
        formula_from: b"$*18040\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/18040\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_WEIGHT: [conv_t; 16] = [
    conv_t {
        unit: IDS_WEIGHT_BAHT as DWORD,
        formula_from: b"$*12.244\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/12.244\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_CARATS as DWORD,
        formula_from: b"$*0.2\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/0.2\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_GRAMS as DWORD,
        formula_from: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_KAN as DWORD,
        formula_from: b"$*3750\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3750\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_KILOGRAMS as DWORD,
        formula_from: b"$*1000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_KIN as DWORD,
        formula_from: b"$*600\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/600\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_MONME as DWORD,
        formula_from: b"$*3.75\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/3.75\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_OUNCES_AVOIRDUPOIS as DWORD,
        formula_from: b"$*28.349523125\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/28.349523125\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_OUNCES_TROY as DWORD,
        formula_from: b"$*31.1034768\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/31.1034768\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_POUNDS as DWORD,
        formula_from: b"$*453.59237\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/453.59237\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_QUINTAL_METRIC as DWORD,
        formula_from: b"$*100000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/100000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_STONES as DWORD,
        formula_from: b"$*6350.29318\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/6350.29318\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_TONNES as DWORD,
        formula_from: b"$*1000000\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1000000\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_TONS_UK as DWORD,
        formula_from: b"$*1016046.9088\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/1016046.9088\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: IDS_WEIGHT_TONS_US as DWORD,
        formula_from: b"$*907184.74\0" as *const u8 as *const ::core::ffi::c_char,
        formula_to: b"$/907184.74\0" as *const u8 as *const ::core::ffi::c_char,
    },
    conv_t {
        unit: 0 as DWORD,
        formula_from: ::core::ptr::null::<::core::ffi::c_char>(),
        formula_to: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut conv_table: [conv_category_t; 13] = unsafe {
    [
        conv_category_t {
            category: IDS_CONV_ANGLE as DWORD,
            items: &raw const conv_ANGLE as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_AREA as DWORD,
            items: &raw const conv_AREA as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_CONSUMPTION as DWORD,
            items: &raw const conv_CONSUMPTION as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_CURRENCY as DWORD,
            items: &raw const conv_CURRENCY as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_ENERGY as DWORD,
            items: &raw const conv_ENERGY as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_LENGTH as DWORD,
            items: &raw const conv_LENGTH as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_POWER as DWORD,
            items: &raw const conv_POWER as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_PRESSURE as DWORD,
            items: &raw const conv_PRESSURE as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_TIME as DWORD,
            items: &raw const conv_TIME as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_TEMPERATURE as DWORD,
            items: &raw const conv_TEMPERATURE as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_VELOCITY as DWORD,
            items: &raw const conv_VELOCITY as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_VOLUME as DWORD,
            items: &raw const conv_VOLUME as *const conv_t,
        },
        conv_category_t {
            category: IDS_CONV_WEIGHT as DWORD,
            items: &raw const conv_WEIGHT as *const conv_t,
        },
    ]
};

// Unit IDs (IDS_<CATEGORY>_<UNIT>) are referenced by name; their numeric values come from the resource header.

#[repr(C)]
pub struct conv_t {
    pub unit: DWORD,
    pub formula_from: *const i8,
    pub formula_to: *const i8,
}

#[repr(C)]
pub struct conv_category_t {
    pub category: DWORD,
    pub items: *const conv_t,
}

unsafe impl Sync for conv_t {}
unsafe impl Sync for conv_category_t {}
#[no_mangle]
pub unsafe extern "C" fn ConvExecute(hWnd: HWND) {
    let mut c_cat: DWORD = SendDlgItemMessage(hWnd, (IDC_COMBO_CATEGORY as i32), CB_GETCURSEL, 0, 0) as DWORD;
    let mut items: *const conv_t = core::ptr::null();
    let mut from: DWORD = SendDlgItemMessage(hWnd, (IDC_COMBO_FROM as i32), CB_GETCURSEL, 0, 0) as DWORD;
    let mut to: DWORD = SendDlgItemMessage(hWnd, (IDC_COMBO_TO as i32), CB_GETCURSEL, 0, 0) as DWORD;
    let mut txt_cb: [TCHAR; 128] = [0; 128];
    let mut txt: [TCHAR; 128] = [0; 128];
    let mut item: *const conv_t = core::ptr::null();

    /* do nothing if the indexes point to the same unit */
    if from == to {
        return;
    }

    /* Search correct category, since it can be sorted too */
    SendDlgItemMessage(hWnd, (IDC_COMBO_CATEGORY as i32), CB_GETLBTEXT, c_cat as WPARAM, txt_cb.as_mut_ptr() as LPARAM);
    c_cat = 0 as DWORD;
    while (c_cat as usize) < SIZEOF!(conv_table) {
        LoadString(calc.hInstance, conv_table[c_cat as usize].category, txt.as_mut_ptr(), (SIZEOF!(txt)) as i32);
        if _tcscmp(txt_cb.as_mut_ptr(), txt.as_mut_ptr()) == 0 {
            items = conv_table[c_cat as usize].items;
            break;
        }
        c_cat += 1;
    }

    /* The units can be sorted, so I must search the exact match */
    item = items;
    SendDlgItemMessage(hWnd, (IDC_COMBO_FROM as i32), CB_GETLBTEXT, from as WPARAM, txt_cb.as_mut_ptr() as LPARAM);
    while (*item).unit != 0 {
        LoadString(calc.hInstance, (*item).unit, txt.as_mut_ptr(), (SIZEOF!(txt)) as i32);
        if _tcscmp(txt_cb.as_mut_ptr(), txt.as_mut_ptr()) == 0 {
            from = item.offset_from(items) as DWORD;
            break;
        }
        item = item.add(1);
    }
    SendDlgItemMessage(hWnd, (IDC_COMBO_TO as i32), CB_GETLBTEXT, to as WPARAM, txt_cb.as_mut_ptr() as LPARAM);
    item = items;
    while (*item).unit != 0 {
        LoadString(calc.hInstance, (*item).unit, txt.as_mut_ptr(), (SIZEOF!(txt)) as i32);
        if _tcscmp(txt_cb.as_mut_ptr(), txt.as_mut_ptr()) == 0 {
            to = item.offset_from(items) as DWORD;
            break;
        }
        item = item.add(1);
    }

    calc.Convert[0].data = (*items.add(from as usize)).formula_from as *mut ::core::ffi::c_char;
    calc.Convert[1].data = (*items.add(to as usize)).formula_to as *mut ::core::ffi::c_char;
    calc.Convert[0].wm_msg = WM_HANDLE_FROM;
    calc.Convert[1].wm_msg = WM_HANDLE_TO;
    PostMessage(hWnd, WM_START_CONV, 0, MAKELPARAM!(0, WM_HANDLE_FROM));
}
#[no_mangle]
pub unsafe extern "C" fn ConvAdjust(hWnd: HWND, n_cat: ::core::ffi::c_int) {
    let mut txt: [TCHAR; 128] = [0; 128];
    let mut txt_cat: [TCHAR; 128] = [0; 128];
    let hFromWnd: HWND = GetDlgItem(hWnd, (IDC_COMBO_FROM as i32));
    let hToWnd: HWND = GetDlgItem(hWnd, (IDC_COMBO_TO as i32));
    let mut item: *const conv_t = core::ptr::null();
    let mut n: ::core::ffi::c_uint;

    SendDlgItemMessage(
        hWnd,
        (IDC_COMBO_CATEGORY as i32),
        CB_GETLBTEXT,
        n_cat as WPARAM,
        core::ptr::addr_of_mut!(txt_cat) as *mut TCHAR as LPARAM,
    );
    n = 0;
    while (n as usize) < SIZEOF!(conv_table) {
        item = conv_table[n as usize].items;
        LoadString(
            calc.hInstance,
            conv_table[n as usize].category,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR,
            (SIZEOF!(txt)) as i32,
        );
        if _tcscmp(
            core::ptr::addr_of_mut!(txt_cat) as *mut TCHAR,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR,
        ) == 0
        {
            break;
        }
        n = n.wrapping_add(1);
    }

    SendMessage(hFromWnd, CB_RESETCONTENT, 0, 0);
    SendMessage(hToWnd, CB_RESETCONTENT, 0, 0);
    while (*item).unit != 0 {
        LoadString(
            calc.hInstance,
            (*item).unit,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR,
            (SIZEOF!(txt)) as i32,
        );
        SendMessage(
            hFromWnd,
            CB_ADDSTRING,
            0,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR as LPARAM,
        );
        SendMessage(
            hToWnd,
            CB_ADDSTRING,
            0,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR as LPARAM,
        );
        item = item.offset(1);
    }
    SendMessage(hFromWnd, CB_SETCURSEL, 0, 0);
    SendMessage(hToWnd, CB_SETCURSEL, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn ConvInit(hWnd: HWND) {
    let hCatWnd: HWND = GetDlgItem(hWnd, IDC_COMBO_CATEGORY as _);
    let mut txt: [TCHAR; 128] = [0; 128];
    let mut n: ::core::ffi::c_uint;

    /* Fill category combo */
    n = 0 as ::core::ffi::c_uint;
    while n < SIZEOF!(conv_table) as ::core::ffi::c_uint {
        LoadString(
            calc.hInstance,
            conv_table[n as usize].category as UINT,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR,
            SIZEOF!(txt) as ::core::ffi::c_int,
        );
        SendMessage(
            hCatWnd,
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(txt) as *mut TCHAR as LPARAM,
        );
        n = n.wrapping_add(1);
    }
    SendMessage(
        hCatWnd,
        CB_SETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    ConvAdjust(hWnd, 0 as ::core::ffi::c_int);
}
