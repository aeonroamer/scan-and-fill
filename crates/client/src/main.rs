mod database;
mod focus_watch;
mod logging;
mod pages;
mod scan_fill;
mod screens;
mod state;

use crate::state::{Io, Runner, Theme};
use gpui_kit::{AppContext, assets::Assets, base::Root};
use scan_fill::ScanFill;

fn main() {
    let _log_guard = logging::init();
    tracing::info!("starting Scan & Fill {}", env!("CARGO_PKG_VERSION"));
    // focus_watch::spawn();

    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        Theme::provide(cx);
        Io::provide(cx);
        Runner::provide(cx);

        let win_ops = ScanFill::window_options(cx);
        gpui_kit::open_window(win_ops, cx, |window, cx| {
            let _ = Theme::follow_system(window);
            let app = ScanFill::build_view(window, cx);
            cx.new(|cx| Root::new(app, window, cx))
        })
        .expect("failed to open window");
    })
}
