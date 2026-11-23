#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillKind {
    Electricity,
}

impl BillKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Electricity => "Electricity",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bill {
    pub kind: BillKind,
    pub reference: String,
    pub amount: u32,
    pub late_fee: bool,
}
