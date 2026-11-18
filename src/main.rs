mod scan_fill;
mod database;
mod screens;
mod state;

use crate::state::{Io, Theme};
use scan_fill::ScanFill;
use gpui_kit::{AppContext, assets::Assets, base::Root};

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        Theme::provide(cx);
        Io::provide(cx);

        let win_ops = ScanFill::window_options(cx);
        gpui_kit::open_window(win_ops, cx, |window, cx| {
            Theme::follow_system(window);
            let app = ScanFill::build_view(cx);
            cx.new(|cx| Root::new(app, window, cx))
        })
        .expect("failed to open window");
    })
}
