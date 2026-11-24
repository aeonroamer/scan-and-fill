use super::*;

impl BulkBillsPage {
    pub(super) fn amount_cell(
        &self,
        index: usize,
        entry: &Entry,
        theme: &Theme,
        cx_self: &mut Context<Self>,
    ) -> Div {
        if let Some(editor) = self.editing.as_ref().filter(|editor| editor.index == index) {
            return div()
                .w(px(COL_AMOUNT))
                .child(Input::new(&editor.input).small());
        }

        let value = match entry.amount {
            Some(amount) => div()
                .text_size(px(15.))
                .text_color(theme.text_main)
                .child(rupees(amount)),
            None => div()
                .text_size(px(13.))
                .text_color(theme.accent)
                .child("Set amount"),
        };

        let edit = cx_self.listener(move |this, _, window, cx_self| {
            this.start_edit(index, window, cx_self)
        });
        let hover_bg = theme.bg_active;

        div().w(px(COL_AMOUNT)).flex().justify_end().child(
            div()
                .id(SharedString::from(format!("amount-{index}")))
                .px(px(8.))
                .py(px(4.))
                .rounded(px(6.))
                .cursor_pointer()
                .flex()
                .flex_col()
                .items_end()
                .hover(move |s| s.bg(hover_bg))
                .child(value)
                .when(entry.edited(), |el| {
                    el.child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.accent)
                            .child("Edited"),
                    )
                })
                .on_click(edit),
        )
    }

    pub(super) fn entry_row(
        &self,
        index: usize,
        entry: &Entry,
        cursor: usize,
        theme: &Theme,
        mono: &SharedString,
        cx_self: &mut Context<Self>,
    ) -> ContextMenu<Stateful<Div>> {
        let is_cursor = index == cursor && !entry.paid;
        let is_duplicate = self.duplicate == Some(index);

        let (title, subtitle): (SharedString, &str) = match &entry.source {
            Source::Scanned(bill) => (bill.label.clone(), "Scanned barcode"),
            Source::Manual => ("Reference".into(), "Added by hand"),
        };
        let (badge, badge_fg, badge_bg) = match &entry.source {
            _ if entry.paid => ("Paid", theme.success, theme.success_subtle),
            Source::Scanned(bill) if bill.late_fee => {
                ("Late fee", theme.warning, theme.warning_subtle)
            }
            Source::Scanned(_) => ("On time", theme.text_main, theme.bg_hover),
            Source::Manual => ("Typed", theme.text_muted, theme.bg_hover),
        };

        let remove = cx_self.listener(move |this, _, _, cx_self| this.remove(index, cx_self));
        let hover_bg = theme.bg_hover;
        let row_hover_icon = theme.text_muted;
        let danger = theme.error;
        let danger_bg = theme.error_subtle;

        let row_bg = if is_duplicate {
            Some(theme.warning_subtle)
        } else if is_cursor {
            Some(theme.accent_subtle)
        } else if entry.paid {
            Some(theme.success_subtle)
        } else {
            None
        };

        let (chip_bg, chip_fg) = if is_cursor {
            (theme.accent, theme.text_on_accent)
        } else if entry.paid {
            (theme.success, theme.text_on_accent)
        } else {
            (theme.bg_hover, theme.text_muted)
        };

        let row = div()
            .id(SharedString::from(format!("bill-{}", entry.reference)))
            .px(px(16.))
            .py(px(12.))
            .flex()
            .items_center()
            .gap(px(16.))
            .border_t_1()
            .border_color(theme.border_subtle)
            .group(ROW_GROUP)
            .when_some(row_bg, |el, bg| el.bg(bg))
            .when(row_bg.is_none(), |el| el.hover(move |s| s.bg(hover_bg)))
            .child(
                div().w(px(COL_INDEX)).child(
                    div()
                        .size(px(26.))
                        .rounded(px(6.))
                        .bg(chip_bg)
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family(mono.clone())
                        .text_size(px(11.))
                        .text_color(chip_fg)
                        .child(format!("{:02}", index + 1)),
                ),
            )
            .child(
                div()
                    .w(px(COL_BILL))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text_main)
                                    .child(title),
                            )
                            .when(is_cursor, |el| {
                                el.child(
                                    div()
                                        .px(px(6.))
                                        .rounded(px(4.))
                                        .bg(theme.accent)
                                        .text_size(px(10.))
                                        .text_color(theme.text_on_accent)
                                        .child("Next"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.text_muted)
                            .child(subtitle),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_family(mono.clone())
                    .text_size(px(14.))
                    .text_color(theme.text_main)
                    .child(entry.reference.clone()),
            )
            .child(div().w(px(COL_STATUS)).child(badge_pill(badge, badge_fg, badge_bg)))
            .child(self.amount_cell(index, entry, theme, cx_self))
            .child(
                div().w(px(COL_ACTION)).flex().justify_end().child(
                    div()
                        .id(SharedString::from(format!("remove-{index}")))
                        .size(px(28.))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_color(theme.text_disabled)
                        .group_hover(ROW_GROUP, move |s| s.text_color(row_hover_icon))
                        .hover(move |s| s.bg(danger_bg).text_color(danger))
                        .child(Icon::new(IconName::Close).small())
                        .on_click(remove),
                ),
            );

        let bills = self.current_bills().map(|bills| bills.downgrade());
        let paid = entry.paid;
        row.context_menu(move |menu, _, _| {
            let Some(bills) = bills.clone() else {
                return menu;
            };
            let pay = bills.clone();
            let cursor_to = bills.clone();
            let remover = bills;
            menu.item(
                PopupMenuItem::new(if paid { "Mark as unpaid" } else { "Mark as paid" }).on_click(
                    move |_, _, cx| {
                        let _ = pay.update(cx, |bills, cx_self| {
                            bills.set_paid(index, !paid, cx_self)
                        });
                    },
                ),
            )
            .item(
                PopupMenuItem::new("Start from here")
                    .disabled(index == cursor)
                    .on_click(move |_, _, cx| {
                        let _ = cursor_to.update(cx, |bills, cx_self| {
                            bills.set_cursor(index, cx_self)
                        });
                    }),
            )
            .separator()
            .item(PopupMenuItem::new("Remove").on_click(move |_, _, cx| {
                let _ = remover.update(cx, |bills, cx_self| bills.remove(index, cx_self));
            }))
        })
    }
}

pub(super) fn badge_pill(text: &str, fg: Rgba, bg: Rgba) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(2.))
        .rounded_full()
        .bg(bg)
        .text_size(px(12.))
        .text_color(fg)
        .child(div().size(px(6.)).rounded_full().bg(fg))
        .child(text.to_string())
}

pub(super) fn column(label: &'static str, theme: &Theme) -> Div {
    div()
        .text_size(px(12.))
        .text_color(theme.text_muted)
        .child(label)
}
