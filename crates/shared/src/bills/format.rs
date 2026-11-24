use std::ops::Range;

use chrono::NaiveDate;
use gpui_shared_string::SharedString;

use super::Bill;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LateRule {
    pub after: Range<usize>,
    pub amount: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillFormat {
    pub id: SharedString,
    pub label: SharedString,
    pub length: usize,
    pub reference: Range<usize>,
    pub amount: Range<usize>,
    pub date_format: SharedString,
    pub late: Vec<LateRule>,
}

impl BillFormat {
    pub fn all() -> Vec<Self> {
        vec![Self::electricity()]
    }

    pub fn find(id: &str) -> Option<Self> {
        Self::all().into_iter().find(|format| format.id == id)
    }

    pub fn electricity() -> Self {
        Self {
            id: "electricity".into(),
            label: "Electricity".into(),
            length: 60,
            reference: 1..15,
            amount: 26..34,
            date_format: "%d%m%y".into(),
            late: vec![
                LateRule {
                    after: 43..49,
                    amount: 35..43,
                },
                LateRule {
                    after: 19..25,
                    amount: 50..58,
                },
            ],
        }
    }

    pub fn parse(&self, raw: &str, today: NaiveDate) -> Option<Bill> {
        if raw.len() != self.length {
            return None;
        }

        let reference = raw.get(self.reference.clone())?;

        let dates = self
            .late
            .iter()
            .map(|rule| NaiveDate::parse_from_str(raw.get(rule.after.clone())?, &self.date_format).ok())
            .collect::<Option<Vec<_>>>()?;

        let late = self
            .late
            .iter()
            .zip(dates)
            .find(|(_, date)| *date < today)
            .map(|(rule, _)| rule);

        let amount_range = late.map_or(&self.amount, |rule| &rule.amount);
        let amount = raw.get(amount_range.clone())?.parse().ok()?;

        Some(Bill {
            format: self.id.clone(),
            label: self.label.clone(),
            reference: reference.to_string().into(),
            amount,
            late_fee: late.is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn barcode(due: &str, last: &str, on_time: &str, after_due: &str, after_last: &str) -> String {
        let code = format!(
            "X{reference}0000{due}0{on_time}0{after_last}{last}0{after_due}00",
            reference = "04127781229011",
        );
        assert_eq!(code.len(), 60);
        code
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn sample() -> String {
        barcode("101226", "201226", "00003920", "00004120", "00004320")
    }

    #[test]
    fn on_time_uses_the_normal_amount() {
        let bill = BillFormat::electricity().parse(&sample(), date(2026, 12, 1)).unwrap();
        assert_eq!(bill.reference, "04127781229011");
        assert_eq!(bill.amount, 3920);
        assert!(!bill.late_fee);
    }

    #[test]
    fn on_the_due_date_is_still_on_time() {
        let bill = BillFormat::electricity().parse(&sample(), date(2026, 12, 10)).unwrap();
        assert_eq!(bill.amount, 3920);
        assert!(!bill.late_fee);
    }

    #[test]
    fn after_the_due_date_uses_the_first_late_amount() {
        let bill = BillFormat::electricity().parse(&sample(), date(2026, 12, 15)).unwrap();
        assert_eq!(bill.amount, 4120);
        assert!(bill.late_fee);
    }

    #[test]
    fn after_the_last_date_uses_the_final_amount() {
        let bill = BillFormat::electricity().parse(&sample(), date(2026, 12, 25)).unwrap();
        assert_eq!(bill.amount, 4320);
        assert!(bill.late_fee);
    }

    #[test]
    fn wrong_length_is_rejected() {
        assert!(BillFormat::electricity().parse("12345", date(2026, 12, 1)).is_none());
    }

    #[test]
    fn broken_date_is_rejected() {
        let code = barcode("991399", "201226", "00003920", "00004120", "00004320");
        assert!(BillFormat::electricity().parse(&code, date(2026, 12, 1)).is_none());
    }

    #[test]
    fn real_electricity_barcode() {
        let code = "E0415437415317811262611260000021300000022833011260000022076E";
        let format = BillFormat::electricity();

        let on_time = format.parse(code, date(2026, 11, 24)).unwrap();
        assert_eq!(on_time.reference, "04154374153178");
        assert_eq!(on_time.amount, 2130);
        assert!(!on_time.late_fee);

        let after_due = format.parse(code, date(2026, 11, 28)).unwrap();
        assert_eq!(after_due.amount, 2207);
        assert!(after_due.late_fee);

        let after_last = format.parse(code, date(2026, 12, 1)).unwrap();
        assert_eq!(after_last.amount, 2283);
        assert!(after_last.late_fee);
    }

    #[test]
    fn bill_parse_finds_the_right_format() {
        let bill = Bill::parse(&sample(), date(2026, 12, 1)).unwrap();
        assert_eq!(bill.format, "electricity");
        assert_eq!(bill.label, "Electricity");
    }
}
