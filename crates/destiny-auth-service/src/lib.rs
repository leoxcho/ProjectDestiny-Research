use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub display_name: String,
}
#[derive(Clone, Debug, PartialEq)]
pub enum AuthState {
    Unauthenticated,
    Authenticating,
    Authenticated,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub account: Account,
    pub state: AuthState,
    pub expires_at: Instant,
}
#[derive(Default)]
pub struct AuthService {
    sessions: HashMap<String, Session>,
}
impl AuthService {
    pub fn authenticate_placeholder(
        &mut self,
        account: Account,
        session_id: String,
        ttl: Duration,
    ) -> Session {
        let s = Session {
            id: session_id.clone(),
            account,
            state: AuthState::Authenticated,
            expires_at: Instant::now() + ttl,
        };
        self.sessions.insert(session_id, s.clone());
        s
    }
    pub fn validate(&mut self, id: &str) -> Option<&Session> {
        if self
            .sessions
            .get(id)
            .map(|s| s.expires_at <= Instant::now())
            .unwrap_or(false)
        {
            if let Some(s) = self.sessions.get_mut(id) {
                s.state = AuthState::Expired;
            }
            return None;
        }
        self.sessions.get(id)
    }
    pub fn expire(&mut self) {
        let now = Instant::now();
        for s in self.sessions.values_mut() {
            if s.expires_at <= now {
                s.state = AuthState::Expired
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholder_session_validates() {
        let mut a = AuthService::default();
        let s = a.authenticate_placeholder(
            Account {
                id: "a".into(),
                display_name: "mock".into(),
            },
            "s".into(),
            Duration::from_secs(60),
        );
        assert_eq!(a.validate(&s.id).unwrap().state, AuthState::Authenticated);
    }
}
