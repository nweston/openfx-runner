use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize)]
pub struct Timer {
    #[serde(rename = "timer")]
    name: String,
    durations: HashMap<String, Duration>,
}

impl Timer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            durations: HashMap::new(),
        }
    }

    /// Add time to the total for a given action
    pub fn record_action(&mut self, action_name: &str, duration: Duration) {
        let d = self.durations.entry(action_name.to_string()).or_default();
        *d += duration;
    }
}
