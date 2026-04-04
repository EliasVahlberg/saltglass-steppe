use crate::game::action::action_cost;
use crate::game::effects::context::QueryContext;
use crate::game::effects::{
    Effect, EventEffect, MapEffect, MoveOutput, MoveResult, PlayerEffect, Presentation,
};
use crate::game::map::Tile;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Pure rule: determine movement outcome and produce effects.
///
/// Returns `MoveResult::Npc` or `MoveResult::Combat` if an NPC or enemy is at the
/// target — dispatch handles those branches imperatively (Phase 3+ migration).
/// Returns `MoveResult::Moved` with effects for the movement-only case.
pub fn rule_move(dx: i32, dy: i32, ctx: &QueryContext, rng: &mut ChaCha8Rng) -> MoveOutput {
    let new_x = ctx.player.x + dx;
    let new_y = ctx.player.y + dy;

    // Priority 1: NPC interaction
    if ctx.has_npc_at(new_x, new_y) {
        return MoveOutput {
            result: MoveResult::Npc,
            effects: vec![Effect::Player(PlayerEffect::ResetWaitCounter)],
            presentation: vec![],
        };
    }

    // Priority 2: Enemy combat (bump-to-attack)
    if ctx.has_enemy_at(new_x, new_y) {
        return MoveOutput {
            result: MoveResult::Combat,
            effects: vec![Effect::Player(PlayerEffect::ResetWaitCounter)],
            presentation: vec![],
        };
    }

    // Priority 3: Actual movement
    let tile = match ctx.map.get(new_x, new_y) {
        Some(t) => t.clone(),
        None => return blocked(),
    };

    let walkable = tile.walkable() || ctx.debug_phase;
    if !walkable {
        return blocked();
    }

    let cost = action_cost("move");
    if ctx.player.ap < cost {
        return blocked();
    }

    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    // Reset wait counter
    effects.push(Effect::Player(PlayerEffect::ResetWaitCounter));

    // Spend AP
    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));

    // Pre-movement: Mirage Step decoy
    if ctx
        .player
        .adaptations
        .iter()
        .any(|a| a.has_ability("mirage_step"))
    {
        effects.push(Effect::Player(PlayerEffect::PlaceDecoy {
            x: ctx.player.x,
            y: ctx.player.y,
        }));
    }

    // Update position
    effects.push(Effect::Player(PlayerEffect::SetPosition {
        x: new_x,
        y: new_y,
    }));

    // Clear storm highlight at new position
    let tile_idx = new_y as usize * ctx.map.width + new_x as usize;
    effects.push(Effect::Map(MapEffect::ClearStormHighlight { tile_index: tile_idx }));

    // Emit movement event
    effects.push(Effect::Event(EventEffect::EmitGameEvent {
        event_name: format!(
            "player_moved:{},{},{},{}",
            ctx.player.x, ctx.player.y, new_x, new_y
        ),
    }));

    // Tile effects
    match &tile {
        Tile::Glass => {
            if ctx
                .player
                .adaptations
                .iter()
                .any(|a| a.has_immunity("glass"))
            {
                presentation.push(Presentation::LogMessage {
                    text: "Your saltblood protects you from the glass.".into(),
                    msg_type: "system".into(),
                });
            } else {
                effects.push(Effect::Player(PlayerEffect::TakeDamage { amount: 1 }));
                effects.push(Effect::Player(PlayerEffect::ModifyRefraction { delta: 1 }));
                presentation.push(Presentation::LogMessage {
                    text: "Sharp glass cuts you! (-1 HP, +1 Refraction)".into(),
                    msg_type: "system".into(),
                });
            }
        }
        Tile::Glare => {
            effects.push(Effect::Player(PlayerEffect::SpendAp { amount: 1 }));
            presentation.push(Presentation::LogMessage {
                text: "Intense glare impairs your movement! (-1 AP)".into(),
                msg_type: "system".into(),
            });
            if rng.gen_range(0..100) < 30 {
                presentation.push(Presentation::LogMessage {
                    text: "The glare blinds you temporarily!".into(),
                    msg_type: "system".into(),
                });
            }
        }
        _ => {}
    }

    MoveOutput {
        result: MoveResult::Moved,
        effects,
        presentation,
    }
}

fn blocked() -> MoveOutput {
    MoveOutput {
        result: MoveResult::Blocked,
        effects: vec![],
        presentation: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::effects::context::TestContext;
    use rand::SeedableRng;

    fn movement_ctx() -> TestContext {
        TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_floor_at(6, 5)
            .with_floor_at(5, 6)
            .with_floor_at(4, 5)
    }

    #[test]
    fn move_to_floor_produces_position_and_ap() {
        let tc = movement_ctx();
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Moved);
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::SetPosition { x: 6, y: 5 })));
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::SpendAp {
            amount: action_cost("move")
        })));
    }

    #[test]
    fn move_into_wall_is_blocked() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10);
        // Map::new fills with walls, so (6,5) is a wall
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Blocked);
        assert!(output.effects.is_empty());
    }

    #[test]
    fn move_with_no_ap_is_blocked() {
        let tc = movement_ctx().with_player_ap(0);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Blocked);
    }

    #[test]
    fn move_into_npc_returns_npc_result() {
        let tc = movement_ctx().with_npc_at(6, 5, 0);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Npc);
    }

    #[test]
    fn move_into_enemy_returns_combat_result() {
        let tc = movement_ctx().with_enemy_at(6, 5, 0);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Combat);
    }

    #[test]
    fn move_onto_glass_produces_damage_and_refraction() {
        let tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_tile_at(6, 5, Tile::Glass);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Moved);
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::TakeDamage { amount: 1 })));
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::ModifyRefraction { delta: 1 })));
    }

    #[test]
    fn glass_immunity_prevents_damage() {
        use crate::game::adaptation::Adaptation;
        let mut tc = TestContext::new()
            .with_player_position(5, 5)
            .with_player_ap(10)
            .with_tile_at(6, 5, Tile::Glass);
        tc.player.adaptations.push(Adaptation::Saltblood);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let output = rule_move(1, 0, &ctx, &mut rng);

        assert_eq!(output.result, MoveResult::Moved);
        assert!(!output.effects.iter().any(|e| matches!(e, Effect::Player(PlayerEffect::TakeDamage { .. }))));
    }
}
