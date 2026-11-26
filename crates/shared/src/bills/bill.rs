use chrono::NaiveDate;
use gpui_shared_string::SharedString;

use super::BillFormat;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bill {
    pub format: SharedString,
    pub label: SharedString,
    pub reference: SharedString,
    pub company: Option<SharedString>,
    pub amount: u32,
    pub late_fee: bool,
}

impl Bill {
    pub fn parse(raw: &str, today: NaiveDate) -> Option<Self> {
        BillFormat::all()
            .iter()
            .find_map(|format| format.parse(raw, today))
    }
}
