use crate::screens::{DashboardScreen, LoginScreen, SplashScreen};
use gpui_kit::{AnyElement, Entity, IntoElement, component::IconName};

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

impl Route {
    pub fn id(self) -> &'static str {
        match self {
            Self::SingleBill => "nav-single-bill",
            Self::BulkBill => "nav-bulk-bills",
            Self::Settings => "nav-settings",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::SingleBill => "Single bill",
            Self::BulkBill => "Bulk bills",
            Self::Settings => "Settings",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::SingleBill => "Scan one barcode and the bill form fills itself.",
            Self::BulkBill => "Scan a stack of bills and review them together.",
            Self::Settings => "Your account, devices and preferences.",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Self::SingleBill => IconName::FileText,
            Self::BulkBill => IconName::Copy,
            Self::Settings => IconName::Settings,
        }
    }
}
