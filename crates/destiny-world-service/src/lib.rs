use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,Serialize,Deserialize)]pub struct Zone{pub id:String,pub definition_hash:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]pub struct WorldObject{pub id:String,pub definition_hash:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize,Default)]pub struct WorldState{pub zones:Vec<Zone>,pub player_locations:std::collections::HashMap<String,String>,pub objects:Vec<WorldObject>,pub events:Vec<String>,pub activity_state:std::collections::HashMap<String,String>}
/// World transitions are intentionally absent until observed behavior is documented.
#[derive(Default)]pub struct WorldService{pub state:WorldState}
