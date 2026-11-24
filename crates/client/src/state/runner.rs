use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use gpui_kit::{App, AppContext, Context, Entity, Global, SharedString, WeakEntity};
use shared::Software;

use super::{AppSettings, BillList, RunState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Single,
    Bulk,
}

pub struct Job {
    mode: Mode,
    stop: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

pub struct Runner {
    software: Option<Software>,
    contact: Option<SharedString>,
    bills: Option<WeakEntity<BillList>>,
    job: Option<Job>,
}

pub struct ActiveRunner(Entity<Runner>);

impl Global for ActiveRunner {}

impl Runner {
    pub fn provide(cx: &mut App) {
        let runner = cx.new(|_| Self {
            software: None,
            contact: None,
            bills: None,
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
        cx_self: &mut Context<Self>,
    ) {
        if self.job.is_some() {
            return;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let handle = thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
            }
        });

        self.software = Some(software);
        self.contact = Some(AppSettings::get(cx_self).contact().to_string().into());
        self.bills = bills;
        self.job = Some(Job {
            mode,
            stop,
            _handle: handle,
        });
        RunState::set(true, cx_self);
        cx_self.notify();
    }

    pub fn stop(&mut self, cx_self: &mut Context<Self>) {
        if let Some(job) = self.job.take() {
            job.stop.store(true, Ordering::Relaxed);
        }
        RunState::set(false, cx_self);
        cx_self.notify();
    }
}
