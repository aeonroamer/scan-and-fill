pub struct Utility {
    pub id: &'static str,
    pub label: &'static str,
    pub companies: &'static [&'static str],
}

pub const UTILITIES: &[Utility] = &[
    Utility {
        id: "electricity",
        label: "Electricity",
        companies: &[
            "FESCO", "GEPCO", "HESCO", "IESCO", "LESCO", "MEPCO", "PESCO", "QESCO", "HAZECO",
            "SEPCO", "TESCO",
        ],
    },
    Utility {
        id: "gas",
        label: "Gas",
        companies: &["SNGPL", "SSGC"],
    },
];

pub fn utility_label(id: &str) -> Option<&'static str> {
    UTILITIES
        .iter()
        .find(|utility| utility.id == id)
        .map(|utility| utility.label)
}
