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

mod mods;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_parse_mods_num() {
        let result = parse_mods("64", GameMode::Osu);
        assert!(result.is_ok());
        match result.unwrap() {
            GameMods::Legacy(mods) => assert_eq!(mods.bits(), 64),
            _ => panic!("expected Legacy mods"),
        }
    }

    #[test]
    fn test_parse_mods_acr() {
        let result = parse_mods("HD", GameMode::Osu);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_mods_arr() {
        let result = parse_mods(r#"["HD", "DT"]"#, GameMode::Osu);
        assert!(result.is_ok());
        match result.unwrap() {
            GameMods::Lazer(_) => (),
            _ => panic!("expected Lazer mods"),
        }
    }

    #[test]
    fn test_parse_mods_empty() {
        let result = parse_mods("", GameMode::Osu);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_mods_invalid_json() {
        let result = parse_mods("[invalid json", GameMode::Osu);
        assert!(result.is_err());
    }

     #[test]
    fn test_parse_mods_with_settings() {
        let mod_str = r#"
        [
            {
                "acronym": "DT",
                "settings": {
                    "speed_change": 2.0
                }
            }
        ]
        "#;

        let mods = parse_mods(mod_str, GameMode::Osu).unwrap();

        match mods {
            GameMods::Lazer(m) => assert!(m.len() > 0),
            _ => panic!("expected lazer mods"),
        }
    }

    #[test]
    fn test_calculate_score_ffi() {
        let path = CString::new("resources/test.osu").unwrap();
        let mods = CString::new("64").unwrap();
        
        unsafe {
            let result = calculate_score(
                path.as_ptr(),
                0,
                mods.as_ptr(),
                1000,
                95.0,
                5,
                FFIOption::none(),
                false
            );
            
            assert!(result.pp > 0.0);
            assert!(result.stars > 0.0);
        }
    }

    #[test]
    fn test_calculate_score_ffi_lazer() {
        let path = CString::new("resources/test.osu").unwrap();
        let mods = CString::new(r#"["HD"]"#).unwrap();

        unsafe {
            let result = calculate_score(
                path.as_ptr(),
                0,
                mods.as_ptr(),
                1200,
                97.2,
                2,
                FFIOption::some(1200),
                true,
            );

            assert!(result.pp > 0.0);
            assert!(result.stars > 0.0);
        }
    }
}