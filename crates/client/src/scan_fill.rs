use std::time::Duration;

use crate::{
    database::{self, DbError, Settings},
    screens::{DashboardScreen, SplashEvent, SplashScreen},
    state::{AppSettings, Database, Io, Screen},
};
use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, IntoElement, Render, SharedString, TitlebarOptions,
    Window, WindowBounds, WindowOptions, px, size,
};

const SPLASH_MIN: Duration = Duration::from_secs(3);
const BOOT_FAILED: &str = "Something went wrong. Try again.";

#[derive(Debug, Clone)]
pub struct ScanFill {
    screen: Screen,
}

impl ScanFill {
    pub fn bounds(app: &mut App) -> WindowBounds {
        let bounds = Bounds::centered(None, size(px(1280.), px(800.0)), app);
        WindowBounds::Windowed(bounds)
    }

    pub fn window_options(app: &mut App) -> WindowOptions {
        let win_size = Self::bounds(app);
        WindowOptions {
            window_bounds: Some(win_size),
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::new("Scan & Fill")),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn show_dashboard(&mut self, window: &mut Window, cx_self: &mut Context<Self>) {
        self.screen = Screen::Dashboard(DashboardScreen::build_view(window, cx_self));
        cx_self.notify();
    }

    pub fn boot(&mut self, window: &mut Window, cx_self: &mut Context<Self>) {
        let Screen::Splash(splash) = self.screen.clone() else {
            return;
        };

        let job = Io::spawn(cx_self, async move {
            let db = database::open().await?;
            let settings = database::load_settings(&db).await.unwrap_or_else(|e| {
                tracing::error!("loading settings failed: {e}");
                Settings::default()
            });
            Ok::<_, DbError>((db, settings))
        });

        cx_self
            .spawn_in(window, async move |this, cx| {
                let timer = cx.background_executor().timer(SPLASH_MIN);
                let result = job.await;

                let detail = match result {
                    Ok(Ok((db, settings))) => {
                        timer.await;
                        let _ = this.update_in(cx, |this, window, cx_self| {
                            cx_self.set_global(Database::new(db));
                            AppSettings::provide(settings, cx_self);
                            this.show_dashboard(window, cx_self);
                        });
                        return;
                    }
                    Ok(Err(e)) => {
                        tracing::error!("database open failed: {e}");
                        e.to_string()
                    }
                    Err(e) => {
                        tracing::error!("boot task failed: {e}");
                        e.to_string()
                    }
                };

                let _ = splash.update(cx, |splash, cx_self| {
                    splash.set_error(BOOT_FAILED);
                    splash.set_detail(detail);
                    cx_self.notify();
                });
            })
            .detach();
    }

    pub fn build_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let splash = SplashScreen::build_view(cx_self);
            let handle_splash = splash.clone();
            let mut app = ScanFill {
                screen: Screen::Splash(splash),
            };

            cx_self
                .subscribe_in(
                    &handle_splash,
                    window,
                    |this: &mut ScanFill, _, event: &SplashEvent, window, cx_self| match event {
                        SplashEvent::Retry => this.boot(window, cx_self),
                    },
                )
                .detach();

            app.boot(window, cx_self);
            app
        })
    }
}

impl Render for ScanFill {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl IntoElement {
        self.screen.element()
    }
}
