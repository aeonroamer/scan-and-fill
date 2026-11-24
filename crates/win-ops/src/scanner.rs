use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use shared::{Trigger, WindowRule};

use crate::{Detected, Detector, DetectorConfig, Hook, HookEvent};

const IDLE_WAIT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanEvent {
    Scanned(String),
    Human { text: String, enter: bool },
    Trigger,
    Focus(bool),
}

pub struct Scanner {
    hook: Option<Hook>,
    worker: Option<JoinHandle<()>>,
}

impl Scanner {
    pub fn start(
        rule: WindowRule,
        trigger: Option<Trigger>,
        config: DetectorConfig,
        mut on_event: impl FnMut(ScanEvent) + Send + 'static,
    ) -> Option<Self> {
        let (tx, rx) = mpsc::channel();
        let hook = Hook::start(rule, trigger, tx)?;

        let worker = thread::Builder::new()
            .name("key-detector".into())
            .spawn(move || {
                let mut detector = Detector::new(config);
                loop {
                    let wait = detector
                        .deadline()
                        .map(|deadline| deadline.saturating_duration_since(Instant::now()))
                        .unwrap_or(IDLE_WAIT);

                    match rx.recv_timeout(wait) {
                        Ok(HookEvent::Key(key, at)) => {
                            if let Some(detected) = detector.push(key, at) {
                                on_event(detected.into());
                            }
                        }
                        Ok(HookEvent::Trigger) => on_event(ScanEvent::Trigger),
                        Ok(HookEvent::Focus(active)) => {
                            if !active {
                                detector.clear();
                                Hook::clear_buffered();
                            }
                            on_event(ScanEvent::Focus(active));
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            if let Some(detected) = detector.poll(Instant::now()) {
                                Hook::clear_buffered();
                                on_event(detected.into());
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .ok()?;

        Some(Self {
            hook: Some(hook),
            worker: Some(worker),
        })
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(hook) = self.hook.take() {
            hook.stop();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl From<Detected> for ScanEvent {
    fn from(detected: Detected) -> Self {
        match detected {
            Detected::Scanned(code) => Self::Scanned(code),
            Detected::Human { text, enter } => Self::Human { text, enter },
        }
    }
}
