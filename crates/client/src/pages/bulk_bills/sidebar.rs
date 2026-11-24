use super::*;

impl BulkBillsPage {
    pub(super) fn session_row(
        &self,
        index: usize,
        session: &Session,
        theme: &Theme,
        cx_self: &mut Context<Self>,
    ) -> Stateful<Div> {
        let selected = self.current == Some(index);
        let hover_bg = theme.bg_hover;
        let (count, paid) = {
            let bills = session.bills.read(cx_self);
            (bills.entries().len(), bills.paid_count())
        };
        let open = cx_self.listener(move |this, event: &ClickEvent, window, cx_self| {
            this.open_session(index, cx_self);
            if event.click_count() >= 2 {
                this.start_rename(true, window, cx_self);
            }
        });

        let editor = self
            .renaming
            .as_ref()
            .filter(|editor| self.rename_in_sidebar && editor.index == index);
        let name = match editor {
            Some(editor) => div()
                .child(Input::new(&editor.input).small())
                .into_any_element(),
            None => div()
                .text_size(px(14.))
                .text_color(theme.text_main)
                .when(selected, |el| el.font_weight(FontWeight::MEDIUM))
                .child(session.name.clone())
                .into_any_element(),
        };

        div()
            .id(SharedString::from(format!("session-{}", session.key)))
            .px(px(10.))
            .py(px(8.))
            .rounded(px(6.))
            .cursor_pointer()
            .flex()
            .flex_col()
            .gap(px(2.))
            .when(selected, |el| el.bg(theme.bg_selected))
            .when(!selected, |el| el.hover(move |s| s.bg(hover_bg)))
            .child(name)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child(format!(
                        "{} · {paid}/{count} paid",
                        local_date(session.updated_at)
                    )),
            )
            .on_click(open)
    }

    pub(super) fn sessions_panel(&self, theme: &Theme, cx_self: &mut Context<Self>) -> Div {
        let new = cx_self.listener(|this, _, _, cx_self| this.new_session(cx_self));

        let rows: Vec<_> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(index, session)| self.session_row(index, session, theme, cx_self))
            .collect();

        let list = if rows.is_empty() && self.loaded {
            div()
                .px(px(10.))
                .py(px(12.))
                .text_size(px(13.))
                .text_color(theme.text_muted)
                .child("No sessions yet.")
        } else {
            div().flex().flex_col().gap(px(2.)).children(rows)
        };

        card(theme)
            .w_full()
            .flex_1()
            .min_h_0()
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .px(px(6.))
                    .pt(px(4.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(theme.text_muted)
                            .child("Sessions"),
                    )
                    .child(
                        Button::new("bulk-new-session")
                            .label("New")
                            .icon(IconName::Plus)
                            .outline()
                            .xsmall()
                            .on_click(new),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(list)
                    .overflow_y_scrollbar(),
            )
    }

    pub(super) fn run_panel(
        &self,
        compact: bool,
        theme: &Theme,
        mono: &SharedString,
        cx_self: &mut Context<Self>,
    ) -> Div {
        let target = self.software_label();
        let (count, paid, paid_amount, total_amount, next, can_step_back) =
            match self.current_bills() {
                Some(bills) => {
                    let bills = bills.read(cx_self);
                    (
                        bills.entries().len(),
                        bills.paid_count(),
                        bills.paid_amount(),
                        bills.total_amount(),
                        bills
                            .next()
                            .map(|(index, entry)| (index, entry.reference.clone())),
                        bills.can_step_back(),
                    )
                }
                None => (0, 0, 0, 0, None, false),
            };
        let ready = next.is_some();

        let running = self.is_running(cx_self);
        let (title, helper): (SharedString, &str) = if running {
            (
                format!("Typing into {target}").into(),
                "Bills are typed one by one, starting at the cursor.",
            )
        } else if ready {
            ("Ready to type".into(), "Start types the bill at the cursor, then moves on.")
        } else if count == 0 {
            ("No bills yet".into(), "Scan or add bills to this session to start typing.")
        } else if paid < count {
            ("Cursor is on a paid bill".into(), "Move the cursor to an unpaid bill to start typing.")
        } else {
            ("Nothing left to type".into(), "Every bill in this session is paid.")
        };

        let softwares: Vec<_> = self
            .softwares
            .iter()
            .map(|software| {
                let id = software.id.clone();
                let subtitle: SharedString = match software.kind {
                    SoftwareKind::Desktop => match software.window.process.first() {
                        Some(exe) => format!("Desktop app · {exe}").into(),
                        None => "Desktop app".into(),
                    },
                    SoftwareKind::Web => "Web portal".into(),
                };
                pick_row(
                    format!("bulk-software-{}", software.id).into(),
                    software.label.clone(),
                    subtitle,
                    self.software.as_ref() == Some(&software.id),
                    theme,
                    cx_self.listener(move |this, _, _, cx_self| {
                        this.select_software(id.clone(), cx_self)
                    }),
                )
            })
            .collect();

        let toggle = cx_self.listener(|this, _, _, cx_self| this.toggle_running(cx_self));
        let start = if running {
            Button::new("bulk-run")
                .label("Stop")
                .icon(IconName::Square)
                .outline()
                .large()
                .w_full()
                .on_click(toggle)
        } else {
            Button::new("bulk-run")
                .label("Start typing")
                .icon(IconName::Play)
                .primary()
                .large()
                .w_full()
                .disabled(!ready)
                .on_click(toggle)
        };

        let back = cx_self.listener(|this, _, _, cx_self| this.step_back(cx_self));
        let progress = if count == 0 {
            0.
        } else {
            paid as f32 / count as f32
        };
        let dot_color = if running {
            theme.accent
        } else if ready {
            theme.text_muted
        } else {
            theme.success
        };

        let stat = |label: &'static str, value: Div| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child(label),
                )
                .child(value)
        };

        let next_value = match next {
            Some((index, reference)) => div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_size(px(13.))
                .text_color(theme.text_main)
                .child(format!("#{:02}", index + 1))
                .child(
                    div()
                        .font_family(mono.clone())
                        .text_color(theme.text_muted)
                        .child(reference),
                ),
            None => div()
                .text_size(px(13.))
                .text_color(theme.text_disabled)
                .child("—"),
        };

        card(theme)
            .w_full()
            .flex_none()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(div().size(px(8.)).rounded_full().bg(dot_color))
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text_main)
                                    .child(title),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.text_muted)
                            .child(helper),
                    ),
            )
            .when(!compact, |el| el.child(picker_card(theme).children(softwares)))
            .child(start)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(13.))
                            .child(
                                div()
                                    .text_color(theme.text_main)
                                    .child(format!("{paid} of {count} paid")),
                            )
                            .child(
                                div()
                                    .text_color(theme.text_muted)
                                    .child(format!("{:.0}%", progress * 100.)),
                            ),
                    )
                    .child(
                        div()
                            .h(px(6.))
                            .w_full()
                            .rounded_full()
                            .bg(theme.bg_hover)
                            .overflow_hidden()
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(progress))
                                    .rounded_full()
                                    .bg(theme.success),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(stat("Next", next_value))
                    .when(!compact, |el| {
                        el.child(stat(
                            "Paid",
                            div()
                                .text_size(px(13.))
                                .text_color(theme.text_main)
                                .child(rupees(paid_amount)),
                        ))
                        .child(stat(
                            "Left",
                            div()
                                .text_size(px(13.))
                                .text_color(theme.text_main)
                                .child(rupees(total_amount.saturating_sub(paid_amount))),
                        ))
                    }),
            )
            .child(
                Button::new("bulk-step-back")
                    .label("Undo last paid")
                    .ghost()
                    .small()
                    .disabled(!can_step_back)
                    .on_click(back),
            )
    }
}
