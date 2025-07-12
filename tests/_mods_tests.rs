use refx_pp::model::mode::GameMode;

use rosu_ffi::{calculate_score, mods::{parse_mods, GameMods}};
use interoptopus::patterns::option::FFIOption;

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