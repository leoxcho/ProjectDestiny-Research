//! Serializable player/game state model. Persistence backend is intentionally deferred.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Player {
    pub id: String,
    pub inventory: Vec<InventoryItem>,
    pub characters: Vec<Character>,
    pub progression: Progression,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InventoryItem {
    pub item_hash: String,
    pub quantity: u32,
    pub state: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Character {
    pub id: String,
    pub class: String,
    pub level: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Progression {
    pub level: u32,
    pub score: u64,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_round_trip() {
        let p = Player {
            id: "p".into(),
            ..Default::default()
        };
        assert_eq!(
            serde_json::from_str::<Player>(&serde_json::to_string(&p).unwrap()).unwrap(),
            p
        );
    }
}
