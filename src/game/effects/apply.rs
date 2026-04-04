use super::{
    CombatEffect, Effect, EventEffect, ItemEffect, MapEffect, Presentation, PlayerEffect,
    ResourceEffect,
};
use crate::game::state::{GameState, MsgType};

impl GameState {
    pub fn apply_effect(&mut self, effect: &Effect) {
        match effect {
            Effect::Player(e) => self.apply_player_effect(e),
            Effect::Combat(e) => self.apply_combat_effect(e),
            Effect::Item(e) => self.apply_item_effect(e),
            Effect::Map(e) => self.apply_map_effect(e),
            Effect::Resource(e) => self.apply_resource_effect(e),
            Effect::Event(e) => self.apply_event_effect(e),
        }
    }

    fn apply_player_effect(&mut self, effect: &PlayerEffect) {
        match effect {
            PlayerEffect::Heal { amount } => {
                self.player.hp = (self.player.hp + amount).min(self.player.max_hp);
            }
            PlayerEffect::TakeDamage { amount } => {
                self.player.hp -= amount;
            }
            PlayerEffect::SpendAp { amount } => {
                self.player.ap -= amount;
            }
            PlayerEffect::SetPosition { x, y } => {
                self.player.x = *x;
                self.player.y = *y;
            }
            PlayerEffect::ModifyRefraction { delta } => {
                self.player.refraction =
                    (self.player.refraction as i32 + delta).max(0) as u32;
            }
            PlayerEffect::SuppressAdaptations { turns } => {
                self.player.adaptations_hidden_turns = *turns;
            }
            PlayerEffect::PlaceDecoy { x, y } => {
                self.decoys.push(crate::game::state::Decoy {
                    x: *x,
                    y: *y,
                    turns_remaining: 3,
                });
            }
            PlayerEffect::ResetWaitCounter => {
                self.wait_counter = 0;
            }
            PlayerEffect::GainXp { amount } => {
                self.gain_xp(*amount);
            }
            PlayerEffect::RecordDamageDealt { amount } => {
                self.player.last_damage_dealt = *amount;
            }
            PlayerEffect::ResetAp => {
                self.player.ap = self.player.max_ap;
            }
            PlayerEffect::AdvanceTurn => {
                self.turn += 1;
            }
        }
    }

    fn apply_combat_effect(&mut self, effect: &CombatEffect) {
        match effect {
            CombatEffect::DealDamage { enemy_idx, amount } => {
                if let Some(enemy) = self.world.enemies.get_mut(*enemy_idx) {
                    enemy.hp -= amount;
                }
            }
            CombatEffect::Miss { .. } => {}
            CombatEffect::Kill { enemy_id, x, y, .. } => {
                self.spatial.enemy_positions.remove(&(*x, *y));
                self.meta.discover_enemy(enemy_id);
                self.emit(crate::game::event::GameEvent::EnemyKilled {
                    enemy_id: enemy_id.clone(),
                    x: *x,
                    y: *y,
                });
            }
            CombatEffect::Provoke { enemy_idx } => {
                if let Some(enemy) = self.world.enemies.get_mut(*enemy_idx) {
                    enemy.provoked = true;
                }
            }
        }
    }

    fn apply_item_effect(&mut self, effect: &ItemEffect) {
        match effect {
            ItemEffect::Consume { inventory_index, .. } => {
                if *inventory_index < self.player.inventory.len() {
                    self.player.inventory.remove(*inventory_index);
                }
            }
            ItemEffect::RemoveFromInventory { index } => {
                if *index < self.player.inventory.len() {
                    self.player.inventory.remove(*index);
                }
            }
        }
    }

    fn apply_map_effect(&mut self, effect: &MapEffect) {
        match effect {
            MapEffect::RevealAll => {
                for idx in 0..self.world.map.tiles.len() {
                    self.revealed.insert(idx);
                }
            }
            MapEffect::ClearStormHighlight { tile_index } => {
                self.world
                    .visual_effects
                    .storm_changed_tiles
                    .remove(tile_index);
            }
            MapEffect::DamageWall { x, y, damage } => {
                let tile_idx = (*y * self.world.map.width as i32 + *x) as usize;
                if tile_idx < self.world.map.tiles.len() {
                    let mut broken = false;
                    if let crate::game::map::Tile::Wall { hp, .. } =
                        &mut self.world.map.tiles[tile_idx]
                    {
                        *hp -= damage;
                        if *hp <= 0 {
                            broken = true;
                        }
                    }
                    if broken {
                        self.world.map.tiles[tile_idx] =
                            crate::game::map::Tile::default_floor();
                        self.update_lighting();
                    }
                }
            }
        }
    }

    fn apply_resource_effect(&mut self, effect: &ResourceEffect) {
        match effect {
            ResourceEffect::GainLightEnergy { amount } => {
                self.player.light_system.light_energy += amount;
            }
            ResourceEffect::GainVoidEnergy { amount } => {
                self.player.void_system.gain_energy(*amount);
            }
            ResourceEffect::GainVoidExposure { amount } => {
                self.player.void_system.add_exposure(*amount);
            }
            ResourceEffect::GainResonanceEnergy { amount } => {
                self.player.crystal_system.resonance_energy =
                    (self.player.crystal_system.resonance_energy + amount)
                        .min(self.player.crystal_system.max_resonance_energy);
            }
            ResourceEffect::PlaceCrystal { x, y, frequency } => {
                let freq = match frequency.as_str() {
                    "alpha" => crate::game::crystal_resonance::CrystalFrequency::Alpha,
                    "beta" => crate::game::crystal_resonance::CrystalFrequency::Beta,
                    "gamma" => crate::game::crystal_resonance::CrystalFrequency::Gamma,
                    "delta" => crate::game::crystal_resonance::CrystalFrequency::Delta,
                    "epsilon" => crate::game::crystal_resonance::CrystalFrequency::Epsilon,
                    _ => crate::game::crystal_resonance::CrystalFrequency::Alpha,
                };
                self.player.crystal_system.add_crystal(*x, *y, freq);
            }
        }
    }

    fn apply_event_effect(&mut self, effect: &EventEffect) {
        match effect {
            EventEffect::OpenBook { book_id } => {
                self.pending_ui.book_open = Some(book_id.clone());
            }
            EventEffect::EmitGameEvent { event_name } => {
                // Parse and emit known game events
                if event_name.starts_with("player_moved:") {
                    let parts: Vec<&str> =
                        event_name.trim_start_matches("player_moved:").split(',').collect();
                    if parts.len() == 4
                        && let (Ok(fx), Ok(fy), Ok(tx), Ok(ty)) = (
                            parts[0].parse::<i32>(),
                            parts[1].parse::<i32>(),
                            parts[2].parse::<i32>(),
                            parts[3].parse::<i32>(),
                        )
                    {
                        self.emit(crate::game::event::GameEvent::PlayerMoved {
                            from_x: fx,
                            from_y: fy,
                            to_x: tx,
                            to_y: ty,
                        });
                    }
                } else if event_name.starts_with("aria_interfaced:") {
                    let item_id = event_name.trim_start_matches("aria_interfaced:").to_string();
                    self.emit(crate::game::event::GameEvent::AriaInterfaced { item_id });
                } else if event_name.starts_with("item_used:") {
                    let item_id = event_name.trim_start_matches("item_used:").to_string();
                    self.emit(crate::game::event::GameEvent::ItemUsed { item_id });
                } else if event_name.starts_with("void_exposure_changed:") {
                    if let Ok(level) = event_name
                        .trim_start_matches("void_exposure_changed:")
                        .parse::<u32>()
                    {
                        self.emit(crate::game::event::GameEvent::VoidExposureChanged { level });
                    }
                } else if event_name.starts_with("crystal_resonance_changed:") {
                    let frequency = event_name
                        .trim_start_matches("crystal_resonance_changed:")
                        .to_string();
                    self.emit(crate::game::event::GameEvent::CrystalResonanceChanged {
                        frequency,
                    });
                }
            }
        }
    }

    pub fn apply_presentation(&mut self, p: &Presentation) {
        match p {
            Presentation::LogMessage { text, msg_type } => {
                let mt = match msg_type.as_str() {
                    "combat" => MsgType::Combat,
                    "loot" => MsgType::Loot,
                    "status" => MsgType::Status,
                    "system" => MsgType::System,
                    "warning" => MsgType::Warning,
                    "dialogue" => MsgType::Dialogue,
                    "social" => MsgType::Social,
                    _ => MsgType::System,
                };
                self.log_typed(text.clone(), mt);
            }
        }
    }
}
