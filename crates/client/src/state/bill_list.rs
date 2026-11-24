use gpui_kit::{Context, EventEmitter, SharedString};
use shared::Bill;

use crate::database::StoredBill;

const SOURCE_SCANNED: &str = "scanned";
const SOURCE_MANUAL: &str = "manual";

#[derive(Clone, Debug)]
pub enum Source {
    Scanned(Bill),
    Manual,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub reference: SharedString,
    pub source: Source,
    pub amount: Option<u32>,
    pub paid: bool,
}

impl Entry {
    pub fn scanned(bill: Bill) -> Self {
        Self {
            reference: bill.reference.clone(),
            amount: Some(bill.amount),
            source: Source::Scanned(bill),
            paid: false,
        }
    }

    pub fn manual(reference: SharedString) -> Self {
        Self {
            reference,
            source: Source::Manual,
            amount: None,
            paid: false,
        }
    }

    pub fn is_scanned(&self) -> bool {
        matches!(self.source, Source::Scanned(_))
    }

    pub fn edited(&self) -> bool {
        match &self.source {
            Source::Scanned(bill) => self.amount != Some(bill.amount),
            Source::Manual => false,
        }
    }

    pub fn to_stored(&self) -> StoredBill {
        let (source, format, label, late_fee, original_amount) = match &self.source {
            Source::Scanned(bill) => (
                SOURCE_SCANNED,
                Some(bill.format.to_string()),
                Some(bill.label.to_string()),
                bill.late_fee,
                Some(bill.amount),
            ),
            Source::Manual => (SOURCE_MANUAL, None, None, false, None),
        };
        StoredBill {
            reference: self.reference.to_string(),
            source: source.into(),
            format,
            label,
            late_fee,
            original_amount,
            amount: self.amount,
            paid: Some(self.paid),
        }
    }

    pub fn from_stored(stored: StoredBill) -> Self {
        let reference: SharedString = stored.reference.into();
        let source = if stored.source == SOURCE_SCANNED {
            Source::Scanned(Bill {
                format: stored.format.unwrap_or_default().into(),
                label: stored.label.unwrap_or_default().into(),
                reference: reference.clone(),
                amount: stored.original_amount.unwrap_or_default(),
                late_fee: stored.late_fee,
            })
        } else {
            Source::Manual
        };
        Self {
            reference,
            source,
            amount: stored.amount,
            paid: stored.paid.unwrap_or(false),
        }
    }
}

pub enum BillListEvent {
    Changed,
}

pub enum PushOutcome {
    Added,
    Duplicate(usize),
}

pub struct BillList {
    entries: Vec<Entry>,
    cursor: usize,
}

impl EventEmitter<BillListEvent> for BillList {}

impl BillList {
    pub fn new(entries: Vec<Entry>, cursor: Option<usize>) -> Self {
        let cursor = cursor
            .unwrap_or_else(|| next_unpaid(&entries, 0))
            .min(entries.len());
        Self { entries, cursor }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn next(&self) -> Option<(usize, &Entry)> {
        self.entries
            .get(self.cursor)
            .filter(|entry| !entry.paid)
            .map(|entry| (self.cursor, entry))
    }

    pub fn paid_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.paid).count()
    }

    pub fn paid_amount(&self) -> u32 {
        self.entries
            .iter()
            .filter(|entry| entry.paid)
            .filter_map(|entry| entry.amount)
            .sum()
    }

    pub fn total_amount(&self) -> u32 {
        self.entries.iter().filter_map(|entry| entry.amount).sum()
    }

    pub fn can_step_back(&self) -> bool {
        self.entries[..self.cursor.min(self.entries.len())]
            .iter()
            .any(|entry| entry.paid)
    }

    pub fn to_stored(&self) -> Vec<StoredBill> {
        self.entries.iter().map(Entry::to_stored).collect()
    }

    pub fn push(&mut self, entry: Entry, cx_self: &mut Context<Self>) -> PushOutcome {
        if let Some(index) = self
            .entries
            .iter()
            .position(|item| item.reference == entry.reference)
        {
            return PushOutcome::Duplicate(index);
        }
        self.entries.push(entry);
        self.changed(cx_self);
        PushOutcome::Added
    }

    pub fn remove(&mut self, index: usize, cx_self: &mut Context<Self>) {
        if index >= self.entries.len() {
            return;
        }
        self.entries.remove(index);
        if index < self.cursor {
            self.cursor -= 1;
        }
        self.cursor = self.cursor.min(self.entries.len());
        self.changed(cx_self);
    }

    pub fn set_amount(&mut self, index: usize, amount: Option<u32>, cx_self: &mut Context<Self>) {
        let Some(entry) = self.entries.get_mut(index) else {
            return;
        };
        if entry.amount != amount {
            entry.amount = amount;
            self.changed(cx_self);
        }
    }

    pub fn set_paid(&mut self, index: usize, paid: bool, cx_self: &mut Context<Self>) {
        let Some(entry) = self.entries.get_mut(index) else {
            return;
        };
        if entry.paid == paid {
            return;
        }
        entry.paid = paid;
        if paid && self.cursor == index {
            self.cursor = next_unpaid(&self.entries, index + 1);
        }
        self.changed(cx_self);
    }

    pub fn set_cursor(&mut self, index: usize, cx_self: &mut Context<Self>) {
        let index = index.min(self.entries.len());
        if self.cursor != index {
            self.cursor = index;
            self.changed(cx_self);
        }
    }

    pub fn step_back(&mut self, cx_self: &mut Context<Self>) {
        let end = self.cursor.min(self.entries.len());
        let Some(index) = self.entries[..end].iter().rposition(|entry| entry.paid) else {
            return;
        };
        self.entries[index].paid = false;
        self.cursor = index;
        self.changed(cx_self);
    }

    fn changed(&mut self, cx_self: &mut Context<Self>) {
        cx_self.emit(BillListEvent::Changed);
        cx_self.notify();
    }
}

fn next_unpaid(entries: &[Entry], from: usize) -> usize {
    entries
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, entry)| !entry.paid)
        .map(|(index, _)| index)
        .unwrap_or(entries.len())
}
