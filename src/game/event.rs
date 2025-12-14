/// Game events for decoupled communication between systems
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: String },
    PlayerHealed { amount: i32 },
    EnemyKilled { enemy_id: String, x: i32, y: i32 },
    ItemPickedUp { item_id: String },
    ItemUsed { item_id: String },
    AdaptationGained { name: String },
    StormArrived { intensity: u8 },
}
