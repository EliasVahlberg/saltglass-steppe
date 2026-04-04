use super::Effect;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub turn: u32,
    pub source: TraceSource,
    pub effect: Effect,
}

#[derive(Debug, Clone)]
pub enum TraceSource {
    Rule { name: &'static str },
    Reaction { name: &'static str, trigger: Box<Effect> },
}

#[derive(Clone, Default)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
    pub enabled: bool,
}

impl Trace {
    pub fn record(&mut self, effect: &Effect, source: TraceSource, turn: u32) {
        if self.enabled {
            self.entries.push(TraceEntry {
                turn,
                source,
                effect: effect.clone(),
            });
        }
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        self.entries.iter().any(|e| &e.effect == effect)
    }
}

impl fmt::Debug for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Trace")
            .field("enabled", &self.enabled)
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(
                f,
                "  Turn {}: [{:?}] {:?}",
                entry.turn, entry.source, entry.effect
            )?;
        }
        Ok(())
    }
}
