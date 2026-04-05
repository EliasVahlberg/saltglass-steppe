use super::{
    CombatEffect, Effect, EventEffect, ItemEffect, MapEffect, Presentation, PlayerEffect,
    QuestEffect, ResourceEffect,
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
            Effect::Quest(e) => self.apply_quest_effect(e),
        }
    }

    fn apply_player_effect(&mut self, effect: &PlayerEffect) {
        use crate::game::mutations::Mutation;
        match effect {
            PlayerEffect::Heal { amount } => {
                let new_hp = (self.player.hp + amount).min(self.player.max_hp);
                self.apply_one(&Mutation::SetPlayerHp(new_hp));
            }
            PlayerEffect::TakeDamage { amount } => {
                self.apply_one(&Mutation::SetPlayerHp(self.player.hp - amount));
            }
            PlayerEffect::SpendAp { amount } => {
                self.apply_one(&Mutation::SetPlayerAp(self.player.ap - amount));
            }
            PlayerEffect::SetPosition { x, y } => {
                self.apply_one(&Mutation::SetPlayerPosition { x: *x, y: *y });
            }
            PlayerEffect::ModifyRefraction { delta } => {
                let new_val = (self.player.refraction as i32 + delta).max(0) as u32;
                self.apply_one(&Mutation::SetPlayerRefraction(new_val));
            }
            PlayerEffect::SuppressAdaptations { turns } => {
                self.apply_one(&Mutation::SetAdaptationsHidden(*turns));
            }
            PlayerEffect::PlaceDecoy { x, y } => {
                self.apply_one(&Mutation::PlaceDecoy { x: *x, y: *y });
            }
            PlayerEffect::ResetWaitCounter => {
                self.apply_one(&Mutation::SetWaitCounter(0));
            }
            PlayerEffect::GainXp { amount } => {
                // Compound: level-up loop stays here until systems/player.rs is created (Stage 3)
                use crate::game::progression::{max_level, xp_for_level};
                self.player.xp += amount;
                self.log_typed(format!("+{} XP", amount), MsgType::System);
                while self.player.level < max_level() {
                    let next_threshold = xp_for_level(self.player.level + 1);
                    if self.player.xp >= next_threshold {
                        self.player.level += 1;
                        let points = crate::game::progression::stat_points_per_level();
                        self.player.pending_stat_points += points;
                        self.player.skills.skill_points += 2;
                        self.log_typed(
                            format!("⬆ LEVEL {}! (+{} stat points, +2 skill points)", self.player.level, points),
                            MsgType::System,
                        );
                    } else {
                        break;
                    }
                }
            }
            PlayerEffect::RecordDamageDealt { amount } => {
                self.apply_one(&Mutation::SetLastDamageDealt(*amount));
            }
            PlayerEffect::ResetAp => {
                self.apply_one(&Mutation::SetPlayerAp(self.player.max_ap));
            }
            PlayerEffect::AdvanceTurn => {
                self.apply_one(&Mutation::AdvanceTurn);
            }
            PlayerEffect::IncrementWaitCounter => {
                self.apply_one(&Mutation::SetWaitCounter(self.wait_counter + 1));
            }
            PlayerEffect::AllocateStat { stat } => {
                match stat.as_str() {
                    "max_hp" => {
                        self.apply_one(&Mutation::SetPlayerMaxHp(self.player.max_hp + 1));
                        self.apply_one(&Mutation::SetPlayerHp(self.player.hp + 1));
                    }
                    "max_ap" => { self.apply_one(&Mutation::SetPlayerMaxAp(self.player.max_ap + 1)); }
                    "reflex" => { self.apply_one(&Mutation::SetPlayerReflex(self.player.reflex + 1)); }
                    _ => {}
                }
                self.apply_one(&Mutation::SetPlayerStatPoints(self.player.pending_stat_points - 1));
            }
            PlayerEffect::GainSaltScrip { amount } => {
                self.apply_one(&Mutation::SetPlayerSaltScrip(self.player.salt_scrip + amount));
            }
            PlayerEffect::GainSkillPoints { amount } => {
                self.apply_one(&Mutation::SetPlayerSkillPoints(self.player.skills.skill_points + amount));
            }
            PlayerEffect::LevelUp => {
                let points = crate::game::progression::stat_points_per_level();
                self.apply_one(&Mutation::SetPlayerLevel(self.player.level + 1));
                self.apply_one(&Mutation::SetPlayerStatPoints(self.player.pending_stat_points + points));
                self.apply_one(&Mutation::SetPlayerSkillPoints(self.player.skills.skill_points + 2));
            }
            PlayerEffect::ModifyReputation { faction, delta } => {
                let current = self.player.faction_reputation.get(faction.as_str()).copied().unwrap_or(0);
                self.apply_one(&Mutation::SetReputation { faction: faction.clone(), value: current + delta });
            }
            PlayerEffect::ApplyStatusEffect { effect_id, duration } => {
                self.apply_one(&Mutation::AddStatusEffect { id: effect_id.clone(), duration: *duration });
            }
            PlayerEffect::SetPhaseMode { enabled } => {
                self.debug.phase = *enabled;
            }
            PlayerEffect::ClearEncounter => {
                self.apply_one(&Mutation::SetEncounterState(None));
            }
            PlayerEffect::SetLastFleeAttempt { turn } => {
                self.apply_one(&Mutation::SetLastFleeAttempt(*turn));
            }
            PlayerEffect::SetWorldPosition { wx, wy } => {
                self.apply_one(&Mutation::SetWorldPosition { wx: *wx, wy: *wy });
            }
            PlayerEffect::SetLayer { layer } => {
                self.apply_one(&Mutation::SetLayer(*layer));
            }
            PlayerEffect::IncrementTilesTraveled => {
                self.apply_one(&Mutation::IncrementTilesTraveled);
            }
            PlayerEffect::TickPsychic => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Psychic));
            }
            PlayerEffect::TickSkills => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Skills));
            }
            PlayerEffect::TickLightSystem => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Light));
            }
            PlayerEffect::TickVoidSystem => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Void));
            }
            PlayerEffect::TickCrystalSystem => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Crystal));
            }
            PlayerEffect::TickStatusEffects => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Status));
            }
            PlayerEffect::TickHousekeeping => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Housekeeping));
            }
            PlayerEffect::GainAdaptation { adaptation_id } => {
                // Log before delegating so the message appears
                if let Some(a) = crate::game::adaptation::Adaptation::from_id(adaptation_id) {
                    self.log_typed(format!("Gained adaptation: {}", a.name()), MsgType::Status);
                }
                self.apply_one(&Mutation::AddAdaptation(adaptation_id.clone()));
            }
            PlayerEffect::RunAI => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::AI));
            }
        }
    }

    fn apply_combat_effect(&mut self, effect: &CombatEffect) {
        use crate::game::mutations::Mutation;
        match effect {
            CombatEffect::DealDamage { enemy_idx, amount } => {
                if let Some(enemy) = self.world.enemies.get(*enemy_idx) {
                    let new_hp = enemy.hp - amount;
                    self.apply_one(&Mutation::SetEnemyHp { idx: *enemy_idx, hp: new_hp });
                }
            }
            CombatEffect::Miss { .. } => {}
            CombatEffect::Kill { enemy_idx, x, y, .. } => {
                self.apply_one(&Mutation::RemoveEnemy { idx: *enemy_idx, x: *x, y: *y });
            }
            CombatEffect::Provoke { enemy_idx } => {
                self.apply_one(&Mutation::SetEnemyProvoked { idx: *enemy_idx, provoked: true });
            }
            CombatEffect::StunEnemy { enemy_idx, duration } => {
                self.apply_one(&Mutation::AddEnemyStatus { idx: *enemy_idx, id: "stun".into(), duration: *duration });
            }
        }
    }

    fn apply_item_effect(&mut self, effect: &ItemEffect) {
        use crate::game::mutations::Mutation;
        match effect {
            ItemEffect::Consume { inventory_index, .. } => {
                self.apply_one(&Mutation::RemoveFromInventory(*inventory_index));
            }
            ItemEffect::RemoveFromInventory { index } => {
                self.apply_one(&Mutation::RemoveFromInventory(*index));
            }
            ItemEffect::Equip { item_id, slot } => {
                self.apply_one(&Mutation::SetEquipment { slot: slot.clone(), item_id: Some(item_id.clone()) });
            }
            ItemEffect::Unequip { slot } => {
                self.apply_one(&Mutation::SetEquipment { slot: slot.clone(), item_id: None });
            }
            ItemEffect::AddToInventory { item_id } => {
                self.apply_one(&Mutation::AddToInventory(item_id.clone()));
            }
            ItemEffect::SpawnOnMap { item_id, x, y } => {
                self.apply_one(&Mutation::SpawnItemOnMap { item_id: item_id.clone(), x: *x, y: *y });
            }
            ItemEffect::RecalcStats => {
                self.recalc_equipment_stats();
            }
        }
    }

    fn apply_map_effect(&mut self, effect: &MapEffect) {
        use crate::game::mutations::Mutation;
        match effect {
            MapEffect::RevealAll => {
                self.apply_one(&Mutation::RevealAll);
            }
            MapEffect::ClearStormHighlight { tile_index } => {
                self.apply_one(&Mutation::ClearStormHighlight(*tile_index));
            }
            MapEffect::DamageWall { x, y, damage } => {
                let tile_idx = (*y * self.world.map.width as i32 + *x) as usize;
                if tile_idx < self.world.map.tiles.len() {
                    let mut broken = false;
                    if let crate::game::map::Tile::Wall { hp, .. } = &mut self.world.map.tiles[tile_idx] {
                        *hp -= damage;
                        if *hp <= 0 { broken = true; }
                    }
                    if broken {
                        self.apply_one(&Mutation::SetTile {
                            idx: tile_idx,
                            tile: crate::game::map::Tile::default_floor(),
                        });
                        self.update_lighting();
                    }
                }
            }
            MapEffect::SetWorldPath { path, target } => {
                self.apply_one(&Mutation::SetWorldPath { path: path.clone(), target: *target });
            }
            MapEffect::ClearWorldPath => {
                self.apply_one(&Mutation::ClearWorldPath);
            }
            MapEffect::AdvanceTime { new_time } => {
                self.apply_one(&Mutation::SetTimeOfDay(*new_time as u8));
            }
            MapEffect::SetWeather { weather } => {
                let w = match weather.as_str() {
                    "dusty" => crate::game::world_state::Weather::Dusty,
                    "sandstorm" => crate::game::world_state::Weather::Sandstorm,
                    _ => crate::game::world_state::Weather::Clear,
                };
                self.apply_one(&Mutation::SetWeather(w));
            }
            MapEffect::TickEncounterTimer => {
                self.apply_one(&Mutation::IncrementEncounterTimer);
            }
            MapEffect::TickStorm => {
                self.apply_one(&Mutation::TickSubsystem(crate::game::mutations::SubsystemId::Storm));
            }
        }
    }

    fn apply_resource_effect(&mut self, effect: &ResourceEffect) {
        use crate::game::mutations::Mutation;
        match effect {
            ResourceEffect::GainLightEnergy { amount } => {
                self.apply_one(&Mutation::SetLightEnergy(self.player.light_system.light_energy + amount));
            }
            ResourceEffect::GainVoidEnergy { amount } => {
                self.apply_one(&Mutation::AddVoidEnergy(*amount));
            }
            ResourceEffect::GainVoidExposure { amount } => {
                self.apply_one(&Mutation::AddVoidExposure(*amount));
            }
            ResourceEffect::GainResonanceEnergy { amount } => {
                let new_val = self.player.crystal_system.resonance_energy + amount;
                self.apply_one(&Mutation::SetResonanceEnergy(new_val));
            }
            ResourceEffect::PlaceCrystal { x, y, frequency } => {
                self.apply_one(&Mutation::PlaceCrystal { x: *x, y: *y, frequency: frequency.clone() });
            }
        }
    }

    fn apply_event_effect(&mut self, effect: &EventEffect) {
        match effect {
            EventEffect::OpenBook { book_id } => {
                self.pending_ui.book_open = Some(book_id.clone());
            }
            EventEffect::LootDrop { enemy_id, x, y } => {
                let output = crate::game::rules::reactions::reaction_loot_drop(enemy_id, *x, *y, &mut self.rng);
                for effect in &output.effects {
                    self.apply_effect(effect);
                }
                for p in &output.presentation {
                    self.apply_presentation(p);
                }
            }
            EventEffect::QuestNotify { kind } => {
                use super::QuestNotifyKind;
                let completed = match kind {
                    QuestNotifyKind::Kill { enemy_id } => {
                        self.player.quest_log.on_enemy_killed(enemy_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Collect { item_id } => {
                        self.player.quest_log.on_item_collected(item_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Move { x, y } => {
                        self.player.quest_log.on_position_changed(*x, *y);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::NpcTalk { npc_id } => {
                        self.player.quest_log.on_npc_talked(npc_id)
                    }
                    QuestNotifyKind::Interact { target_id } => {
                        self.player.quest_log.on_interact(target_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Examine { target_id } => {
                        self.player.quest_log.on_examine(target_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::AriaInterface { item_id } => {
                        self.player.quest_log.on_aria_interfaced(item_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Turn => {
                        self.player.quest_log.on_turn_passed();
                        self.player.quest_log.check_auto_complete()
                    }
                };
                self.log_quest_completions(&completed);
            }
        }
    }

    fn apply_quest_effect(&mut self, effect: &QuestEffect) {
        match effect {
            QuestEffect::Accept { quest_id } => {
                if let Some(quest) = crate::game::quest::ActiveQuest::new(quest_id) {
                    self.player.quest_log.active.push(quest);
                }
            }
            QuestEffect::Complete { quest_id } => {
                let _ = self.player.quest_log.complete(quest_id);
            }
            QuestEffect::SetFactionAlignment { faction } => {
                self.player.quest_log.set_faction_alignment(faction);
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
