use super::System;
use crate::game::{
    combat::{default_weapon, get_weapon_def, roll_attack, CombatResult},
    effects::context::QueryContext,
    event::GameEvent,
    mutations::Mutation,
    player_state::ActivityField,
    progression::{max_level, stat_points_per_level, xp_for_level},
    state::{GameState, MsgType},
};
use rand_chacha::ChaCha8Rng;
use rand::Rng;

pub struct CombatSystem;

impl System for CombatSystem {
    fn update(&self, _state: &mut GameState) {}
    fn on_event(&self, _state: &mut GameState, _event: &GameEvent) {}
}

// ---------------------------------------------------------------------------
// Command handlers — pure, return Vec<Mutation>
// ---------------------------------------------------------------------------

fn apply_mocks(ctx: &QueryContext, mut result: CombatResult) -> CombatResult {
    if let Some(force_hit) = ctx.mock_combat_hit {
        result.hit = force_hit;
        if !force_hit { result.damage = 0; }
    }
    if let Some(dmg) = ctx.mock_combat_damage && result.hit {
        result.damage = dmg;
    }
    result
}

/// Compute the new XP and any resulting level-up mutations.
fn xp_mutations(current_xp: u32, current_level: u32, current_stat_pts: i32,
                current_skill_pts: u32, gain: u32) -> Vec<Mutation> {
    let mut out = Vec::new();
    let new_xp = current_xp + gain;
    out.push(Mutation::SetPlayerXp(new_xp));
    out.push(Mutation::LogMessage { text: format!("+{} XP", gain), msg_type: MsgType::System });

    let mut level = current_level;
    let mut stat_pts = current_stat_pts;
    let mut skill_pts = current_skill_pts;
    while level < max_level() {
        if new_xp >= xp_for_level(level + 1) {
            level += 1;
            let pts = stat_points_per_level();
            stat_pts += pts;
            skill_pts += 2;
            out.push(Mutation::SetPlayerLevel(level));
            out.push(Mutation::SetPlayerStatPoints(stat_pts));
            out.push(Mutation::SetPlayerSkillPoints(skill_pts));
            out.push(Mutation::LogMessage {
                text: format!("⬆ LEVEL {}! (+{} stat points, +2 skill points)", level, pts),
                msg_type: MsgType::System,
            });
        } else {
            break;
        }
    }
    out
}

/// Command handler: melee attack at (target_x, target_y).
pub fn handle_melee(
    target_x: i32, target_y: i32,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> Vec<Mutation> {
    let mut out = Vec::new();

    let ei = match ctx.enemy_idx_at(target_x, target_y) {
        Some(i) => i,
        None => return out,
    };

    let cost = crate::game::action::action_cost("attack_melee");
    if ctx.player.ap < cost { return out; }

    // killing_edge: if kill_ap_refund_active, this attack costs 0 AP and clears the flag
    if ctx.player.kill_ap_refund_active {
        out.push(Mutation::SetKillApRefund(false));
    } else {
        out.push(Mutation::SetPlayerAp(ctx.player.ap - cost));
    }
    out.push(Mutation::SetEnemyProvoked { idx: ei, provoked: true });

    let weapon = ctx.player.equipped_weapon.as_ref()
        .and_then(|id| get_weapon_def(id))
        .unwrap_or_else(default_weapon);

    let enemy = match ctx.enemy(ei) {
        Some(e) => e,
        None => return out,
    };
    let enemy_reflex = enemy.def().map(|d| d.reflex).unwrap_or(0);
    let enemy_armor  = enemy.def().map(|d| d.armor).unwrap_or(0);
    let accuracy_bonus = ctx.player.skills.melee_accuracy_bonus();
    let damage_bonus   = ctx.player.skills.melee_damage_bonus();

    let result = apply_mocks(ctx, roll_attack(rng, weapon, enemy_reflex, enemy_armor,
                                              -(accuracy_bonus * 100.0) as i32));
    let name = enemy.name().to_string();

    if !result.hit {
        out.push(Mutation::LogMessage { text: format!("You miss the {}.", name), msg_type: MsgType::Combat });
        return out;
    }

    let adapt_mods = crate::game::adaptation::total_stat_modifiers(&ctx.player.adaptations);
    let dmg = ((result.damage as f32 * (1.0 + damage_bonus)) as i32 + adapt_mods.damage_bonus).max(0);

    out.push(Mutation::SetEnemyHp { idx: ei, hp: enemy.hp - dmg });
    out.push(Mutation::SetLastDamageDealt(dmg as u32));

    // bone_spur: 20% chance to apply bleed on hit
    if ctx.player.adaptations.iter().any(|a| a.id() == "bone_spur")
        && rng.gen_range(0..100) < 20
    {
        out.push(Mutation::AddEnemyStatus { idx: ei, id: "bleed".into(), duration: 3 });
    }

    if enemy.hp - dmg <= 0 {
        out.push(Mutation::RemoveEnemy { idx: ei, x: target_x, y: target_y });
        if let Some(def) = enemy.def() && def.xp_value > 0 {
            out.extend(xp_mutations(ctx.player.xp, ctx.player.level,
                                    ctx.player.pending_stat_points,
                                    ctx.player.skills.skill_points, def.xp_value));
            if def.xp_value >= 50 {
                out.push(Mutation::IncrementActivity(ActivityField::EliteEnemiesKilled));
            }
        }
        out.push(Mutation::IncrementActivity(ActivityField::EnemiesKilledMelee));
        // killing_edge: grant free AP on next attack
        if ctx.player.adaptations.iter().any(|a| a.id() == "killing_edge") {
            out.push(Mutation::SetKillApRefund(true));
        }
        out.push(Mutation::LogMessage { text: format!("You kill the {}!", name), msg_type: MsgType::Combat });
    } else {
        let crit = if result.crit { " CRITICAL!" } else { "" };
        out.push(Mutation::LogMessage {
            text: format!("You hit the {} for {} damage.{}", name, dmg, crit),
            msg_type: MsgType::Combat,
        });
    }
    out
}

/// Command handler: ranged attack at (target_x, target_y).
pub fn handle_ranged(
    target_x: i32, target_y: i32,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> Vec<Mutation> {
    let mut out = Vec::new();

    let weapon = match ctx.player.equipped_weapon.as_ref().and_then(|id| get_weapon_def(id)) {
        Some(w) if w.range > 1 => w,
        _ => {
            out.push(Mutation::LogMessage { text: "No ranged weapon equipped.".into(), msg_type: MsgType::Combat });
            return out;
        }
    };

    let dist = (target_x - ctx.player.x).abs() + (target_y - ctx.player.y).abs();
    if dist > weapon.range {
        out.push(Mutation::LogMessage { text: "Target out of range.".into(), msg_type: MsgType::Combat });
        return out;
    }

    let target_idx = ctx.map.idx(target_x, target_y);
    if !ctx.visible.contains(&target_idx) {
        out.push(Mutation::LogMessage { text: "Can't see target.".into(), msg_type: MsgType::Combat });
        return out;
    }

    let cost = weapon.ap_cost;
    if ctx.player.ap < cost { return out; }

    if let Some(ammo_type) = &weapon.ammo_type {
        match ctx.player.inventory.iter().position(|id| id == ammo_type) {
            Some(idx) => out.push(Mutation::RemoveFromInventory(idx)),
            None => {
                out.push(Mutation::LogMessage {
                    text: format!("Out of {}.", ammo_type.replace('_', " ")),
                    msg_type: MsgType::Combat,
                });
                return out;
            }
        }
    }

    out.push(Mutation::SetPlayerAp(ctx.player.ap - cost));

    // Projectile visual
    if weapon.range > 1 {
        let ch = if weapon.range > 3 { '*' } else { '-' };
        out.push(Mutation::SpawnProjectile { from: (ctx.player.x, ctx.player.y), to: (target_x, target_y), ch });
    }

    let ei = match ctx.enemy_idx_at(target_x, target_y) {
        Some(i) => i,
        None => {
            out.push(Mutation::LogMessage { text: "No target there.".into(), msg_type: MsgType::Combat });
            return out;
        }
    };

    out.push(Mutation::SetEnemyProvoked { idx: ei, provoked: true });

    let enemy = match ctx.enemy(ei) {
        Some(e) => e,
        None => return out,
    };
    let enemy_reflex = enemy.def().map(|d| d.reflex).unwrap_or(0);
    let enemy_armor  = enemy.def().map(|d| d.armor).unwrap_or(0);
    let accuracy_bonus = ctx.player.skills.ranged_accuracy_bonus();
    let damage_bonus   = ctx.player.skills.ranged_damage_bonus();

    // lens_eye: never miss within ranged_accuracy_bonus tiles
    let dist = (target_x - ctx.player.x).abs().max((target_y - ctx.player.y).abs());
    let lens_range: i32 = ctx.player.adaptations.iter()
        .filter_map(|a| a.def())
        .flat_map(|d| d.effects.iter())
        .filter(|e| e.effect_type == "ranged_accuracy_bonus")
        .filter_map(|e| e.value)
        .sum();
    let guaranteed_hit = lens_range > 0 && dist <= lens_range;

    let result = if guaranteed_hit {
        crate::game::combat::CombatResult { hit: true, damage: roll_attack(rng, weapon, enemy_reflex, enemy_armor, -(accuracy_bonus * 100.0) as i32).damage, crit: false }
    } else {
        apply_mocks(ctx, roll_attack(rng, weapon, enemy_reflex, enemy_armor, -(accuracy_bonus * 100.0) as i32))
    };
    let name = enemy.name().to_string();

    if !result.hit {
        out.push(Mutation::LogMessage { text: format!("Your shot misses the {}.", name), msg_type: MsgType::Combat });
        return out;
    }

    let dmg = ((result.damage as f32 * (1.0 + damage_bonus)) as i32).max(0);
    out.push(Mutation::SetEnemyHp { idx: ei, hp: enemy.hp - dmg });

    if enemy.hp - dmg <= 0 {
        out.push(Mutation::RemoveEnemy { idx: ei, x: target_x, y: target_y });
        if let Some(def) = enemy.def() && def.xp_value > 0 {
            out.extend(xp_mutations(ctx.player.xp, ctx.player.level,
                                    ctx.player.pending_stat_points,
                                    ctx.player.skills.skill_points, def.xp_value));
            if def.xp_value >= 50 {
                out.push(Mutation::IncrementActivity(ActivityField::EliteEnemiesKilled));
            }
        }
        out.push(Mutation::IncrementActivity(ActivityField::EnemiesKilledRanged));
        out.push(Mutation::LogMessage {
            text: format!("You kill the {} with a ranged shot!", name),
            msg_type: MsgType::Combat,
        });
    } else {
        let crit = if result.crit { " CRITICAL!" } else { "" };
        out.push(Mutation::LogMessage {
            text: format!("You hit the {} for {} damage.{}", name, dmg, crit),
            msg_type: MsgType::Combat,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Notification handlers — called from notify.rs
// ---------------------------------------------------------------------------

/// EnemyHpChanged: swarm aggro + reflect damage.
pub fn on_enemy_hit(state: &GameState, idx: usize, old_hp: i32, new_hp: i32) -> Vec<Mutation> {
    let mut out = Vec::new();
    let damage = old_hp - new_hp;
    if damage <= 0 { return out; }

    let enemy = match state.world.enemies.get(idx) {
        Some(e) => e,
        None => return out,
    };

    // Swarm aggro
    if enemy.def().map(|d| d.swarm).unwrap_or(false) {
        let (id, ex, ey) = (enemy.id.clone(), enemy.x, enemy.y);
        let mut alerted = 0;
        for e in &state.world.enemies {
            if e.id == id && !e.provoked {
                let dist = (e.x - ex).abs() + (e.y - ey).abs();
                if dist <= 8 { alerted += 1; }
            }
        }
        if alerted > 0 {
            out.push(Mutation::LogMessage { text: "The swarm is alerted!".into(), msg_type: MsgType::Combat });
        }
        // Actual provoke mutations emitted per-enemy
        for (i, e) in state.world.enemies.iter().enumerate() {
            if e.id == id && !e.provoked {
                let dist = (e.x - ex).abs() + (e.y - ey).abs();
                if dist <= 8 { out.push(Mutation::SetEnemyProvoked { idx: i, provoked: true }); }
            }
        }
    }

    // Reflect damage
    if let Some(def) = enemy.def() {
        for behavior in &def.behaviors {
            if behavior.behavior_type == "reflect_damage" {
                let percent = behavior.percent.unwrap_or(25);
                let reflected = (damage as u32 * percent / 100) as i32;
                if reflected > 0 {
                    out.push(Mutation::SetPlayerHp(state.player.hp - reflected));
                    out.push(Mutation::LogMessage {
                        text: format!("The enemy reflects {} damage back at you!", reflected),
                        msg_type: MsgType::Combat,
                    });
                }
            }
        }
    }

    // Visual
    out.push(Mutation::HitFlash { x: enemy.x, y: enemy.y });
    out.push(Mutation::DamageNumber { x: enemy.x, y: enemy.y, value: damage, is_heal: false });

    out
}

/// EnemyHpReachedZero: on-death effects, split-on-death, loot, quest.
pub fn on_enemy_killed(state: &GameState, idx: usize, enemy_id: &str, x: i32, y: i32,
                       rng: &mut ChaCha8Rng) -> Vec<Mutation> {
    let mut out = Vec::new();

    if let Some(def) = state.world.enemies.get(idx).and_then(|e| e.def()) {
        let enemy_name = state.world.enemies[idx].name().to_string();

        // On-death triggered effects
        for e in &def.effects {
            if e.condition == "on_death" {
                out.push(Mutation::TriggerEffect { effect: e.effect.clone(), duration: 3 });
            }
        }

        // split_on_death
        for behavior in &def.behaviors {
            if behavior.behavior_type == "split_on_death"
                && let Some(child_id) = &behavior.condition
            {
                let count = behavior.value.unwrap_or(2) as usize;
                let mut spawned = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        if spawned >= count { break; }
                        let (nx, ny) = (x + dx, y + dy);
                        if state.world.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                            && state.world.enemies.iter().all(|e| e.x != nx || e.y != ny)
                            && !(nx == state.player.x && ny == state.player.y)
                        {
                            out.push(Mutation::SpawnEnemy { id: child_id.clone(), x: nx, y: ny });
                            spawned += 1;
                        }
                    }
                }
                if spawned > 0 {
                    out.push(Mutation::LogMessage {
                        text: format!("The {} splits into smaller forms!", enemy_name),
                        msg_type: MsgType::Combat,
                    });
                }
            }
        }
    }

    // Loot drop
    let loot_out = crate::game::rules::reactions::reaction_loot_drop(enemy_id, x, y, rng);
    for effect in &loot_out.effects {
        // Convert SpawnOnMap effect to Mutation
        if let crate::game::effects::Effect::Item(crate::game::effects::ItemEffect::SpawnOnMap { item_id, x, y }) = effect {
            out.push(Mutation::SpawnItemOnMap { item_id: item_id.clone(), x: *x, y: *y });
        }
    }
    for p in &loot_out.presentation {
        let crate::game::effects::Presentation::LogMessage { text, msg_type } = p;
        let mt = match msg_type.as_str() {
            "loot" => MsgType::Loot,
            _ => MsgType::System,
            };
            out.push(Mutation::LogMessage { text: text.clone(), msg_type: mt });
    }

    out
}
