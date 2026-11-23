use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(3);

pub fn spawn() {
    let spawned = thread::Builder::new()
        .name("focus-watch".into())
        .spawn(|| {
            loop {
                match win_ops::focused_window() {
                    Some(info) => tracing::info!(
                        title = %info.title,
                        exe = info.exe_name().unwrap_or("unknown"),
                        pid = info.process_id,
                        "focused window"
                    ),
                    None => tracing::info!("no focused window"),
                }
                thread::sleep(INTERVAL);
            }
        });

    if let Err(e) = spawned {
        tracing::error!("couldn't start focus watcher: {e}");
    }
}
