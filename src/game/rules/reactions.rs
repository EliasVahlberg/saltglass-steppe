use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::super::effects::{Effect, ItemEffect, Presentation, RuleOutput};
use super::super::enemy::get_enemy_def;
use super::super::item::get_item_def;

/// Reaction: enemy killed → roll loot table → spawn item on map.
/// Pure function: reads enemy def, uses rng, returns SpawnOnMap effect.
pub fn reaction_loot_drop(enemy_id: &str, x: i32, y: i32, rng: &mut ChaCha8Rng) -> RuleOutput {
    let mut effects = Vec::new();
    let mut presentation = Vec::new();

    let def = match get_enemy_def(enemy_id) {
        Some(d) => d,
        None => return RuleOutput { effects, presentation },
    };

    if def.loot_table.is_empty() {
        return RuleOutput { effects, presentation };
    }

    let total_weight: u32 = def.loot_table.iter().map(|e| e.weight).sum();
    if total_weight == 0 {
        return RuleOutput { effects, presentation };
    }

    let roll = rng.gen_range(0..total_weight);
    let mut cumulative = 0u32;
    for entry in &def.loot_table {
        cumulative += entry.weight;
        if roll < cumulative {
            effects.push(Effect::Item(ItemEffect::SpawnOnMap {
                item_id: entry.item.clone(),
                x,
                y,
            }));
            if let Some(item_def) = get_item_def(&entry.item) {
                presentation.push(Presentation::LogMessage {
                    text: format!("The enemy drops {}.", item_def.name),
                    msg_type: "loot".into(),
                });
            }
            break;
        }
    }

    RuleOutput { effects, presentation }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn loot_drop_unknown_enemy_produces_nothing() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let output = reaction_loot_drop("nonexistent_enemy", 5, 5, &mut rng);
        assert!(output.effects.is_empty());
    }
}
