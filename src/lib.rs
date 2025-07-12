use refx_pp::{
    Beatmap,
    model::mode::GameMode,
    any::PerformanceAttributes,
};
use interoptopus::{
    extra_type, ffi_function, ffi_type, function, patterns::option::FFIOption, Inventory,
    InventoryBuilder,
};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

pub mod mods;
use mods::{GameMods, parse_mods};

#[ffi_type]
#[repr(C)]
#[derive(Clone, Default, PartialEq)]
pub struct CalculatePerformanceResult {
    pub pp: f64,
    pub stars: f64,
}

impl std::fmt::Display for CalculatePerformanceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("CalculateResult");
        s.field("pp", &self.pp).field("stars", &self.stars);

        s.finish()
    }
}

impl CalculatePerformanceResult {
    fn from_attributes(attributes: PerformanceAttributes) -> Self {
        Self {
            pp: attributes.pp(),
            stars: attributes.stars(),
        }
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn calculate_score(
    beatmap_path: *const c_char,
    mode: u32,
    mods: *const c_char,
    max_combo: u32,
    accuracy: f64,
    miss_count: u32,
    passed_objects: FFIOption<u32>,
    lazer: bool
) -> CalculatePerformanceResult {
    let path_str = CStr::from_ptr(beatmap_path).to_str().unwrap();
    let beatmap = Beatmap::from_path(Path::new(path_str)).unwrap();
    
    let mode_ = match mode {
        0 => GameMode::Osu,
        1 => GameMode::Taiko,
        2 => GameMode::Catch,
        3 => GameMode::Mania,
        _ => panic!("Invalid mode"),
    };
    
    let mods_str = CStr::from_ptr(mods).to_str().unwrap();
    let mods = parse_mods(mods_str, mode_).unwrap_or_default();
    
    let mut calculator = beatmap
        .performance()
        .try_mode(mode_)
        .unwrap()
        .lazer(lazer)
        .combo(max_combo)
        .misses(miss_count);

    calculator = match mods {
        GameMods::Legacy(legacy_mods) => {
            if lazer {
                calculator.mods(legacy_mods)
            } else {
                calculator.mods(legacy_mods.bits())
            }
        },
        GameMods::Intermode(intermode_mods) => calculator.mods(intermode_mods),
        GameMods::Lazer(lazer_mods) => calculator.mods(lazer_mods),
    };

    if let Some(passed_objects) = passed_objects.into_option() {
        calculator = calculator.passed_objects(passed_objects);
    }

    calculator = calculator.accuracy(accuracy);

    let rosu_result = calculator.calculate();
    CalculatePerformanceResult::from_attributes(rosu_result)   
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn calculate_score_bytes(
    beatmap_bytes: *const u8, 
    len: u32,
    mode: u32,
    mods: u32,
    max_combo: u32,
    accuracy: f64,
    miss_count: u32,
    passed_objects: FFIOption<u32>,
    lazer: bool
) -> CalculatePerformanceResult {
    let beatmap = Beatmap::from_bytes(std::slice::from_raw_parts(beatmap_bytes, len as usize)).unwrap();
    let mut calculator = beatmap
        .performance()
        .try_mode(match mode {
            0 => GameMode::Osu,
            1 => GameMode::Taiko,
            2 => GameMode::Catch,
            3 => GameMode::Mania,
            _ => panic!("Invalid mode"),
        })
        .unwrap()
        .mods(mods)
        .lazer(lazer)
        .combo(max_combo)
        .misses(miss_count);

    if let Some(passed_objects) = passed_objects.into_option() {
        calculator = calculator.passed_objects(passed_objects);
    }

    calculator = calculator.accuracy(accuracy);

    let rosu_result = calculator.calculate();
    CalculatePerformanceResult::from_attributes(rosu_result)
}

pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(extra_type!(CalculatePerformanceResult))
        .register(function!(calculate_score))
        .register(function!(calculate_score_bytes))
        .inventory()
}