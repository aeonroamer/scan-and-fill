use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_channel::Sender;
use chrono::Local;
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, Global, SharedString, Task, WeakEntity,
};
use shared::{Bill, Key, Software, Step};
use win_ops::{DetectorConfig, Missing, ScanEvent, Scanner, Typist, resolve};

use super::{AppSettings, BillList, RunState};

const REFERENCE: &str = "reference";
const AMOUNT: &str = "amount";
const UTILITY: &str = "utility";
const COMPANY: &str = "company";
const CONTACT: &str = "contact";

fn beep(cx: &App) {
    if AppSettings::get(cx).sound.unwrap_or(true) {
        win_ops::play_reject();
    }
}

fn bill_field(
    software: &Software,
    field: &str,
    reference: &str,
    amount: Option<u32>,
    utility: Option<&str>,
    company: Option<&str>,
) -> Option<String> {
    match field {
        REFERENCE => Some(reference.to_string()),
        AMOUNT => amount.map(|amount| amount.to_string()),
        UTILITY => utility
            .and_then(|utility| software.pick(UTILITY, utility))
            .map(|keys| keys.to_string()),
        COMPANY => company
            .and_then(|company| software.pick(COMPANY, company))
            .map(|keys| keys.to_string()),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Single,
    Bulk,
}

#[derive(Clone, Debug)]
pub enum RunnerEvent {
    Typed(Bill),
    Rejected(String),
    Failed(String),
    Focus(bool),
    Cleared,
}

enum WorkerMessage {
    Event(RunnerEvent),
    TypeNext,
}

pub struct Job {
    mode: Mode,
    stop: Arc<AtomicBool>,
    delay: Duration,
    typing: bool,
    batch: usize,
    remaining: usize,
    _scanner: Option<Scanner>,
    _task: Option<Task<()>>,
}

pub struct Runner {
    software: Option<Software>,
    contact: Option<SharedString>,
    bills: Option<WeakEntity<BillList>>,
    focused: bool,
    job: Option<Job>,
    paying: Option<Paying>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paying {
    pub utility: SharedString,
    pub company: SharedString,
}

impl EventEmitter<RunnerEvent> for Runner {}

pub struct ActiveRunner(Entity<Runner>);

impl Global for ActiveRunner {}

impl Runner {
    pub fn provide(cx: &mut App) {
        let runner = cx.new(|_| Self {
            software: None,
            contact: None,
            bills: None,
            paying: None,
            focused: false,
            job: None,
        });
        cx.set_global(ActiveRunner(runner));
    }

    pub fn get(cx: &App) -> Entity<Self> {
        cx.global::<ActiveRunner>().0.clone()
    }

    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }

    pub fn mode(&self) -> Option<Mode> {
        self.job.as_ref().map(|job| job.mode)
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn software(&self) -> Option<&Software> {
        self.software.as_ref()
    }

    pub fn contact(&self) -> Option<&SharedString> {
        self.contact.as_ref()
    }

    pub fn bills(&self) -> Option<&WeakEntity<BillList>> {
        self.bills.as_ref()
    }

    pub fn start(
        &mut self,
        mode: Mode,
        software: Software,
        bills: Option<WeakEntity<BillList>>,
        batch: usize,
        paying: Option<Paying>,
        cx_self: &mut Context<Self>,
    ) {
        if self.job.is_some() {
            return;
        }

        let settings = AppSettings::get(cx_self).clone();
        let contact = settings.contact().to_string();
        let delay = settings
            .step_delay_ms
            .map(|ms| Duration::from_millis(ms as u64))
            .unwrap_or(software.step_delay);
        let stop = Arc::new(AtomicBool::new(false));

        let (tx, rx) = async_channel::unbounded();
        let worker = Worker {
            mode,
            software: software.clone(),
            paying: paying.clone(),
            contact: contact.clone(),
            delay,
            stop: stop.clone(),
            out: tx,
        };
        let scanner = Scanner::start(
            software.window.clone(),
            Some(settings.trigger()),
            DetectorConfig::default(),
            move |event| worker.handle(event),
        );
        let Some(scanner) = scanner else {
            tracing::error!("couldn't install the keyboard hook");
            cx_self.emit(RunnerEvent::Failed(
                "Couldn't start listening to the keyboard.".into(),
            ));
            return;
        };
        let task = cx_self.spawn(async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let _ = this.update(cx, |runner, cx_self| match message {
                    WorkerMessage::Event(event) => runner.receive(event, cx_self),
                    WorkerMessage::TypeNext => runner.trigger(cx_self),
                });
            }
        });

        tracing::info!(?mode, software = %software.label, "runner started");
        self.software = Some(software);
        self.contact = Some(contact.into());
        self.bills = bills;
        self.paying = paying;
        self.focused = false;
        self.job = Some(Job {
            mode,
            stop,
            delay,
            typing: false,
            batch: batch.clamp(1, self.software.as_ref().map_or(1, |software| software.max_batch.max(1))),
            remaining: 0,
            _scanner: Some(scanner),
            _task: Some(task),
        });
        RunState::set(true, cx_self);
        cx_self.notify();
    }

    pub fn stop(&mut self, cx_self: &mut Context<Self>) {
        if let Some(job) = self.job.take() {
            job.stop.store(true, Ordering::Relaxed);
            tracing::info!(mode = ?job.mode, "runner stopped");
        }
        self.focused = false;
        RunState::set(false, cx_self);
        cx_self.notify();
    }

    fn trigger(&mut self, cx_self: &mut Context<Self>) {
        let Some(job) = self.job.as_mut() else {
            return;
        };
        if job.typing {
            tracing::info!("trigger ignored, still typing");
            beep(cx_self);
            return;
        }
        job.remaining = job.batch;
        cx_self.emit(RunnerEvent::Cleared);
        self.type_next(cx_self);
    }

    fn type_next(&mut self, cx_self: &mut Context<Self>) {
        let (Some(job), Some(software), Some(bills)) = (
            self.job.as_mut(),
            self.software.as_ref(),
            self.bills.clone(),
        ) else {
            return;
        };
        if job.typing || job.remaining == 0 {
            return;
        }
        if !self.focused {
            tracing::info!("portal lost focus, batch stopped");
            job.remaining = 0;
            return;
        }
        let Some(list) = bills.upgrade() else {
            return;
        };
        let Some((reference, amount)) = list
            .read(cx_self)
            .next()
            .map(|(_, entry)| (entry.reference.to_string(), entry.amount))
        else {
            tracing::info!("no unpaid bill at the cursor");
            return;
        };
        let (utility, company) = match &self.paying {
            Some(paying) => (Some(paying.utility.to_string()), Some(paying.company.to_string())),
            None => (None, None),
        };

        let contact = self
            .contact
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default();
        let actions = resolve(
            &software.steps,
            |field| {
                bill_field(
                    software,
                    field,
                    &reference,
                    amount,
                    utility.as_deref(),
                    company.as_deref(),
                )
            },
            |setting| (setting == CONTACT).then(|| contact.clone()),
        );
        let actions = match actions {
            Ok(actions) => actions,
            Err(missing) => {
                let what = match &missing {
                    Missing::Field(name) | Missing::Setting(name) => name.clone(),
                };
                tracing::error!(%reference, %what, "bulk typing stopped, bill is incomplete");
                job.remaining = 0;
                beep(cx_self);
                cx_self.emit(RunnerEvent::Failed(format!(
                    "Stopped at {reference}: {what} is not set for {}. Stop, choose it in the panel, then start again.",
                    software.label
                )));
                return;
            }
        };

        job.typing = true;
        job.remaining -= 1;
        let stop = job.stop.clone();
        let delay = job.delay;
        cx_self
            .spawn(async move |this, cx| {
                let finished = cx
                    .background_executor()
                    .spawn(async move { Typist::new(delay).with_stop(&stop).run(&actions) })
                    .await;
                let _ = this.update(cx, |runner, cx_self| {
                    if let Some(job) = runner.job.as_mut() {
                        job.typing = false;
                    }
                    if finished {
                        tracing::info!(%reference, "bulk bill typed");
                        let _ = bills.update(cx_self, |bills, cx_self| {
                            let index = bills
                                .entries()
                                .iter()
                                .position(|entry| entry.reference == reference.as_str());
                            if let Some(index) = index {
                                bills.set_paid(index, true, cx_self);
                            }
                        });
                        runner.type_next(cx_self);
                    } else if let Some(job) = runner.job.as_mut() {
                        job.remaining = 0;
                    }
                });
            })
            .detach();
    }

    fn receive(&mut self, event: RunnerEvent, cx_self: &mut Context<Self>) {
        match &event {
            RunnerEvent::Typed(bill) => {
                tracing::info!(reference = %bill.reference, amount = bill.amount, "bill typed")
            }
            RunnerEvent::Rejected(raw) => {
                tracing::warn!(%raw, "barcode not recognized");
                beep(cx_self);
            }
            RunnerEvent::Failed(reason) => tracing::error!(%reason, "typing failed"),
            RunnerEvent::Focus(focused) => tracing::info!(focused, "portal focus changed"),
            RunnerEvent::Cleared => {}
        }
        if let RunnerEvent::Focus(focused) = event {
            self.focused = focused;
            cx_self.notify();
        }
        cx_self.emit(event);
    }
}

struct Worker {
    mode: Mode,
    software: Software,
    paying: Option<Paying>,
    contact: String,
    delay: Duration,
    stop: Arc<AtomicBool>,
    out: Sender<WorkerMessage>,
}

impl Worker {
    fn handle(&self, event: ScanEvent) {
        match event {
            ScanEvent::Scanned(raw) if self.mode == Mode::Single => self.scanned(raw),
            ScanEvent::Scanned(raw) => self.send(RunnerEvent::Rejected(raw)),
            ScanEvent::Human { text, enter } => {
                let typist = self.typist();
                if typist.text(&text) && enter {
                    typist.key(Key::Enter);
                }
            }
            ScanEvent::Trigger if self.mode == Mode::Bulk => {
                let _ = self.out.send_blocking(WorkerMessage::TypeNext);
            }
            ScanEvent::Trigger => self.continue_after_reference(),
            ScanEvent::Focus(focused) => self.send(RunnerEvent::Focus(focused)),
        }
    }

    fn scanned(&self, raw: String) {
        let Some(bill) = Bill::parse(&raw, Local::now().date_naive()) else {
            self.send(RunnerEvent::Rejected(raw));
            return;
        };

        let paying = self.paying.as_ref();
        let actions = resolve(
            &self.software.steps,
            |field| {
                bill_field(
                    &self.software,
                    field,
                    &bill.reference,
                    Some(bill.amount),
                    paying.map(|paying| &*paying.utility),
                    paying.map(|paying| &*paying.company),
                )
            },
            |setting| (setting == CONTACT).then(|| self.contact.clone()),
        );

        match actions {
            Ok(actions) => {
                if self.typist().run(&actions) {
                    self.send(RunnerEvent::Typed(bill));
                }
            }
            Err(missing) => self.send(RunnerEvent::Failed(format!("Missing {missing:?}"))),
        }
    }

    fn continue_after_reference(&self) {
        let start = self
            .software
            .steps
            .iter()
            .position(|step| matches!(step, Step::Field(name) if name == REFERENCE))
            .map(|index| index + 1)
            .unwrap_or(0);

        let actions = resolve(
            &self.software.steps[start..],
            |_| None,
            |setting| (setting == CONTACT).then(|| self.contact.clone()),
        );

        match actions {
            Ok(actions) => {
                self.typist().run(&actions);
            }
            Err(missing) => self.send(RunnerEvent::Failed(format!("Missing {missing:?}"))),
        }
    }

    fn typist(&self) -> Typist<'_> {
        Typist::new(self.delay).with_stop(&self.stop)
    }

    fn send(&self, event: RunnerEvent) {
        let _ = self.out.send_blocking(WorkerMessage::Event(event));
    }
}
