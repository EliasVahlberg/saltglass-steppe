use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Storm { pub turns_until: u32, pub intensity: u8 }

impl Storm {
    pub fn forecast(rng: &mut ChaCha8Rng) -> Self {
        Self { turns_until: rng.gen_range(15..30), intensity: rng.gen_range(1..4) }
    }
    pub fn tick(&mut self) -> bool {
        if self.turns_until > 0 { self.turns_until -= 1; self.turns_until == 0 } else { false }
    }
}
