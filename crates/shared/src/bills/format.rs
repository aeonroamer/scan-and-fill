use std::ops::Range;

use chrono::NaiveDate;
use gpui_shared_string::SharedString;

use super::Bill;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LateRule {
    pub after: Range<usize>,
    pub amount: Option<Range<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompanyRule {
    Fixed(SharedString),
    ByCode {
        at: Range<usize>,
        codes: Vec<(SharedString, SharedString)>,
    },
}

impl CompanyRule {
    pub fn find(&self, reference: &str) -> Option<SharedString> {
        match self {
            Self::Fixed(name) => Some(name.clone()),
            Self::ByCode { at, codes } => {
                let code = reference.get(at.clone())?;
                codes
                    .iter()
                    .find(|(candidate, _)| &**candidate == code)
                    .map(|(_, name)| name.clone())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillFormat {
    pub id: SharedString,
    pub label: SharedString,
    pub length: usize,
    pub reference: Range<usize>,
    pub company: CompanyRule,
    pub amount: Range<usize>,
    pub date_format: SharedString,
    pub late: Vec<LateRule>,
}

impl BillFormat {
    pub fn all() -> Vec<Self> {
        vec![Self::electricity(), Self::gas()]
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
            company: CompanyRule::ByCode {
                at: 2..4,
                codes: [
                    ("11", "LESCO"),
                    ("12", "GEPCO"),
                    ("13", "FESCO"),
                    ("14", "IESCO"),
                    ("15", "MEPCO"),
                    ("26", "PESCO"),
                    ("37", "HESCO"),
                    ("38", "SEPCO"),
                    ("48", "QESCO"),
                    ("59", "TESCO"),
                ]
                .into_iter()
                .map(|(code, name)| (code.into(), name.into()))
                .collect(),
            },
            amount: 26..34,
            date_format: "%d%m%y".into(),
            late: vec![
                LateRule {
                    after: 43..49,
                    amount: Some(35..43),
                },
                LateRule {
                    after: 19..25,
                    amount: Some(50..58),
                },
            ],
        }
    }

    pub fn gas() -> Self {
        Self {
            id: "gas".into(),
            label: "Gas".into(),
            length: 50,
            reference: 5..16,
            company: CompanyRule::Fixed("SNGPL".into()),
            amount: 32..42,
            date_format: "%d%m%y".into(),
            late: vec![LateRule {
                after: 26..32,
                amount: None,
            }],
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

        let amount_range = late
            .and_then(|rule| rule.amount.as_ref())
            .unwrap_or(&self.amount);
        let amount = raw.get(amount_range.clone())?.parse().ok()?;

        Some(Bill {
            format: self.id.clone(),
            label: self.label.clone(),
            reference: reference.to_string().into(),
            company: self.company.find(reference),
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
        assert_eq!(on_time.company.as_deref(), Some("MEPCO"));
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
    fn real_gas_barcode() {
        let code = "03000523323561130102600000021226000000134000013003";
        let format = BillFormat::gas();

        let on_time = format.parse(code, date(2026, 12, 2)).unwrap();
        assert_eq!(on_time.reference, "52332356113");
        assert_eq!(on_time.company.as_deref(), Some("SNGPL"));
        assert_eq!(on_time.amount, 1340);
        assert!(!on_time.late_fee);

        let late = format.parse(code, date(2026, 12, 3)).unwrap();
        assert_eq!(late.amount, 1340);
        assert!(late.late_fee);

        assert_eq!(Bill::parse(code, date(2026, 12, 1)).unwrap().format, "gas");
    }

    #[test]
    fn bill_parse_finds_the_right_format() {
        let bill = Bill::parse(&sample(), date(2026, 12, 1)).unwrap();
        assert_eq!(bill.format, "electricity");
        assert_eq!(bill.label, "Electricity");
    }
}
