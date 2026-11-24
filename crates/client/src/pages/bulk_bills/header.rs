use gpui_kit::ClipboardItem;

use super::*;

impl BulkBillsPage {
    pub(super) fn session_header(&self, theme: &Theme, cx_self: &mut Context<Self>) -> Option<Div> {
        let session = self.current_session()?;

        let header_editor = self
            .renaming
            .as_ref()
            .filter(|_| !self.rename_in_sidebar);
        let title = match header_editor {
            Some(editor) => div()
                .w(px(320.))
                .child(Input::new(&editor.input).small())
                .into_any_element(),
            None => {
                let rename = cx_self
                    .listener(|this, _, window, cx_self| this.start_rename(false, window, cx_self));
                let hover_bg = theme.bg_hover;
                div()
                    .id("bulk-session-name")
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .text_size(px(18.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_main)
                    .child(session.name.clone())
                    .on_click(rename)
                    .into_any_element()
            }
        };

        let rename_button = Button::new("bulk-rename-session")
            .label("Rename")
            .ghost()
            .small()
            .on_click(
                cx_self
                    .listener(|this, _, window, cx_self| this.start_rename(false, window, cx_self)),
            );
        let delete_button = Button::new("bulk-delete-session")
            .label("Delete session")
            .ghost()
            .small()
            .on_click(
                cx_self.listener(|this, _, window, cx_self| this.start_delete(window, cx_self)),
            );

        let confirm_panel = self.deleting.as_ref().map(|confirm| {
            let count = session.bills.read(cx_self).entries().len();
            let ready = self.delete_typed_correctly(cx_self);
            let cancel = cx_self.listener(|this, _, _, cx_self| this.cancel_delete(cx_self));
            let delete = cx_self.listener(|this, _, _, cx_self| this.confirm_delete(cx_self));

            div()
                .p(px(16.))
                .rounded(px(8.))
                .border_1()
                .border_color(theme.error)
                .bg(theme.error_subtle)
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_main)
                        .child(format!("Delete \"{}\"?", session.name)),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child(format!(
                            "This removes the session and its {count} {}. It can't be undone.",
                            if count == 1 { "bill" } else { "bills" }
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(4.))
                        .text_size(px(13.))
                        .text_color(theme.text_main)
                        .child("Type")
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.error)
                                .child(session.name.clone()),
                        )
                        .child({
                            let name = session.name.to_string();
                            Button::new("bulk-delete-copy")
                                .icon(IconName::Copy)
                                .ghost()
                                .xsmall()
                                .tooltip("Copy name")
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()))
                                })
                        })
                        .child("to confirm."),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().flex_1().child(Input::new(&confirm.input)))
                        .child(
                            Button::new("bulk-delete-cancel")
                                .label("Cancel")
                                .ghost()
                                .on_click(cancel),
                        )
                        .child(
                            Button::new("bulk-delete-confirm")
                                .label("Delete session")
                                .danger()
                                .disabled(!ready)
                                .on_click(delete),
                        ),
                )
        });

        let title_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(title)
                    .child(
                        div()
                            .px(px(6.))
                            .text_size(px(12.))
                            .text_color(theme.text_muted)
                            .child(format!("Created {}", local_time(session.created_at))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(rename_button)
                    .when(self.deleting.is_none(), |el| el.child(delete_button)),
            );

        Some(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(title_row)
                .children(confirm_panel),
        )
    }
}
