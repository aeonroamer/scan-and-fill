mod editing;
mod header;
mod helpers;
mod session;
mod sidebar;
mod table;

use helpers::*;
use session::*;
use table::*;

use chrono::{Local, TimeZone, Utc};
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    menu::{ContextMenu, ContextMenuExt, PopupMenuItem},
    scroll::ScrollableElement,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AppContext, ClickEvent, Context, Div, Entity, Focusable, FontWeight, InteractiveElement,
    IntoElement, ParentElement, Render, Rgba, SharedString, Stateful, StatefulInteractiveElement,
    Styled, Subscription, Window, div, px, relative,
};
use shared::{Bill, Software, SoftwareKind};

use super::ui::{card, pick_row, picker_card, rupees};
use crate::database::{self, BulkSession};
use crate::state::{
    BillList, BillListEvent, Database, Entry, Io, Mode, PushOutcome, Runner, Source, Theme,
};

const NOT_RECOGNIZED: &str =
    "That barcode wasn't recognized. Press Add or Ctrl+Enter to keep it as a reference number.";
const ALREADY_SCANNED: &str = "This bill reference number has already been scanned. If this is a detection bill, you can edit the amount directly.";
const AMOUNT_DIGITS: usize = 9;
const NO_SOFTWARE: &str = "your portal";

const COMPOSER_H: f32 = 56.0;
const SIDE_W: f32 = 300.0;
const COL_INDEX: f32 = 44.0;
const COL_BILL: f32 = 180.0;
const COL_STATUS: f32 = 116.0;
const COL_AMOUNT: f32 = 130.0;
const COL_ACTION: f32 = 32.0;
const ROW_GROUP: &str = "bill-row";

pub struct BulkBillsPage {
    input: Entity<InputState>,
    sessions: Vec<Session>,
    current: Option<usize>,
    loaded: bool,
    error: Option<SharedString>,
    duplicate: Option<usize>,
    editing: Option<InlineEditor>,
    renaming: Option<InlineEditor>,
    rename_in_sidebar: bool,
    deleting: Option<DeleteConfirm>,
    runner: Entity<Runner>,
    softwares: Vec<Software>,
    software: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl BulkBillsPage {
    pub fn build_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let input = cx_self.new(|cx_self| {
                InputState::new(window, cx_self).placeholder("Scan a bill or type a reference number")
            });

            let subscription = cx_self.subscribe_in(
                &input,
                window,
                |this: &mut Self, _, event: &InputEvent, window, cx_self| match event {
                    InputEvent::PressEnter {
                        secondary: true, ..
                    } => this.add_manual(window, cx_self),
                    InputEvent::PressEnter { .. } => this.scan(window, cx_self),
                    InputEvent::Change
                        if (this.error.is_some() || this.duplicate.is_some())
                            && !this.text(cx_self).is_empty() =>
                    {
                        this.error = None;
                        this.duplicate = None;
                        cx_self.notify();
                    }
                    _ => {}
                },
            );

            let softwares = Software::all();
            let runner = Runner::get(cx_self);
            let page = Self {
                input,
                sessions: Vec::new(),
                current: None,
                loaded: false,
                error: None,
                duplicate: None,
                editing: None,
                renaming: None,
                rename_in_sidebar: false,
                deleting: None,
                runner: runner.clone(),
                software: softwares.first().map(|software| software.id.clone()),
                softwares,
                _subscriptions: vec![
                    subscription,
                    cx_self.observe(&runner, |_: &mut Self, _, cx_self| cx_self.notify()),
                ],
            };
            page.load(cx_self);
            page
        })
    }

    fn make_session(
        &self,
        key: String,
        name: SharedString,
        created_at: i64,
        updated_at: i64,
        entries: Vec<Entry>,
        cursor: Option<usize>,
        cx_self: &mut Context<Self>,
    ) -> Session {
        let bills = cx_self.new(|_| BillList::new(entries, cursor));
        let saved_key = key.clone();
        let subscription = cx_self.subscribe(
            &bills,
            move |this: &mut Self, _, _: &BillListEvent, cx_self| {
                this.save_session(&saved_key, cx_self);
                cx_self.notify();
            },
        );
        Session {
            key,
            name,
            created_at,
            updated_at,
            bills,
            _subscription: subscription,
        }
    }

    fn fresh_session(
        &self,
        name: Option<SharedString>,
        entries: Vec<Entry>,
        cx_self: &mut Context<Self>,
    ) -> Session {
        let now = Utc::now().timestamp_millis();
        let name = name.unwrap_or_else(|| local_time(now).into());
        self.make_session(now.to_string(), name, now, now, entries, None, cx_self)
    }

    fn load(&self, cx_self: &mut Context<Self>) {
        let db = Database::get(cx_self);
        let job = Io::spawn(cx_self, async move { database::list_sessions(&db).await });

        cx_self
            .spawn(async move |this, cx| {
                let records = match job.await {
                    Ok(Ok(records)) => records,
                    Ok(Err(e)) => {
                        tracing::error!("loading bulk sessions failed: {e}");
                        Vec::new()
                    }
                    Err(e) => {
                        tracing::error!("bulk session task failed: {e}");
                        Vec::new()
                    }
                };

                let _ = this.update(cx, |this, cx_self| {
                    for record in records {
                        let entries = record.bills.into_iter().map(Entry::from_stored).collect();
                        let session = this.make_session(
                            record.key,
                            record.name.into(),
                            record.created_at,
                            record.updated_at,
                            entries,
                            record.cursor.map(|cursor| cursor as usize),
                            cx_self,
                        );
                        this.sessions.push(session);
                    }

                    if this.sessions.is_empty() {
                        if let Some(entries) = sample_entries() {
                            let session =
                                this.fresh_session(Some("Sample session".into()), entries, cx_self);
                            let key = session.key.clone();
                            this.sessions.push(session);
                            this.save_session(&key, cx_self);
                        }
                    }
                    if !this.sessions.is_empty() {
                        this.current = Some(0);
                    }
                    this.loaded = true;
                    cx_self.notify();
                });
            })
            .detach();
    }

    fn save_session(&mut self, key: &str, cx_self: &mut Context<Self>) {
        let Some(session) = self.sessions.iter_mut().find(|session| session.key == key) else {
            return;
        };
        session.updated_at = Utc::now().timestamp_millis();
        let record = session.to_record(cx_self);
        let db = Database::get(cx_self);

        Io::spawn(cx_self, async move {
            if let Err(e) = database::save_session(&db, record).await {
                tracing::error!("saving bulk session failed: {e}");
            }
        });
    }

    fn current_session(&self) -> Option<&Session> {
        self.current.and_then(|index| self.sessions.get(index))
    }

    fn current_bills(&self) -> Option<Entity<BillList>> {
        self.current_session().map(|session| session.bills.clone())
    }

    fn current_name(&self) -> Option<SharedString> {
        self.current_session().map(|session| session.name.clone())
    }

    fn ensure_session(&mut self, cx_self: &mut Context<Self>) -> Entity<BillList> {
        if let Some(bills) = self.current_bills() {
            return bills;
        }
        let session = self.fresh_session(None, Vec::new(), cx_self);
        let bills = session.bills.clone();
        self.sessions.insert(0, session);
        self.current = Some(0);
        bills
    }

    fn reset_row_state(&mut self) {
        self.editing = None;
        self.renaming = None;
        self.duplicate = None;
        self.deleting = None;
        self.error = None;
    }

    fn new_session(&mut self, cx_self: &mut Context<Self>) {
        if self.is_running(cx_self) {
            return;
        }
        self.commit_edit(cx_self);
        self.commit_rename(cx_self);
        self.reset_row_state();
        let session = self.fresh_session(None, Vec::new(), cx_self);
        let key = session.key.clone();
        self.sessions.insert(0, session);
        self.current = Some(0);
        self.save_session(&key, cx_self);
        cx_self.notify();
    }

    fn open_session(&mut self, index: usize, cx_self: &mut Context<Self>) {
        if self.is_running(cx_self) || self.current == Some(index) {
            return;
        }
        self.commit_edit(cx_self);
        self.commit_rename(cx_self);
        self.reset_row_state();
        self.current = Some(index);
        cx_self.notify();
    }

    fn text(&self, cx_self: &Context<Self>) -> String {
        self.input.read(cx_self).value().trim().to_string()
    }

    fn clear_input(&self, window: &mut Window, cx_self: &mut Context<Self>) {
        self.input
            .update(cx_self, |state, cx_self| state.clean(window, cx_self));
    }

    fn push(&mut self, entry: Entry, window: &mut Window, cx_self: &mut Context<Self>) {
        let bills = self.ensure_session(cx_self);
        let reference = entry.reference.clone();
        let outcome = bills.update(cx_self, |bills, cx_self| bills.push(entry, cx_self));

        match outcome {
            PushOutcome::Duplicate(index) => {
                self.duplicate = Some(index);
                self.error = Some(
                    format!(
                        "Bill {reference} is already in this session (#{:02}).",
                        index + 1
                    )
                    .into(),
                );
            }
            PushOutcome::Added => {
                self.duplicate = None;
                self.error = None;
            }
        }
        self.clear_input(window, cx_self);
        cx_self.notify();
    }

    fn scan(&mut self, window: &mut Window, cx_self: &mut Context<Self>) {
        let raw = self.text(cx_self);
        if raw.is_empty() {
            return;
        }

        match Bill::parse(&raw, Local::now().date_naive()) {
            Some(bill) => self.push(Entry::scanned(bill), window, cx_self),
            None => {
                self.duplicate = None;
                self.error = Some(NOT_RECOGNIZED.into());
                cx_self.notify();
            }
        }
    }

    fn add_manual(&mut self, window: &mut Window, cx_self: &mut Context<Self>) {
        let raw = self.text(cx_self);
        if raw.is_empty() {
            return;
        }
        self.push(Entry::manual(raw.into()), window, cx_self);
    }

    fn remove(&mut self, index: usize, cx_self: &mut Context<Self>) {
        let Some(bills) = self.current_bills() else {
            return;
        };
        self.editing = None;
        self.duplicate = None;
        bills.update(cx_self, |bills, cx_self| bills.remove(index, cx_self));
    }

    fn is_running(&self, cx: &App) -> bool {
        self.runner.read(cx).is_running()
    }

    fn toggle_running(&mut self, cx_self: &mut Context<Self>) {
        let runner = self.runner.clone();
        if runner.read(cx_self).is_running() {
            runner.update(cx_self, |runner, cx_self| runner.stop(cx_self));
            return;
        }

        let Some(bills) = self.current_bills() else {
            return;
        };
        let Some(software) = self
            .softwares
            .iter()
            .find(|software| Some(&software.id) == self.software.as_ref())
            .cloned()
        else {
            return;
        };
        runner.update(cx_self, |runner, cx_self| {
            runner.start(Mode::Bulk, software, Some(bills.downgrade()), cx_self)
        });
    }

    fn step_back(&mut self, cx_self: &mut Context<Self>) {
        if let Some(bills) = self.current_bills() {
            bills.update(cx_self, |bills, cx_self| bills.step_back(cx_self));
        }
    }

    fn select_software(&mut self, id: SharedString, cx_self: &mut Context<Self>) {
        if self.software.as_ref() != Some(&id) {
            self.software = Some(id);
            cx_self.notify();
        }
    }

    fn software_label(&self) -> SharedString {
        self.softwares
            .iter()
            .find(|software| Some(&software.id) == self.software.as_ref())
            .map(|software| software.label.clone())
            .unwrap_or_else(|| NO_SOFTWARE.into())
    }
}

impl Render for BulkBillsPage {
    fn render(&mut self, window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();
        let mono = cx_self.theme().mono_font_family.clone();
        let focused = self
            .input
            .read(cx_self)
            .focus_handle(cx_self)
            .is_focused(window);

        let add = cx_self.listener(|this, _, window, cx_self| this.add_manual(window, cx_self));

        let composer = div()
            .h(px(COMPOSER_H))
            .pl(px(16.))
            .pr(px(8.))
            .flex()
            .items_center()
            .gap(px(16.))
            .rounded(px(8.))
            .border_1()
            .border_color(if focused {
                theme.border_focused
            } else {
                theme.border
            })
            .bg(theme.bg_surface)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Input::new(&self.input).appearance(false)),
            )
            .child(shortcut("Ctrl+Enter", "add", &theme, &mono))
            .child(
                Button::new("bulk-add")
                    .label("Add bill")
                    .icon(IconName::Plus)
                    .primary()
                    .on_click(add),
            );

        let entry_box = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(composer)
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(13.))
                        .text_color(theme.error)
                        .child(Icon::new(IconName::CircleAlert).small().text_color(theme.error))
                        .child(error),
                )
            });

        let (entries, cursor) = match self.current_bills() {
            Some(bills) => {
                let bills = bills.read(cx_self);
                (bills.entries().to_vec(), bills.cursor())
            }
            None => (Vec::new(), 0),
        };
        let count = entries.len();
        let paid = entries.iter().filter(|entry| entry.paid).count();
        let scanned = entries.iter().filter(|entry| entry.is_scanned()).count();
        let total: u32 = entries.iter().filter_map(|entry| entry.amount).sum();

        let table = if count == 0 {
            card(&theme)
                .py(px(48.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.))
                .child(
                    Icon::new(IconName::FileText)
                        .large()
                        .text_color(theme.text_disabled),
                )
                .child(
                    div()
                        .mt(px(6.))
                        .text_size(px(14.))
                        .text_color(theme.text_main)
                        .child("No bills in this session"),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child("Scan a bill or add a reference number to start the list."),
                )
        } else {
            let header = div()
                .px(px(16.))
                .py(px(10.))
                .flex()
                .items_center()
                .gap(px(16.))
                .bg(theme.bg_panel)
                .child(div().w(px(COL_INDEX)).child(column("#", &theme)))
                .child(div().w(px(COL_BILL)).child(column("Bill", &theme)))
                .child(div().flex_1().child(column("Reference", &theme)))
                .child(div().w(px(COL_STATUS)).child(column("Status", &theme)))
                .child(
                    div()
                        .w(px(COL_AMOUNT))
                        .flex()
                        .justify_end()
                        .child(column("Amount", &theme)),
                )
                .child(div().w(px(COL_ACTION)));

            let rows: Vec<_> = entries
                .iter()
                .enumerate()
                .map(|(index, entry)| self.entry_row(index, entry, cursor, &theme, &mono, cx_self))
                .collect();

            let typed = count - scanned;
            let footer = div()
                .px(px(16.))
                .py(px(14.))
                .flex()
                .items_center()
                .justify_between()
                .border_t_1()
                .border_color(theme.border_subtle)
                .bg(theme.bg_panel)
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.text_muted)
                        .child(format!(
                            "{count} {} · {paid} paid · {scanned} scanned · {typed} typed",
                            if count == 1 { "bill" } else { "bills" }
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(theme.text_muted)
                                .child("Total"),
                        )
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_main)
                                .child(rupees(total)),
                        ),
                );

            card(&theme)
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(header)
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .children(rows)
                        .overflow_y_scrollbar(),
                )
                .child(footer)
        };

        let header = self.session_header(&theme, cx_self);

        let main = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .gap(px(20.))
            .children(header)
            .child(entry_box)
            .child(table);

        let side = div()
            .w(px(SIDE_W))
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.run_panel(&theme, &mono, cx_self))
            .child(self.sessions_panel(&theme, cx_self));

        div()
            .size_full()
            .flex()
            .gap(px(24.))
            .child(main)
            .child(side)
    }
}
