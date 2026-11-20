//! Extended logind interfaces: Seat and User objects

use super::Session;
use zbus::interface;
use zvariant::OwnedObjectPath;

pub struct SeatObject {
    pub id: String,
    pub active_session: Option<String>,
    pub can_multi_session: bool,
    pub can_tty: bool,
    pub can_graphical: bool,
}

#[interface(name = "org.freedesktop.login1.Seat")]
impl SeatObject {
    #[zbus(property)]
    async fn id(&self) -> String {
        self.id.clone()
    }

    #[zbus(property)]
    async fn active_session(&self) -> (String, OwnedObjectPath) {
        let sid = self.active_session.clone().unwrap_or_default();
        let path = if sid.is_empty() {
            OwnedObjectPath::try_from("/").unwrap()
        } else {
            OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", sid)).unwrap()
        };
        (sid, path)
    }

    #[zbus(property)]
    async fn can_multi_session(&self) -> bool {
        self.can_multi_session
    }

    #[zbus(property)]
    async fn can_tty(&self) -> bool {
        self.can_tty
    }

    #[zbus(property)]
    async fn can_graphical(&self) -> bool {
        self.can_graphical
    }

    #[zbus(property)]
    async fn sessions(&self) -> Vec<(String, OwnedObjectPath)> {
        vec![]
    }
}

pub struct UserObject {
    pub uid: u32,
    pub name: String,
    pub state: String,
    pub display: Option<String>,
    pub sessions: Vec<String>,
}

#[interface(name = "org.freedesktop.login1.User")]
impl UserObject {
    #[zbus(property)]
    async fn uid(&self) -> u32 {
        self.uid
    }

    #[zbus(property)]
    async fn name(&self) -> String {
        self.name.clone()
    }

    #[zbus(property)]
    async fn state(&self) -> String {
        self.state.clone()
    }

    #[zbus(property)]
    async fn display(&self) -> (String, OwnedObjectPath) {
        let d = self.display.clone().unwrap_or_default();
        let path = if d.is_empty() {
            OwnedObjectPath::try_from("/").unwrap()
        } else {
            OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", d)).unwrap()
        };
        (d, path)
    }

    #[zbus(property)]
    async fn sessions(&self) -> Vec<(String, OwnedObjectPath)> {
        self.sessions
            .iter()
            .map(|s| {
                (
                    s.clone(),
                    OwnedObjectPath::try_from(format!("/org/freedesktop/login1/session/{}", s))
                        .unwrap(),
                )
            })
            .collect()
    }

    #[zbus(property)]
    async fn idle_hint(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn idle_since_hint(&self) -> u64 {
        0
    }
}
