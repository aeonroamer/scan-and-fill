use gpui_kit::{App, Global};
use tokio::{
    runtime::{Builder, Runtime},
    task::JoinHandle,
};

pub struct Io(Runtime);
impl Global for Io {}

impl Io {
    pub fn provide(cx: &mut App) {
        let rt = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("scanfill-io")
            .build()
            .expect("couldn't start background workers");
        cx.set_global(Io(rt));
    }

    pub fn spawn<F>(cx: &App, fut: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        cx.global::<Self>().0.spawn(fut)
    }
}
