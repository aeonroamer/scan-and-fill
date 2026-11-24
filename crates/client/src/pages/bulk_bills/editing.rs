use super::*;

impl BulkBillsPage {
    pub(super) fn start_delete(&mut self, window: &mut Window, cx_self: &mut Context<Self>) {
        if self.is_running(cx_self) || self.deleting.is_some() {
            return;
        }
        let Some(name) = self.current_name() else {
            return;
        };
        self.commit_rename(cx_self);

        let input = cx_self.new(|cx_self| InputState::new(window, cx_self).placeholder(name));
        input.update(cx_self, |state, cx_self| state.focus(window, cx_self));

        let subscription = cx_self.subscribe_in(
            &input,
            window,
            |this: &mut Self, _, event: &InputEvent, _, cx_self| match event {
                InputEvent::PressEnter { .. } => this.confirm_delete(cx_self),
                InputEvent::Change => cx_self.notify(),
                _ => {}
            },
        );

        self.deleting = Some(DeleteConfirm {
            input,
            _subscription: subscription,
        });
        cx_self.notify();
    }

    pub(super) fn cancel_delete(&mut self, cx_self: &mut Context<Self>) {
        self.deleting = None;
        cx_self.notify();
    }

    pub(super) fn delete_typed_correctly(&self, cx_self: &Context<Self>) -> bool {
        let (Some(confirm), Some(name)) = (self.deleting.as_ref(), self.current_name()) else {
            return false;
        };
        confirm.input.read(cx_self).value().trim() == name.trim()
    }

    pub(super) fn confirm_delete(&mut self, cx_self: &mut Context<Self>) {
        if !self.delete_typed_correctly(cx_self) {
            return;
        }
        let Some(index) = self.current else {
            return;
        };
        let session = self.sessions.remove(index);
        let db = Database::get(cx_self);
        Io::spawn(cx_self, async move {
            if let Err(e) = database::delete_session(&db, session.key).await {
                tracing::error!("deleting bulk session failed: {e}");
            }
        });

        self.reset_row_state();
        self.current = if self.sessions.is_empty() {
            None
        } else {
            Some(index.min(self.sessions.len() - 1))
        };
        cx_self.notify();
    }

    pub(super) fn start_edit(&mut self, index: usize, window: &mut Window, cx_self: &mut Context<Self>) {
        if self.editing.as_ref().is_some_and(|editor| editor.index == index) {
            return;
        }
        self.commit_edit(cx_self);

        let Some(bills) = self.current_bills() else {
            return;
        };
        let Some(current) = bills.read(cx_self).entries().get(index).map(|entry| entry.amount)
        else {
            return;
        };

        let input = cx_self.new(|cx_self| {
            let state = InputState::new(window, cx_self)
                .placeholder("0")
                .validate(|text, _| {
                    text.len() <= AMOUNT_DIGITS && text.chars().all(|c| c.is_ascii_digit())
                });
            match current {
                Some(amount) => state.default_value(amount.to_string()),
                None => state,
            }
        });
        input.update(cx_self, |state, cx_self| state.focus(window, cx_self));

        let subscription = cx_self.subscribe_in(
            &input,
            window,
            |this: &mut Self, _, event: &InputEvent, _, cx_self| match event {
                InputEvent::PressEnter { .. } | InputEvent::Blur => this.commit_edit(cx_self),
                _ => {}
            },
        );

        self.editing = Some(InlineEditor {
            index,
            input,
            _subscription: subscription,
        });
        cx_self.notify();
    }

    pub(super) fn commit_edit(&mut self, cx_self: &mut Context<Self>) {
        let Some(editor) = self.editing.take() else {
            return;
        };
        let text = editor.input.read(cx_self).value().trim().to_string();

        if let Some(bills) = self.current_bills() {
            bills.update(cx_self, |bills, cx_self| {
                let Some(entry) = bills.entries().get(editor.index) else {
                    return;
                };
                let amount = if text.is_empty() {
                    None
                } else {
                    text.parse().ok().or(entry.amount)
                };
                bills.set_amount(editor.index, amount, cx_self);
            });
        }
        cx_self.notify();
    }

    pub(super) fn start_rename(&mut self, in_sidebar: bool, window: &mut Window, cx_self: &mut Context<Self>) {
        let Some(index) = self.current else {
            return;
        };
        if self.renaming.is_some() {
            return;
        }
        self.deleting = None;
        self.rename_in_sidebar = in_sidebar;
        let name = self.sessions[index].name.clone();

        let input = cx_self.new(|cx_self| {
            InputState::new(window, cx_self)
                .placeholder("Session name")
                .default_value(name)
        });
        input.update(cx_self, |state, cx_self| state.focus(window, cx_self));

        let subscription = cx_self.subscribe_in(
            &input,
            window,
            |this: &mut Self, _, event: &InputEvent, _, cx_self| match event {
                InputEvent::PressEnter { .. } | InputEvent::Blur => this.commit_rename(cx_self),
                _ => {}
            },
        );

        self.renaming = Some(InlineEditor {
            index,
            input,
            _subscription: subscription,
        });
        cx_self.notify();
    }

    pub(super) fn commit_rename(&mut self, cx_self: &mut Context<Self>) {
        let Some(editor) = self.renaming.take() else {
            return;
        };
        let text = editor.input.read(cx_self).value().trim().to_string();

        let renamed = match self.sessions.get_mut(editor.index) {
            Some(session) if !text.is_empty() && session.name != text.as_str() => {
                session.name = text.into();
                Some(session.key.clone())
            }
            _ => None,
        };
        if let Some(key) = renamed {
            self.save_session(&key, cx_self);
        }
        cx_self.notify();
    }
}
