use crate::game::adaptation::Adaptation;
use crate::game::enemy::Enemy;
use crate::game::item::{ItemDef, get_item_def};
use crate::game::map::Map;
use crate::game::player_state::PlayerState;
use crate::game::state::GameState;
use std::collections::{HashMap, HashSet};

/// Read-only view of game state for rule functions.
pub struct QueryContext<'a> {
    pub player: &'a PlayerState,
    pub map: &'a Map,
    pub revealed_count: usize,
    pub tile_count: usize,
    pub npc_positions: &'a HashMap<(i32, i32), usize>,
    pub enemy_positions: &'a HashMap<(i32, i32), usize>,
    pub enemies: &'a [Enemy],
    pub visible: &'a HashSet<usize>,
    pub debug_phase: bool,
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
}

impl<'a> QueryContext<'a> {
    pub fn from_state(state: &'a GameState) -> Self {
        Self {
            player: &state.player,
            map: &state.world.map,
            revealed_count: state.revealed.len(),
            tile_count: state.world.map.tiles.len(),
            npc_positions: &state.spatial.npc_positions,
            enemy_positions: &state.spatial.enemy_positions,
            enemies: &state.world.enemies,
            visible: &state.visible,
            debug_phase: state.debug.phase,
            mock_combat_hit: state.debug.mock_combat_hit,
            mock_combat_damage: state.debug.mock_combat_damage,
        }
    }

    pub fn item_def(&self, id: &str) -> Option<&'static ItemDef> {
        get_item_def(id)
    }

    pub fn has_npc_at(&self, x: i32, y: i32) -> bool {
        self.npc_positions.contains_key(&(x, y))
    }

    pub fn has_enemy_at(&self, x: i32, y: i32) -> bool {
        self.enemy_positions.contains_key(&(x, y))
    }

    pub fn enemy_idx_at(&self, x: i32, y: i32) -> Option<usize> {
        self.enemy_positions.get(&(x, y)).copied()
    }

    pub fn enemy(&self, idx: usize) -> Option<&'a Enemy> {
        self.enemies.get(idx)
    }

    pub fn has_adaptation(&self, adaptation: &Adaptation) -> bool {
        self.player.adaptations.contains(adaptation)
    }
}

// ---------------------------------------------------------------------------
// TestContext — lightweight builder for rule unit tests (no GameState needed)
// ---------------------------------------------------------------------------

pub struct TestContext {
    pub player: PlayerState,
    pub map: Map,
    pub revealed: HashSet<usize>,
    pub npc_positions: HashMap<(i32, i32), usize>,
    pub enemy_positions: HashMap<(i32, i32), usize>,
    pub enemies: Vec<Enemy>,
    pub visible: HashSet<usize>,
    pub debug_phase: bool,
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            player: PlayerState::new(),
            map: Map::new(20, 20),
            revealed: HashSet::new(),
            npc_positions: HashMap::new(),
            enemy_positions: HashMap::new(),
            enemies: Vec::new(),
            visible: HashSet::new(),
            debug_phase: false,
            mock_combat_hit: None,
            mock_combat_damage: None,
        }
    }

    pub fn with_player_hp(mut self, hp: i32) -> Self {
        self.player.hp = hp;
        self
    }

    pub fn with_player_max_hp(mut self, max_hp: i32) -> Self {
        self.player.max_hp = max_hp;
        self
    }

    pub fn with_player_ap(mut self, ap: i32) -> Self {
        self.player.ap = ap;
        self
    }

    pub fn with_inventory(mut self, items: Vec<String>) -> Self {
        self.player.inventory = items;
        self
    }

    pub fn with_player_refraction(mut self, refraction: u32) -> Self {
        self.player.refraction = refraction;
        self
    }

    pub fn with_player_position(mut self, x: i32, y: i32) -> Self {
        self.player.x = x;
        self.player.y = y;
        self
    }

    pub fn with_floor_at(mut self, x: i32, y: i32) -> Self {
        let idx = y as usize * self.map.width + x as usize;
        if idx < self.map.tiles.len() {
            self.map.tiles[idx] = crate::game::map::Tile::default_floor();
        }
        self
    }

    pub fn with_tile_at(mut self, x: i32, y: i32, tile: crate::game::map::Tile) -> Self {
        let idx = y as usize * self.map.width + x as usize;
        if idx < self.map.tiles.len() {
            self.map.tiles[idx] = tile;
        }
        self
    }

    pub fn with_npc_at(mut self, x: i32, y: i32, index: usize) -> Self {
        self.npc_positions.insert((x, y), index);
        self
    }

    pub fn with_enemy_at(mut self, x: i32, y: i32, index: usize) -> Self {
        self.enemy_positions.insert((x, y), index);
        self
    }

    pub fn with_enemy(mut self, enemy: Enemy) -> Self {
        let idx = self.enemies.len();
        self.enemy_positions.insert((enemy.x, enemy.y), idx);
        self.enemies.push(enemy);
        self
    }

    pub fn with_mock_combat_hit(mut self, hit: bool) -> Self {
        self.mock_combat_hit = Some(hit);
        self
    }

    pub fn with_mock_combat_damage(mut self, dmg: i32) -> Self {
        self.mock_combat_damage = Some(dmg);
        self
    }

    pub fn with_visible(mut self, x: i32, y: i32) -> Self {
        let idx = y as usize * self.map.width + x as usize;
        self.visible.insert(idx);
        self
    }

    pub fn build(&self) -> QueryContext<'_> {
        QueryContext {
            player: &self.player,
            map: &self.map,
            revealed_count: self.revealed.len(),
            tile_count: self.map.tiles.len(),
            npc_positions: &self.npc_positions,
            enemy_positions: &self.enemy_positions,
            enemies: &self.enemies,
            visible: &self.visible,
            debug_phase: self.debug_phase,
            mock_combat_hit: self.mock_combat_hit,
            mock_combat_damage: self.mock_combat_damage,
        }
    }
}
