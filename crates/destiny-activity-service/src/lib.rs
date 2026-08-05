use destiny_runtime_core::{Definition,Runtime};use serde::{Deserialize,Serialize};use std::sync::Arc;
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]pub enum ActivityState{Matchmaking,Active,Completed,Unknown}
#[derive(Clone,Debug,Serialize,Deserialize)]pub struct ActivityInstance{pub id:String,pub definition_hash:String,pub players:Vec<String>,pub state:ActivityState,pub rewards:Vec<String>}
pub struct ActivityService{pub runtime:Arc<Runtime>,pub instances:Vec<ActivityInstance>}
impl ActivityService{pub fn new(runtime:Arc<Runtime>)->Self{Self{runtime,instances:vec![]}}pub fn definition(&self,h:&str)->anyhow::Result<Option<Definition>>{self.runtime.get_definition(h)}pub fn create(&mut self,id:String,h:String)->&ActivityInstance{self.instances.push(ActivityInstance{id,definition_hash:h,players:vec![],state:ActivityState::Unknown,rewards:vec![]});self.instances.last().unwrap()} }
