//! Versioned save/load system.
//!
//! Wraps `GameState` in a [`SaveFile`] envelope that carries a schema version,
//! enabling forward-compatible saves and future migrations.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use super::state::GameState;

/// Current save schema version. Bump when `GameState` layout changes.
pub const SAVE_VERSION: u32 = 2;

/// Envelope for serialization (borrows state).
#[derive(Serialize)]
struct SaveFile<'a> {
    version: u32,
    state: &'a GameState,
}

/// Envelope for deserialization (owns state).
#[derive(Deserialize)]
struct SaveFileOwned {
    version: u32,
    state: GameState,
}

/// Serialize `state` inside a versioned envelope and write to `path`.
pub fn save_game(state: &GameState, path: impl AsRef<Path>) -> Result<(), String> {
    let data = ron::to_string(&SaveFile {
        version: SAVE_VERSION,
        state,
    })
    .map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

/// Read a versioned save from `path`. Returns a clear error on version mismatch.
pub fn load_game(path: impl AsRef<Path>) -> Result<GameState, String> {
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file: SaveFileOwned = ron::from_str(&data).map_err(|e| format!("Corrupt save: {e}"))?;
    
    // Handle version migrations
    let mut state = if file.version < SAVE_VERSION {
        eprintln!("Migrating save from v{} to v{}", file.version, SAVE_VERSION);
        migrate_save(file.state, file.version)?
    } else if file.version > SAVE_VERSION {
        return Err(format!(
            "Save version mismatch (too new): file is v{}, game expects v{}. \
             Please update the game.",
            file.version, SAVE_VERSION
        ));
    } else {
        file.state
    };
    
    state.rebuild_spatial_index();
    state.update_lighting();
    Ok(state)
}

/// Migrate old save to current version
fn migrate_save(mut state: GameState, from_version: u32) -> Result<GameState, String> {
    match from_version {
        1 => {
            // v1 -> v2: Add faction territories to world map
            if let Some(ref mut world_map) = state.world.world_map {
                if world_map.faction_territories.is_empty() {
                    eprintln!("Regenerating faction territories for existing world...");
                    world_map.faction_territories = 
                        super::world_map::WorldMap::generate_faction_territories(state.seed);
                }
            }
            // Faction reputation already exists in PlayerState, just ensure it's initialized
            // (empty HashMap is fine, will be populated as player interacts with factions)
            Ok(state)
        }
        _ => Err(format!("Unknown save version: {}", from_version)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_roundtrip_preserves_state() {
        let state = GameState::new(42);
        let path = "/tmp/saltglass_save_roundtrip_test.ron";
        save_game(&state, path).unwrap();
        let loaded = load_game(path).unwrap();
        assert_eq!(state.player_x(), loaded.player_x());
        assert_eq!(state.player_y(), loaded.player_y());
        assert_eq!(state.turn, loaded.turn);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn load_rejects_wrong_version() {
        let state = GameState::new(42);
        let path = "/tmp/saltglass_save_version_test.ron";
        save_game(&state, path).unwrap();

        // Tamper: replace version field
        let mut data = std::fs::read_to_string(path).unwrap();
        data = data.replacen(
            &format!("version:{SAVE_VERSION}"),
            "version:9999",
            1,
        );
        std::fs::write(path, &data).unwrap();

        let result = load_game(path);
        assert!(result.is_err(), "Expected version mismatch error");
        let err = result.err().unwrap();
        assert!(err.contains("version mismatch"), "Expected version mismatch error, got: {err}");
        std::fs::remove_file(path).ok();
    }
}
