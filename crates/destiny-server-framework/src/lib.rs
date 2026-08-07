//! Server lifecycle skeleton; authentication and game protocol are not implemented.
use destiny_storage::Player;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
pub struct ConnectionManager {
    pub connections: Arc<Mutex<HashMap<String, String>>>,
}
impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            connections: Default::default(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Connected,
    WaitingHandshake,
    Authenticating,
    Ready,
    Closed,
}
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub peer: String,
    pub state: SessionState,
    pub created_ms: u128,
    pub last_seen_ms: u128,
    pub tcp_bytes: u64,
    pub udp_bytes: u64,
}
#[derive(Default)]
pub struct SessionManager {
    pub sessions: HashMap<String, Session>,
}
impl SessionManager {
    pub fn upsert(&mut self, id: impl Into<String>, peer: impl Into<String>) -> &mut Session {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let id = id.into();
        self.sessions.entry(id.clone()).or_insert(Session {
            id,
            peer: peer.into(),
            state: SessionState::Connected,
            created_ms: now,
            last_seen_ms: now,
            tcp_bytes: 0,
            udp_bytes: 0,
        })
    }
    pub fn transition(&mut self, id: &str, state: SessionState) {
        if let Some(s) = self.sessions.get_mut(id) {
            s.state = state;
            s.last_seen_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
        }
    }
    pub fn close(&mut self, id: &str) {
        self.transition(id, SessionState::Closed);
    }
}
#[derive(Default)]
pub struct PlayerStateModel {
    pub players: HashMap<String, Player>,
}
pub trait Service {
    fn name(&self) -> &'static str;
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEndpoint {
    pub name: String,
    pub endpoint: String,
    pub transport: String,
    pub status: String,
}
#[derive(Default)]
pub struct ServiceRegistry {
    pub endpoints: HashMap<String, ServiceEndpoint>,
}
impl ServiceRegistry {
    pub fn register(
        &mut self,
        name: impl Into<String>,
        endpoint: impl Into<String>,
        transport: impl Into<String>,
        status: impl Into<String>,
    ) {
        let name = name.into();
        self.endpoints.insert(
            name.clone(),
            ServiceEndpoint {
                name,
                endpoint: endpoint.into(),
                transport: transport.into(),
                status: status.into(),
            },
        );
    }
    pub fn observe(
        &mut self,
        name: impl Into<String>,
        endpoint: impl Into<String>,
        transport: impl Into<String>,
    ) {
        self.register(name, endpoint, transport, "observed");
    }
}
macro_rules! service {
    ($n:ident) => {
        pub struct $n;
        impl Service for $n {
            fn name(&self) -> &'static str {
                stringify!($n)
            }
        }
    };
}
service!(AuthenticationService);
service!(SessionService);
service!(WorldService);
service!(InventoryService);
service!(ActivityService);
service!(ConfigurationService);
pub struct ServiceRouter {
    pub services: Vec<Box<dyn Service>>,
}
impl Default for ServiceRouter {
    fn default() -> Self {
        Self {
            services: vec![
                Box::new(AuthenticationService),
                Box::new(SessionService),
                Box::new(WorldService),
                Box::new(InventoryService),
                Box::new(ActivityService),
                Box::new(ConfigurationService),
            ],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_all_evidence_gated_services() {
        assert_eq!(ServiceRouter::default().services.len(), 6);
        assert_eq!(
            ServiceRouter::default().services[0].name(),
            "AuthenticationService"
        );
    }
}
#[cfg(test)]
mod session_tests {
    use super::*;
    #[test]
    fn session_lifecycle_is_explicit() {
        let mut m = SessionManager::default();
        m.upsert("c1", "127.0.0.1:1");
        m.transition("c1", SessionState::WaitingHandshake);
        assert_eq!(m.sessions["c1"].state, SessionState::WaitingHandshake);
        m.close("c1");
        assert_eq!(m.sessions["c1"].state, SessionState::Closed);
    }
}
