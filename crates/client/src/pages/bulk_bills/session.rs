use super::*;

pub(super) struct Session {
    pub(super) key: String,
    pub(super) name: SharedString,
    pub(super) created_at: i64,
    pub(super) updated_at: i64,
    pub(super) bills: Entity<BillList>,
    pub(super) _subscription: Subscription,
}

impl Session {
    pub(super) fn to_record(&self, cx: &App) -> BulkSession {
        let bills = self.bills.read(cx);
        BulkSession {
            key: self.key.clone(),
            name: self.name.to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            cursor: Some(bills.cursor() as u32),
            bills: bills.to_stored(),
        }
    }
}

pub(super) struct InlineEditor {
    pub(super) index: usize,
    pub(super) input: Entity<InputState>,
    pub(super) _subscription: Subscription,
}

pub(super) struct DeleteConfirm {
    pub(super) input: Entity<InputState>,
    pub(super) _subscription: Subscription,
}
