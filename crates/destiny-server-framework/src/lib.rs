//! Server lifecycle skeleton; authentication and game protocol are not implemented.
use destiny_storage::Player; use std::collections::HashMap; use std::sync::{Arc,Mutex};
pub struct ConnectionManager{pub connections:Arc<Mutex<HashMap<String,String>>>} impl Default for ConnectionManager{fn default()->Self{Self{connections:Default::default()}}}
#[derive(Default)]pub struct SessionManager{pub sessions:HashMap<String,String>}
#[derive(Default)]pub struct PlayerStateModel{pub players:HashMap<String,Player>}
pub trait Service{fn name(&self)->&'static str;}
macro_rules! service{($n:ident)=>{pub struct $n;impl Service for $n{fn name(&self)->&'static str{stringify!($n)}}};}
service!(AuthService);service!(InventoryService);service!(ActivityService);service!(WorldService);
pub struct ServiceRouter{pub services:Vec<Box<dyn Service>>}impl Default for ServiceRouter{fn default()->Self{Self{services:vec![Box::new(AuthService),Box::new(InventoryService),Box::new(ActivityService),Box::new(WorldService)]}}}
#[cfg(test)]mod tests{use super::*;#[test]fn routes_placeholders(){assert_eq!(ServiceRouter::default().services.len(),4);}}
