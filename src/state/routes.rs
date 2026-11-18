use crate::screens::{DashboardScreen, LoginScreen, SplashScreen};
use gpui_kit::{AnyElement, Entity, IntoElement};

#[derive(Debug, Clone)]
pub enum Screen {
    Splash(Entity<SplashScreen>),
    Auth(Entity<LoginScreen>),
    Dashboard(Entity<DashboardScreen>),
}

impl Screen {
    pub fn element(&self) -> AnyElement {
        match self {
            Self::Auth(s) => s.clone().into_any_element(),
            Self::Splash(s) => s.clone().into_any_element(),
            Self::Dashboard(s) => s.clone().into_any_element(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    SingleBill,
    BulkBill,
    Settings,
}
