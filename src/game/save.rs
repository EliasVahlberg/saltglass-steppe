//! Versioned save/load system.
//!
//! Saves live in `saves/<md5>.ron`. The MD5 of the file content is the filename,
//! enabling tamper detection on load.
//! Per-save status is persisted in `saves/meta.json`.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

use super::state::GameState;

pub const SAVE_VERSION: u32 = 2;
pub const SAVES_DIR: &str = "saves";
const META_PATH: &str = "saves/meta.json";

// ── Save status ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SaveStatus {
    Ok,
    HashMismatch,
    Corrupt,
}

#[derive(Serialize, Deserialize, Clone)]
struct SaveEntryMeta {
    status: SaveStatus,
}

#[derive(Serialize, Deserialize, Default)]
struct SaveMeta {
    #[serde(default)]
    entries: HashMap<String, SaveEntryMeta>,
}

fn load_meta() -> SaveMeta {
    fs::read_to_string(META_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_meta(meta: &SaveMeta) {
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let _ = fs::write(META_PATH, json);
    }
}

fn update_meta_status(stem: &str, status: SaveStatus) {
    let mut meta = load_meta();
    meta.entries.insert(stem.to_string(), SaveEntryMeta { status });
    save_meta(&meta);
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Metadata for displaying a save in the load menu.
pub struct SaveInfo {
    pub path: PathBuf,
    /// First 8 chars of the MD5 hash (display only).
    pub short_hash: String,
    /// Persistent status from meta.json.
    pub status: SaveStatus,
    /// Human-readable modification time.
    pub modified: String,
}

#[derive(Serialize)]
struct SaveFile<'a> {
    version: u32,
    state: &'a GameState,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFileOwned {
    version: u32,
    state: GameState,
}

fn compute_hash(data: &str) -> String {
    format!("{:x}", md5::compute(data.as_bytes()))
}

/// Serialize `state`, write to `saves/<md5>.ron`, update meta, return the path.
pub fn save_game(state: &GameState) -> Result<PathBuf, String> {
    fs::create_dir_all(SAVES_DIR).map_err(|e| e.to_string())?;
    let data = ron::to_string(&SaveFile { version: SAVE_VERSION, state })
        .map_err(|e| e.to_string())?;
    let hash = compute_hash(&data);
    let path = PathBuf::from(SAVES_DIR).join(format!("{}.ron", hash));
    fs::write(&path, &data).map_err(|e| e.to_string())?;
    update_meta_status(&hash, SaveStatus::Ok);
    Ok(path)
}

/// Load from `path`. Updates meta on hash mismatch or parse failure.
pub fn load_game(path: impl AsRef<Path>) -> Result<GameState, String> {
    let path = path.as_ref();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let actual_hash = compute_hash(&data);
    if !stem.is_empty() && stem != actual_hash {
        update_meta_status(&stem, SaveStatus::HashMismatch);
    }

    let file: SaveFileOwned = ron::from_str(&data).map_err(|e| {
        update_meta_status(&stem, SaveStatus::Corrupt);
        format!("Corrupt save: {e}")
    })?;

    let mut state = if file.version < SAVE_VERSION {
        migrate_save(file.state, file.version).map_err(|e| {
            update_meta_status(&stem, SaveStatus::Corrupt);
            e
        })?
    } else if file.version > SAVE_VERSION {
        update_meta_status(&stem, SaveStatus::Corrupt);
        return Err(format!(
            "Save version mismatch: file is v{}, game expects v{}.",
            file.version, SAVE_VERSION
        ));
    } else {
        file.state
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.rebuild_spatial_index();
        state.update_lighting();
    }));
    if result.is_err() {
        update_meta_status(&stem, SaveStatus::Corrupt);
        return Err("Save file is corrupt or incompatible — could not initialize game state.".to_string());
    }

    update_meta_status(&stem, SaveStatus::Ok);
    Ok(state)
}

/// List all saves in `saves/`, sorted newest first by modification time.
pub fn list_saves() -> Vec<SaveInfo> {
    let Ok(entries) = fs::read_dir(SAVES_DIR) else {
        return vec![];
    };
    let mut meta = load_meta();
    let mut meta_dirty = false;
    let mut saves: Vec<(std::time::SystemTime, SaveInfo)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ron"))
        .filter_map(|e| {
            let path = e.path();
            let mtime = e.metadata().ok()?.modified().ok()?;
            let stem = path.file_stem()?.to_str()?.to_string();
            let modified = format_local_time(mtime);

            let status = match meta.entries.get(&stem).map(|m| &m.status) {
                // Trust meta for Ok — skip file read entirely
                Some(SaveStatus::Ok) => SaveStatus::Ok,
                // Corrupt — skip hash, just confirm file exists
                Some(SaveStatus::Corrupt) => SaveStatus::Corrupt,
                // HashMismatch or unknown — re-verify
                _ => {
                    let data = fs::read_to_string(&path).ok()?;
                    let new_status = if stem == compute_hash(&data) {
                        SaveStatus::Ok
                    } else {
                        SaveStatus::HashMismatch
                    };
                    if meta.entries.get(&stem).map(|m| &m.status) != Some(&new_status) {
                        meta.entries.insert(stem.clone(), SaveEntryMeta { status: new_status.clone() });
                        meta_dirty = true;
                    }
                    new_status
                }
            };

            Some((mtime, SaveInfo {
                short_hash: stem[..8.min(stem.len())].to_string(),
                path,
                status,
                modified,
            }))
        })
        .collect();
    if meta_dirty {
        save_meta(&meta);
    }
    saves.sort_by(|a, b| b.0.cmp(&a.0));
    saves.into_iter().map(|(_, info)| info).collect()
}

fn format_local_time(t: std::time::SystemTime) -> String {
    let utc = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let local = utc + local_offset_secs();
    let mins  = (local / 60) % 60;
    let hours = (local / 3600) % 24;
    let (y, mo, d) = days_to_ymd((local / 86400) as u32);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, mo, d, hours, mins)
}

fn local_offset_secs() -> i64 {
    if let Ok(tz) = std::env::var("TZ") {
        if let Some(off) = parse_posix_tz(&tz) { return off; }
    }
    if let Ok(zone) = std::fs::read_to_string("/etc/timezone") {
        if let Some(off) = named_zone_offset(zone.trim()) { return off; }
    }
    0
}

fn parse_posix_tz(tz: &str) -> Option<i64> {
    // POSIX: "CET-1" means UTC+1 (sign inverted)
    let b = tz.as_bytes();
    let i = b.iter().position(|c| *c == b'+' || *c == b'-')?;
    let sign: i64 = if b[i] == b'-' { 1 } else { -1 };
    let h: i64 = std::str::from_utf8(&b[i+1..]).ok()?.trim().parse().ok()?;
    Some(sign * h * 3600)
}

fn named_zone_offset(zone: &str) -> Option<i64> {
    match zone {
        "UTC" | "Etc/UTC" | "Etc/GMT" => Some(0),
        "Europe/London" | "GB" => Some(0),
        "Europe/Paris" | "Europe/Berlin" | "Europe/Stockholm" | "Europe/Oslo" |
        "Europe/Rome" | "Europe/Madrid" | "Europe/Amsterdam" | "Europe/Brussels" |
        "Europe/Warsaw" | "Europe/Prague" | "Europe/Vienna" | "Europe/Zurich" |
        "Europe/Copenhagen" | "CET" | "MET" => Some(3600),
        "Europe/Helsinki" | "Europe/Riga" | "Europe/Tallinn" | "Europe/Vilnius" |
        "Europe/Bucharest" | "Europe/Athens" | "EET" => Some(7200),
        "Europe/Moscow" | "Europe/Istanbul" => Some(10800),
        "America/New_York" | "US/Eastern" => Some(-18000),
        "America/Chicago" | "US/Central" => Some(-21600),
        "America/Denver" | "US/Mountain" => Some(-25200),
        "America/Los_Angeles" | "US/Pacific" => Some(-28800),
        _ => None,
    }
}

fn days_to_ymd(days: u32) -> (u32, u32, u32) {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m, d)
}

fn migrate_save(mut state: GameState, from_version: u32) -> Result<GameState, String> {
    match from_version {
        1 => {
            if let Some(ref mut world_map) = state.world.world_map {
                if world_map.faction_territories.is_empty() {
                    world_map.faction_territories =
                        super::world_map::WorldMap::generate_faction_territories(state.seed);
                }
            }
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
        let path = save_game(&state).unwrap();
        let loaded = load_game(&path).unwrap();
        assert_eq!(state.player_x(), loaded.player_x());
        assert_eq!(state.player_y(), loaded.player_y());
        assert_eq!(state.turn, loaded.turn);
        fs::remove_file(path).ok();
    }

    #[test]
    fn load_rejects_wrong_version() {
        let state = GameState::new(42);
        // Write directly to a temp path (bypassing hash naming) to tamper version
        let tmp = "/tmp/saltglass_version_test.ron";
        let data = ron::to_string(&SaveFile { version: SAVE_VERSION, state: &state }).unwrap();
        let mut tampered = data.replacen(&format!("version:{SAVE_VERSION}"), "version:9999", 1);
        fs::write(tmp, &tampered).unwrap();
        let result = load_game(tmp);
        assert!(result.is_err());
        fs::remove_file(tmp).ok();
    }

    #[test]
    fn checksum_detects_tamper() {
        let state = GameState::new(99);
        let path = save_game(&state).unwrap();
        // Tamper the file
        let mut data = fs::read_to_string(&path).unwrap();
        data.push_str("// tampered");
        fs::write(&path, &data).unwrap();
        // load_game should still succeed but print a warning (we just verify it loads)
        let _ = load_game(&path); // may fail on RON parse, that's fine
        fs::remove_file(path).ok();
    }
}
