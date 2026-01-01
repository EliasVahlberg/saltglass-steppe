use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightedEntry<T> {
    pub item: T,
    pub weight: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightedTable<T> {
    pub entries: Vec<WeightedEntry<T>>,
}

impl<T: Clone> WeightedTable<T> {
    pub fn new(entries: Vec<WeightedEntry<T>>) -> Self {
        Self { entries }
    }

    pub fn select<R: Rng>(&self, rng: &mut R) -> Option<T> {
        if self.entries.is_empty() {
            return None;
        }

        let total_weight: f32 = self.entries.iter().map(|e| e.weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut roll = rng.gen_range(0.0..total_weight);
        
        for entry in &self.entries {
            if roll < entry.weight {
                return Some(entry.item.clone());
            }
            roll -= entry.weight;
        }

        // Fallback to last entry due to floating point precision
        self.entries.last().map(|e| e.item.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_weight(&self) -> f32 {
        self.entries.iter().map(|e| e.weight).sum()
    }
}
