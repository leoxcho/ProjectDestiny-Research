//! Simulated client and end-to-end test harness. It does not claim wire compatibility.
use anyhow::{anyhow,Result};
use destiny_network::{decode,Direction};
use destiny_runtime_core::Definition;
use destiny_server_runtime::{DispatchResult,ServerRuntime};
use destiny_storage::Player;
use serde::{Deserialize,Serialize};
use std::sync::{Arc,Mutex};
use std::time::{SystemTime,UNIX_EPOCH};

#[derive(Default)] pub struct TestClient{pub connected:bool,pub session_id:Option<String>,pub account_id:Option<String>,pub server:Option<Arc<Mutex<ServerRuntime>>>}
#[derive(Debug,Clone,Serialize,Deserialize)] pub struct ActivityView{pub id:String,pub state:String}
impl TestClient{
 pub fn connect(&mut self,server:Arc<Mutex<ServerRuntime>>)->Result<()> {self.server=Some(server);self.connected=true;Ok(())}
 pub fn create_session(&mut self,account_id:&str)->Result<String>{if !self.connected{return Err(anyhow!("client is not connected"))}let id=format!("test-session-{account_id}");self.account_id=Some(account_id.into());self.session_id=Some(id.clone());Ok(id)}
 pub fn authenticate(&mut self)->Result<()> {let s=self.session_id.clone().ok_or_else(||anyhow!("session not created"))?;let a=self.account_id.clone().unwrap();let mut r=self.server.as_ref().unwrap().lock().unwrap();r.auth.authenticate_placeholder(destiny_auth_service::Account{id:a,display_name:"simulated-client".into()},s.clone(),std::time::Duration::from_secs(300));r.record_session(&s,"authenticated");Ok(())}
 pub fn request_definition(&self,hash:&str)->Result<Option<Definition>>{let r=self.server.as_ref().ok_or_else(||anyhow!("not connected"))?.lock().unwrap();r.inventory.item_definition(hash)}
 pub fn request_inventory(&self)->Result<Player>{let id=self.account_id.as_ref().ok_or_else(||anyhow!("not authenticated"))?;let r=self.server.as_ref().unwrap().lock().unwrap();Ok(r.inventory.players.get(id).cloned().unwrap_or_else(||Player{id:id.clone(),..Default::default()}))}
 pub fn request_activity(&self)->Result<Vec<ActivityView>>{let r=self.server.as_ref().unwrap().lock().unwrap();Ok(r.activities.instances.iter().map(|a|ActivityView{id:a.id.clone(),state:format!("{:?}",a.state)}).collect())}
 pub fn update_world_state(&self,zone:&str)->Result<()>{let id=self.account_id.as_ref().ok_or_else(||anyhow!("not authenticated"))?.clone();let mut r=self.server.as_ref().unwrap().lock().unwrap();r.world.state.player_locations.insert(id,zone.into());Ok(())}
 pub fn dispatch_unknown_packet(&self,payload:Vec<u8>)->Result<DispatchResult>{let mut r=self.server.as_ref().unwrap().lock().unwrap();let mut p=decode(payload);p.direction=Direction::ClientToServer;p.session_id=self.session_id.clone().unwrap_or_default();Ok(r.dispatch(p))}
}
pub fn now_ms()->u128{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()}

#[cfg(test)]mod tests{use super::*;use destiny_activity_service::ActivityState;use destiny_runtime_core::Runtime;use destiny_server_runtime::ServerRuntime;#[test]fn client_to_runtime_services_and_database(){let db=Arc::new(Runtime::open(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../destiny.db")).unwrap());let server=Arc::new(Mutex::new(ServerRuntime::new(db)));let mut c=TestClient::default();c.connect(server.clone()).unwrap();c.create_session("player-1").unwrap();c.authenticate().unwrap();assert!(c.request_definition("80802797").unwrap().is_some());assert!(c.request_inventory().is_ok());{let mut r=server.lock().unwrap();r.activities.create("activity-1".into(),"80802797".into());r.activities.instances[0].state=ActivityState::Completed;}assert_eq!(c.request_activity().unwrap()[0].state,"Completed");c.update_world_state("unknown-zone").unwrap();assert_eq!(server.lock().unwrap().world.state.player_locations["player-1"],"unknown-zone");assert!(matches!(c.dispatch_unknown_packet(vec![1,2,3]).unwrap(),DispatchResult::UnknownOpcode));let t=&server.lock().unwrap().telemetry;assert!(!t.request_log.is_empty());assert!(!t.session_history.is_empty());}}
