//! org.freedesktop.login1.Manager and related interfaces

use log::info;
use std::collections::HashMap;
use zbus::{interface, Result as ZResult};
use zvariant::{OwnedObjectPath, OwnedValue, Value};

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub uid: u32,
    pub user: String,
    pub seat: String,
    pub state: String,
    pub type_: String,
    pub class: String,
    pub display: String,
    pub tty: String,
    pub remote: bool,
    pub timestamp: u64,
}

pub struct LogindManager {
    sessions: HashMap<String, Session>,
    users: HashMap<u32, String>,
    seats: HashMap<String, Vec<String>>,
    next_session: u32,
}

impl LogindManager {
    pub fn new() -> Self {
        let mut seats = HashMap::new();
        seats.insert("seat0".to_string(), vec![]);
        LogindManager {
            sessions: HashMap::new(),
            users: HashMap::new(),
            seats,
            next_session: 1,
        }
    }
}

#[interface(name = "org.freedesktop.login1.Manager")]
impl LogindManager {
    async fn create_session(
        &mut self,
        uid: u32,
        pid: u32,
        service: &str,
        type_: &str,
        class: &str,
        desktop: &str,
        seat_id: &str,
        vtnr: u32,
        tty: &str,
        display: &str,
        remote: bool,
        remote_user: &str,
        remote_host: &str,
        flags: Vec<String>,
    ) -> ZResult<(String, OwnedObjectPath, String, OwnedObjectPath, String, OwnedObjectPath)> {
        let sid = format!("c{}", self.next_session);
        self.next_session += 1;
        info!("CreateSession uid={} seat={} type={}", uid, seat_id, type_);

        let session = Session {
            id: sid.clone(),
            uid,
            user: format!("user{}", uid),
            seat: seat_id.to_string(),
            state: "active".into(),
            type_: type_.to_string(),
            class: class.to_string(),
            display: display.to_string(),
            tty: tty.to_string(),
            remote,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
        };
        self.sessions.insert(sid.clone(), session);
        self.users.insert(uid, format!("user{}", uid));

        if let Some(seat_sessions) = self.seats.get_mut(seat_id) {
            seat_sessions.push(sid.clone());
        }

        let session_path = OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", sid)).unwrap();
        let user_path = OwnedObjectPath::try_from(format!("/org/freedesktop/login1/user/_{}", uid)).unwrap();
        let seat_path = OwnedObjectPath::try_from("/org/freedesktop/login1/seat/seat0").unwrap();

        Ok((
            sid,
            session_path,
            format!("/run/user/{}", uid),
            user_path,
            seat_id.to_string(),
            seat_path,
        ))
    }

    async fn release_session(&mut self, session_id: &str) -> ZResult<()> {
        info!("ReleaseSession {}", session_id);
        self.sessions.remove(session_id);
        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> ZResult<OwnedObjectPath> {
        Ok(OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", session_id)).unwrap())
    }

    async fn list_sessions(&self) -> ZResult<Vec<(String, u32, String, String, OwnedObjectPath)>> {
        let mut out = Vec::new();
        for (id, s) in &self.sessions {
            out.push((
                id.clone(),
                s.uid,
                s.user.clone(),
                s.seat.clone(),
                OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", id)).unwrap(),
            ));
        }
        Ok(out)
    }

    async fn get_user(&self, uid: u32) -> ZResult<OwnedObjectPath> {
        Ok(OwnedObjectPath::try_from(format!("/org/freedesktop/login1/user/_{}", uid)).unwrap())
    }

    async fn list_users(&self) -> ZResult<Vec<(u32, String, OwnedObjectPath)>> {
        let mut out = Vec::new();
        for (uid, name) in &self.users {
            out.push((
                *uid,
                name.clone(),
                OwnedObjectPath::try_from(format!("/org/freedesktop/login1/user/_{}", uid)).unwrap(),
            ));
        }
        Ok(out)
    }

    async fn get_seat(&self, seat_id: &str) -> ZResult<OwnedObjectPath> {
        Ok(OwnedObjectPath::try_from(format!("/org/freedesktop/login1/seat/{}", seat_id)).unwrap())
    }

    async fn list_seats(&self) -> ZResult<Vec<(String, OwnedObjectPath)>> {
        Ok(vec![(
            "seat0".into(),
            OwnedObjectPath::try_from("/org/freedesktop/login1/seat/seat0").unwrap(),
        )])
    }

    async fn activate_session(&self, _session_id: &str) -> ZResult<()> {
        Ok(())
    }

    async fn lock_session(&self, _session_id: &str) -> ZResult<()> {
        Ok(())
    }

    async fn unlock_session(&self, _session_id: &str) -> ZResult<()> {
        Ok(())
    }

    async fn set_idle_hint(&self, _idle: bool) -> ZResult<()> {
        Ok(())
    }

    async fn can_power_off(&self) -> ZResult<String> {
        Ok("yes".into())
    }

    async fn power_off(&self, _interactive: bool) -> ZResult<()> {
        info!("PowerOff requested via logind");
        let _ = std::fs::write("/run/azurewrath.ctl", "0\n");
        Ok(())
    }

    async fn reboot(&self, _interactive: bool) -> ZResult<()> {
        info!("Reboot requested via logind");
        let _ = std::fs::write("/run/azurewrath.ctl", "6\n");
        Ok(())
    }

    async fn suspend(&self, _interactive: bool) -> ZResult<()> {
        Ok(())
    }

    async fn hibernate(&self, _interactive: bool) -> ZResult<()> {
        Ok(())
    }

    async fn inhibit(
        &self,
        _what: &str,
        _who: &str,
        _why: &str,
        _mode: &str,
    ) -> ZResult<zvariant::OwnedFd> {
        // Return a dummy fd
        let (r, _w) = nix::unistd::pipe()?;
        Ok(zvariant::OwnedFd::from(r))
    }
}

// Session object properties
pub struct SessionObject {
    session: Session,
}

#[interface(name = "org.freedesktop.login1.Session")]
impl SessionObject {
    #[zbus(property)]
    async fn id(&self) -> String {
        self.session.id.clone()
    }

    #[zbus(property)]
    async fn user(&self) -> (u32, OwnedObjectPath) {
        (
            self.session.uid,
            OwnedObjectPath::try_from(format!("/org/freedesktop/login1/user/_{}", self.session.uid)).unwrap(),
        )
    }

    #[zbus(property)]
    async fn seat(&self) -> (String, OwnedObjectPath) {
        (
            self.session.seat.clone(),
            OwnedObjectPath::try_from("/org/freedesktop/login1/seat/seat0").unwrap(),
        )
    }

    #[zbus(property)]
    async fn state(&self) -> String {
        self.session.state.clone()
    }

    #[zbus(property)]
    async fn type_(&self) -> String {
        self.session.type_.clone()
    }

    #[zbus(property)]
    async fn class(&self) -> String {
        self.session.class.clone()
    }

    #[zbus(property)]
    async fn display(&self) -> String {
        self.session.display.clone()
    }

    #[zbus(property)]
    async fn tty(&self) -> String {
        self.session.tty.clone()
    }

    #[zbus(property)]
    async fn remote(&self) -> bool {
        self.session.remote
    }

    #[zbus(property)]
    async fn timestamp(&self) -> u64 {
        self.session.timestamp
    }
}
