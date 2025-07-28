use serde_json;
use serde::de::{DeserializeSeed, IntoDeserializer};
use rosu_pp::model::mode::GameMode;

use rosu_mods::{
    GameMode as RosuGameMode,
    GameMods as GameModsLazer,
    GameModsIntermode,
    GameModsLegacy,
    serde::GameModSeed,
};

#[derive(Clone)]
pub enum GameMods {
    Legacy(GameModsLegacy),
    Intermode(GameModsIntermode),
    Lazer(GameModsLazer),
}

impl GameMods {
    pub fn from_legacy_bits(bits: u32) -> Self {
        Self::Legacy(GameModsLegacy::from_bits(bits))
    }

    pub fn from_acronyms(acronyms: &str) -> Self {
        let intermode = GameModsIntermode::from_acronyms(acronyms);

        match intermode.checked_bits() {
            Some(bits) => Self::Legacy(GameModsLegacy::from_bits(bits)),
            _ => Self::Intermode(intermode),
        }
    }

    pub fn from_json_str(json_str: &str, mode: GameMode) -> Result<Self, String> {
        let rosu_mode = match mode {
            GameMode::Osu => RosuGameMode::Osu,
            GameMode::Taiko => RosuGameMode::Taiko,
            GameMode::Catch => RosuGameMode::Catch,
            GameMode::Mania => RosuGameMode::Mania,
        };

        let seed = GameModSeed::Mode {
            mode: rosu_mode,
            deny_unknown_fields: false,
        };

        if json_str.starts_with('[') {
            let values: Vec<serde_json::Value> =
                serde_json::from_str(json_str).map_err(|e| format!("invalid JSON array: {e}"))?;

            let mut mods = GameModsLazer::new();

            for value in values {
                let res = if let Some(acronym) = value.as_str() {
                    seed.deserialize(serde_json::Value::String(acronym.to_string()).into_deserializer())
                } else if let Some(bits) = value.as_u64() {
                    seed.deserialize(serde_json::Value::Number(serde_json::Number::from(bits)).into_deserializer())
                } else {
                    seed.deserialize(value.into_deserializer())
                };

                match res {
                    Ok(m) => mods.insert(m),
                    Err(e) => return Err(format!("failed to deserialize mod: {e}")),
                }
            }

            Ok(Self::Lazer(mods))
        } else {
            let value = serde_json::from_str::<serde_json::Value>(json_str)
                .map_err(|e| format!("invalid JSON: {e}"))?;

            let m = seed.deserialize(value.into_deserializer())
                .map_err(|e| format!("failed to deserialize mod: {e}"))?;

            let mut mods = GameModsLazer::new();
            mods.insert(m);

            Ok(Self::Lazer(mods))
        }
    }
}

impl Default for GameMods {
    fn default() -> Self {
        Self::Legacy(GameModsLegacy::NoMod)
    }
}

pub fn parse_mods(input: &str, mode: GameMode) -> Result<GameMods, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(GameMods::default());
    }

    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        return GameMods::from_json_str(trimmed, mode);
    }

    if let Ok(bits) = trimmed.parse::<u32>() {
        return Ok(GameMods::from_legacy_bits(bits));
    }

    Ok(GameMods::from_acronyms(trimmed))
}