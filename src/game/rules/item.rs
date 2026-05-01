use crate::game::action::action_cost;
use crate::game::effects::context::QueryContext;
use crate::game::effects::{
    Effect, EventEffect, ItemEffect, MapEffect, PlayerEffect, Presentation, ResourceEffect,
    RuleOutput,
};
use crate::game::item::get_item_def;
use crate::game::map::Tile;

/// Pure rule: use an item from inventory by index.
/// Translates the old GameState::use_item (~140 LOC) into effects.
pub fn rule_use_item(item_index: usize, ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    // Validate index
    let id = match ctx.player.inventory.get(item_index) {
        Some(id) => id.clone(),
        None => return RuleOutput { effects, presentation },
    };

    // AP check (before item def lookup, matching original order)
    let cost = action_cost("use_item");
    if ctx.player.ap < cost {
        return RuleOutput { effects, presentation };
    }

    let def = match get_item_def(&id) {
        Some(d) => d,
        None => return RuleOutput { effects, presentation },
    };

    if !def.usable {
        presentation.push(Presentation::LogMessage {
            text: format!("You can't use {} right now.", def.name),
            msg_type: "system".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Book path: no AP cost, no consumption, just open the book
    if let Some(book_id) = &def.book_id {
        effects.push(Effect::Event(EventEffect::OpenBook {
            book_id: book_id.clone(),
        }));
        presentation.push(Presentation::LogMessage {
            text: format!("You read {}.", def.name),
            msg_type: "system".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Spend AP
    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));

    // Healing
    if def.heal > 0 {
        let stat_effects = crate::game::stat_effect::collect_player_stat_effects(ctx.player);
        let healing_blocked = crate::game::stat_effect::resolve_stat_i32(0, "blocks_healing", &stat_effects) > 0;
        if healing_blocked {
            presentation.push(Presentation::LogMessage {
                text: format!("You use {} but bleeding prevents healing.", def.name),
                msg_type: "warning".into(),
            });
        } else {
            let heal = def.heal.min(ctx.player.max_hp - ctx.player.hp);
            effects.push(Effect::Player(PlayerEffect::Heal { amount: heal }));
            presentation.push(Presentation::LogMessage {
                text: format!("You use {}. (+{} HP)", def.name, heal),
                msg_type: "loot".into(),
            });
        }
    }

    // Refraction reduction
    if def.reduces_refraction > 0 {
        let reduce = def.reduces_refraction.min(ctx.player.refraction);
        effects.push(Effect::Player(PlayerEffect::ModifyRefraction {
            delta: -(reduce as i32),
        }));
        presentation.push(Presentation::LogMessage {
            text: format!("Your glow fades slightly. (-{} Refraction)", reduce),
            msg_type: "status".into(),
        });
    }

    // Suppression
    if def.suppresses_adaptations {
        effects.push(Effect::Player(PlayerEffect::SuppressAdaptations {
            turns: 10,
        }));
        presentation.push(Presentation::LogMessage {
            text: "Your glow dims. The tincture masks your changes.".into(),
            msg_type: "status".into(),
        });
    }

    // Map reveal
    if def.reveals_map {
        presentation.push(Presentation::LogMessage {
            text: format!("The {} reveals hidden paths...", def.name),
            msg_type: "loot".into(),
        });
        effects.push(Effect::Map(MapEffect::RevealAll));
    }

    // ARIA dialogue
    if def.enables_aria_dialogue {
        presentation.push(Presentation::LogMessage {
            text: "You interface with ARIA...".into(),
            msg_type: "system".into(),
        });
        effects.push(Effect::Event(EventEffect::QuestNotify {
            kind: super::super::effects::QuestNotifyKind::AriaInterface {
                item_id: def.id.clone(),
            },
        }));
    }

    // Light energy
    if def.light_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainLightEnergy {
            amount: def.light_energy,
        }));
        presentation.push(Presentation::LogMessage {
            text: format!(
                "Light energy surges through you! (+{} Light Energy)",
                def.light_energy
            ),
            msg_type: "status".into(),
        });
    }

    // Teaches light manipulation (presentation only)
    if def.teaches_light_manipulation {
        presentation.push(Presentation::LogMessage {
            text: "You learn to manipulate light! Use debug commands: focus_beam, create_prism"
                .into(),
            msg_type: "system".into(),
        });
    }

    // Void exposure
    if def.void_exposure > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainVoidExposure {
            amount: def.void_exposure,
        }));
        presentation.push(Presentation::LogMessage {
            text: format!(
                "Void corruption seeps into you! (+{} Void Exposure)",
                def.void_exposure
            ),
            msg_type: "status".into(),
        });
    }

    // Void energy
    if def.void_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainVoidEnergy {
            amount: def.void_energy,
        }));
        presentation.push(Presentation::LogMessage {
            text: format!(
                "Void energy flows through you! (+{} Void Energy)",
                def.void_energy
            ),
            msg_type: "status".into(),
        });
    }

    // Teaches crystal resonance (presentation only)
    if def.teaches_crystal_resonance {
        presentation.push(Presentation::LogMessage {
            text: "You learn crystal resonance! Use debug commands: create_crystal, resonate, harmonize".into(),
            msg_type: "system".into(),
        });
    }

    // Resonance energy
    if def.resonance_energy > 0 {
        effects.push(Effect::Resource(ResourceEffect::GainResonanceEnergy {
            amount: def.resonance_energy,
        }));
        presentation.push(Presentation::LogMessage {
            text: format!(
                "Crystal resonance fills you! (+{} Resonance Energy)",
                def.resonance_energy
            ),
            msg_type: "status".into(),
        });
    }

    // Crystal placement
    if let Some(frequency) = &def.crystal_frequency {
        effects.push(Effect::Resource(ResourceEffect::PlaceCrystal {
            x: ctx.player.x,
            y: ctx.player.y,
            frequency: frequency.clone(),
        }));
        presentation.push(Presentation::LogMessage {
            text: format!("A {} crystal grows at your feet!", frequency),
            msg_type: "loot".into(),
        });
    }

    // Consume if consumable
    if def.consumable {
        effects.push(Effect::Item(ItemEffect::Consume {
            item_id: id,
            inventory_index: item_index,
        }));
    }

    RuleOutput {
        effects,
        presentation,
    }
}

/// Pure rule: use an item on a specific tile (wall-breaking).
pub fn rule_use_item_on_tile(
    item_index: usize,
    x: i32,
    y: i32,
    ctx: &QueryContext,
) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    // Validate index
    let id = match ctx.player.inventory.get(item_index) {
        Some(id) => id.clone(),
        None => return RuleOutput { effects, presentation },
    };

    // Range check (adjacent only)
    let dx = (x - ctx.player.x).abs();
    let dy = (y - ctx.player.y).abs();
    if dx > 1 || dy > 1 {
        presentation.push(Presentation::LogMessage {
            text: "That is too far away.".into(),
            msg_type: "system".into(),
        });
        return RuleOutput { effects, presentation };
    }

    let cost = action_cost("use_item");
    if ctx.player.ap < cost {
        return RuleOutput { effects, presentation };
    }

    let def = match get_item_def(&id) {
        Some(d) => d,
        None => return RuleOutput { effects, presentation },
    };

    if !def.breaks_walls {
        presentation.push(Presentation::LogMessage {
            text: format!("You can't use {} on that.", def.name),
            msg_type: "system".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Validate tile is a wall
    let tile_idx = (y * ctx.map.width as i32 + x) as usize;
    if tile_idx >= ctx.map.tiles.len() {
        return RuleOutput { effects, presentation };
    }
    if !matches!(ctx.map.tiles[tile_idx], Tile::Wall { .. }) {
        presentation.push(Presentation::LogMessage {
            text: "You can only use this on walls.".into(),
            msg_type: "system".into(),
        });
        return RuleOutput { effects, presentation };
    }

    // Spend AP
    effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }));

    // Damage the wall
    effects.push(Effect::Map(MapEffect::DamageWall {
        x,
        y,
        damage: 10,
    }));

    presentation.push(Presentation::LogMessage {
        text: "You strike the wall. Cracks spread through the glass.".into(),
        msg_type: "combat".into(),
    });

    // Check if wall will break (hp <= damage)
    if let Tile::Wall { hp, .. } = &ctx.map.tiles[tile_idx]
        && *hp <= 10
    {
        presentation.push(Presentation::LogMessage {
            text: "The wall shatters!".into(),
            msg_type: "combat".into(),
        });
    }

    // Consume if consumable
    if def.consumable {
        effects.push(Effect::Item(ItemEffect::Consume {
            item_id: id,
            inventory_index: item_index,
        }));
    }

    RuleOutput {
        effects,
        presentation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::effects::context::TestContext;

    #[test]
    fn use_healing_item_produces_heal_and_consume() {
        let tc = TestContext::new()
            .with_player_hp(50)
            .with_player_max_hp(100)
            .with_player_ap(10)
            .with_inventory(vec!["brine_vial".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        assert!(output.effects.contains(&Effect::Player(PlayerEffect::SpendAp {
            amount: action_cost("use_item")
        })));
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::Heal { amount: 5 })));
        assert!(output.effects.contains(&Effect::Item(ItemEffect::Consume {
            item_id: "brine_vial".into(),
            inventory_index: 0,
        })));
    }

    #[test]
    fn use_item_with_no_ap_produces_nothing() {
        let tc = TestContext::new()
            .with_player_ap(0)
            .with_inventory(vec!["brine_vial".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        assert!(output.effects.is_empty());
    }

    #[test]
    fn use_item_invalid_index_produces_nothing() {
        let tc = TestContext::new().with_player_ap(10);
        let ctx = tc.build();

        let output = rule_use_item(5, &ctx);

        assert!(output.effects.is_empty());
    }

    #[test]
    fn use_non_usable_item_produces_log_only() {
        let tc = TestContext::new()
            .with_player_ap(10)
            .with_inventory(vec!["storm_glass".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        assert!(output.effects.is_empty());
        assert!(!output.presentation.is_empty());
    }

    #[test]
    fn use_map_reveal_item_produces_reveal_effect() {
        let tc = TestContext::new()
            .with_player_ap(10)
            .with_inventory(vec!["scripture_shard".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        assert!(output.effects.contains(&Effect::Map(MapEffect::RevealAll)));
    }

    #[test]
    fn use_book_produces_open_book_effect() {
        let tc = TestContext::new()
            .with_player_ap(10)
            .with_inventory(vec!["item_book_white_noon".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        assert!(output.effects.contains(&Effect::Event(EventEffect::OpenBook {
            book_id: "book_white_noon".into(),
        })));
        // Book path should NOT spend AP
        assert!(!output.effects.iter().any(|e| matches!(e, Effect::Player(PlayerEffect::SpendAp { .. }))));
    }

    #[test]
    fn healing_caps_at_max_hp() {
        let tc = TestContext::new()
            .with_player_hp(98)
            .with_player_max_hp(100)
            .with_player_ap(10)
            .with_inventory(vec!["brine_vial".into()]);
        let ctx = tc.build();

        let output = rule_use_item(0, &ctx);

        // brine_vial heals 5, but only 2 HP needed
        assert!(output.effects.contains(&Effect::Player(PlayerEffect::Heal { amount: 2 })));
    }
}
