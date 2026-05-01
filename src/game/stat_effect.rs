//! Unified stat effect system.
//!
//! All sources of stat modification (adaptations, equipment, status effects)
//! produce `StatEffect` entries. `resolve_stat()` collapses them to a single value.
//!
//! Resolution: sort by priority ascending, fold each operation sequentially.
//! Within a priority level, Add operations are commutative; Multiply operations
//! are applied to the running value in priority order.

/// A single stat modification from any source.
#[derive(Debug, Clone)]
pub struct StatEffect {
    /// The stat being modified. Convention: snake_case strings.
    /// Known stats: "armor", "damage_bonus", "reflex", "fov",
    ///              "accuracy_penalty", "ranged_accuracy_bonus",
    ///              "craft_ingredient_reduction"
    pub stat: &'static str,
    pub op: StatOp,
    /// Lower priority = applied first. Use 0.0 for base effects, 10.0 for
    /// conditional/late effects. Float allows inserting between existing values.
    pub priority: f32,
    /// For display and deduplication. E.g. "prismhide", "jacket_01", "blinded".
    pub source_id: String,
}

#[derive(Debug, Clone, Copy)]
pub enum StatOp {
    Add(f32),
    Multiply(f32),
}

/// Resolve all effects for a given stat to a single f32 value, starting from `base`.
/// Effects are sorted by priority (ascending) and applied sequentially.
pub fn resolve_stat(base: f32, stat: &str, effects: &[StatEffect]) -> f32 {
    let mut relevant: Vec<&StatEffect> = effects.iter()
        .filter(|e| e.stat == stat)
        .collect();
    relevant.sort_by(|a, b| a.priority.total_cmp(&b.priority));

    relevant.iter().fold(base, |acc, e| match e.op {
        StatOp::Add(v)      => acc + v,
        StatOp::Multiply(v) => acc * v,
    })
}

/// Resolve to i32, rounding toward zero.
pub fn resolve_stat_i32(base: i32, stat: &str, effects: &[StatEffect]) -> i32 {
    resolve_stat(base as f32, stat, effects) as i32
}

/// Collect all active stat effects from a PlayerState.
/// Used by both GameState::active_stat_effects() and QueryContext.
pub fn collect_player_stat_effects(
    player: &crate::game::player_state::PlayerState,
) -> Vec<StatEffect> {
    use crate::game::adaptation::get_adaptation_def;
    use crate::game::item::get_item_def;
    use crate::game::status::get_status_def;

    let mut effects: Vec<StatEffect> = Vec::new();

    // --- Adaptations ---
    for adaptation in &player.adaptations {
        let Some(def) = get_adaptation_def(adaptation.id()) else { continue };

        if def.stat_modifiers.armor != 0 {
            effects.push(StatEffect {
                stat: "armor",
                op: StatOp::Add(def.stat_modifiers.armor as f32),
                priority: 0.0,
                source_id: adaptation.id().to_string(),
            });
        }
        if def.stat_modifiers.damage_bonus != 0 {
            effects.push(StatEffect {
                stat: "damage_bonus",
                op: StatOp::Add(def.stat_modifiers.damage_bonus as f32),
                priority: 0.0,
                source_id: adaptation.id().to_string(),
            });
        }
        if def.stat_modifiers.reflex != 0 {
            effects.push(StatEffect {
                stat: "reflex",
                op: StatOp::Add(def.stat_modifiers.reflex as f32),
                priority: 0.0,
                source_id: adaptation.id().to_string(),
            });
        }

        for effect in &def.effects {
            let stat: Option<&'static str> = match effect.effect_type.as_str() {
                "fov_bonus"                  => Some("fov"),
                "ranged_accuracy_bonus"      => Some("ranged_accuracy_bonus"),
                "craft_ingredient_reduction" => Some("craft_ingredient_reduction"),
                _ => None,
            };
            if let (Some(stat), Some(value)) = (stat, effect.value) {
                effects.push(StatEffect {
                    stat,
                    op: StatOp::Add(value as f32),
                    priority: 0.0,
                    source_id: adaptation.id().to_string(),
                });
            }
        }
    }

    // --- Equipment ---
    for (_slot, item_id) in player.equipment.iter() {
        let Some(id) = item_id else { continue };
        let Some(def) = get_item_def(id) else { continue };
        if def.armor_value != 0 {
            effects.push(StatEffect {
                stat: "armor",
                op: StatOp::Add(def.armor_value as f32),
                priority: 0.0,
                source_id: format!("equip_{}", id),
            });
        }
    }

    // --- scar_lattice dynamic armor ---
    if player.scar_lattice_armor != 0 {
        effects.push(StatEffect {
            stat: "armor",
            op: StatOp::Add(player.scar_lattice_armor as f32),
            priority: 5.0,
            source_id: "scar_lattice_dynamic".to_string(),
        });
    }

    // --- Status effects ---
    for status in &player.status_effects {
        let Some(def) = get_status_def(&status.id) else { continue };
        if def.reduces_accuracy != 0 {
            effects.push(StatEffect {
                stat: "accuracy_penalty",
                op: StatOp::Add(def.reduces_accuracy as f32),
                priority: 0.0,
                source_id: status.id.clone(),
            });
        }
        if def.reduces_damage != 0 {
            effects.push(StatEffect {
                stat: "damage_bonus",
                op: StatOp::Add(-(def.reduces_damage as f32)),
                priority: 0.0,
                source_id: status.id.clone(),
            });
        }
        if def.blocks_healing {
            effects.push(StatEffect {
                stat: "blocks_healing",
                op: StatOp::Add(1.0),
                priority: 0.0,
                source_id: status.id.clone(),
            });
        }
    }

    // --- Skills ---
    use crate::game::skills::get_skill_def;
    for (skill_id, &level) in &player.skills.skills {
        let Some(def) = get_skill_def(skill_id) else { continue };
        for effect in &def.passive_effects {
            let stat: Option<&'static str> = match effect.effect_type.as_str() {
                "melee_accuracy_bonus"  => Some("melee_accuracy_bonus"),
                "melee_damage_bonus"    => Some("melee_damage_bonus"),
                "ranged_accuracy_bonus" => Some("ranged_accuracy_bonus"),
                "ranged_damage_bonus"   => Some("ranged_damage_bonus"),
                "armor"                 => Some("armor"),
                "damage_bonus"          => Some("damage_bonus"),
                "reflex"                => Some("reflex"),
                "fov"                   => Some("fov"),
                _ => None,
            };
            let Some(stat) = stat else { continue };
            let bonus = {
                let raw = effect.value_per_level * level as f32;
                if let Some(max) = effect.max_value { raw.min(max) } else { raw }
            };
            if bonus != 0.0 {
                effects.push(StatEffect {
                    stat,
                    op: StatOp::Add(bonus),
                    priority: 1.0,
                    source_id: format!("skill_{}", skill_id),
                });
            }
        }
    }

    effects
}
/// Base armor/reflex come from EnemyDef and are passed as base values to resolve_stat.
pub fn collect_enemy_stat_effects(enemy: &crate::game::enemy::Enemy) -> Vec<StatEffect> {
    let mut effects = Vec::new();
    for status in &enemy.status_effects {
        let Some(def) = crate::game::status::get_status_def(&status.id) else { continue };
        if def.reduces_damage != 0 {
            effects.push(StatEffect {
                stat: "armor",
                op: StatOp::Add(def.reduces_damage as f32),
                priority: 0.0,
                source_id: status.id.clone(),
            });
        }
        if def.reduces_accuracy != 0 {
            effects.push(StatEffect {
                stat: "reflex",
                op: StatOp::Add(-(def.reduces_accuracy as f32)),
                priority: 0.0,
                source_id: status.id.clone(),
            });
        }
    }
    effects
}
