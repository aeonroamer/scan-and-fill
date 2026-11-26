use super::*;

pub(super) fn local_time(millis: i64) -> String {
    Local
        .timestamp_millis_opt(millis)
        .single()
        .map(|time| time.format("%-d %b, %-I:%M %p").to_string())
        .unwrap_or_default()
}

pub(super) fn local_date(millis: i64) -> String {
    Local
        .timestamp_millis_opt(millis)
        .single()
        .map(|time| time.format("%-d %b").to_string())
        .unwrap_or_default()
}

#[cfg(debug_assertions)]
pub(super) fn sample_entries() -> Option<Vec<Entry>> {
    Some(
        (0..40u32)
            .map(|i| {
                if i % 7 == 6 {
                    Entry::manual(format!("09154379{:07}", 3_400 + i * 37).into())
                } else {
                    Entry::scanned(Bill {
                        format: "electricity".into(),
                        label: "Electricity".into(),
                        reference: format!("0412778{:07}", 1_229_000 + i * 113).into(),
                        company: Some("GEPCO".into()),
                        amount: 1_200 + (i * 937) % 18_000,
                        late_fee: i % 5 == 3,
                    })
                }
            })
            .collect(),
    )
}

#[cfg(not(debug_assertions))]
pub(super) fn sample_entries() -> Option<Vec<Entry>> {
    None
}

pub(super) fn shortcut(keys: &'static str, action: &'static str, theme: &Theme, mono: &SharedString) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(
            div()
                .px(px(6.))
                .py(px(1.))
                .rounded(px(4.))
                .border_1()
                .border_color(theme.border)
                .bg(theme.bg_panel)
                .font_family(mono.clone())
                .text_size(px(11.))
                .text_color(theme.text_muted)
                .child(keys),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.text_muted)
                .child(action),
        )
}

const REFERENCE_MAX: usize = 24;
const REFERENCE_LABEL: &str = "REF NO:";

pub(super) fn clean_reference(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if let Some(at) = raw.to_ascii_uppercase().find(REFERENCE_LABEL) {
        let digits: String = raw[at + REFERENCE_LABEL.len()..]
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        return (!digits.is_empty() && digits.len() <= REFERENCE_MAX).then_some(digits);
    }
    let valid = !raw.is_empty()
        && raw.len() <= REFERENCE_MAX
        && raw.chars().all(|c| c.is_ascii_alphanumeric());
    valid.then(|| raw.to_string())
}
