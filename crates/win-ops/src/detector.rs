use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct DetectorConfig {
    pub max_average_gap: Duration,
    pub min_length: usize,
    pub idle: Duration,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            max_average_gap: Duration::from_millis(30),
            min_length: 8,
            idle: Duration::from_millis(80),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyInput {
    Char(char),
    Enter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Detected {
    Scanned(String),
    Human { text: String, enter: bool },
}

pub struct Detector {
    config: DetectorConfig,
    buffer: String,
    first: Option<Instant>,
    last: Option<Instant>,
}

impl Detector {
    pub fn new(config: DetectorConfig) -> Self {
        Self {
            config,
            buffer: String::new(),
            first: None,
            last: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.last.map(|last| last + self.config.idle)
    }

    pub fn push(&mut self, key: KeyInput, at: Instant) -> Option<Detected> {
        match key {
            KeyInput::Char(ch) => {
                self.buffer.push(ch);
                self.first.get_or_insert(at);
                self.last = Some(at);
                None
            }
            KeyInput::Enter if self.buffer.is_empty() => None,
            KeyInput::Enter => {
                let detected = if self.looks_scanned() {
                    Detected::Scanned(std::mem::take(&mut self.buffer))
                } else {
                    Detected::Human {
                        text: std::mem::take(&mut self.buffer),
                        enter: true,
                    }
                };
                self.reset();
                Some(detected)
            }
        }
    }

    pub fn poll(&mut self, now: Instant) -> Option<Detected> {
        let deadline = self.deadline()?;
        if now < deadline || self.buffer.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut self.buffer);
        self.reset();
        Some(Detected::Human { text, enter: false })
    }

    fn looks_scanned(&self) -> bool {
        let count = self.buffer.chars().count();
        if count < self.config.min_length {
            return false;
        }
        let (Some(first), Some(last)) = (self.first, self.last) else {
            return false;
        };
        let average = last.duration_since(first) / (count as u32 - 1).max(1);
        average <= self.config.max_average_gap
    }

    pub fn clear(&mut self) {
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.first = None;
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(detector: &mut Detector, text: &str, start: Instant, gap_ms: u64) -> Instant {
        let mut at = start;
        for ch in text.chars() {
            assert_eq!(detector.push(KeyInput::Char(ch), at), None);
            at += Duration::from_millis(gap_ms);
        }
        at
    }

    #[test]
    fn fast_burst_with_enter_is_a_scan() {
        let mut detector = Detector::new(DetectorConfig::default());
        let start = Instant::now();
        let end = feed(&mut detector, "041277812290110000", start, 5);
        assert_eq!(
            detector.push(KeyInput::Enter, end),
            Some(Detected::Scanned("041277812290110000".into()))
        );
        assert!(detector.is_empty());
    }

    #[test]
    fn slow_typing_with_enter_is_human() {
        let mut detector = Detector::new(DetectorConfig::default());
        let start = Instant::now();
        let end = feed(&mut detector, "0412778122", start, 150);
        assert_eq!(
            detector.push(KeyInput::Enter, end),
            Some(Detected::Human {
                text: "0412778122".into(),
                enter: true
            })
        );
    }

    #[test]
    fn short_fast_burst_is_human() {
        let mut detector = Detector::new(DetectorConfig::default());
        let start = Instant::now();
        let end = feed(&mut detector, "12", start, 5);
        assert!(matches!(
            detector.push(KeyInput::Enter, end),
            Some(Detected::Human { .. })
        ));
    }

    #[test]
    fn idle_flushes_typed_text_without_enter() {
        let mut detector = Detector::new(DetectorConfig::default());
        let start = Instant::now();
        let end = feed(&mut detector, "ab", start, 120);
        assert_eq!(detector.poll(end - Duration::from_millis(120)), None);
        assert_eq!(
            detector.poll(end + Duration::from_millis(200)),
            Some(Detected::Human {
                text: "ab".into(),
                enter: false
            })
        );
        assert!(detector.deadline().is_none());
    }

    #[test]
    fn enter_alone_is_ignored() {
        let mut detector = Detector::new(DetectorConfig::default());
        assert_eq!(detector.push(KeyInput::Enter, Instant::now()), None);
    }
}
