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

const WINDOW_SCALE: f32 = 0.75;
const MIN_W: f32 = 960.;
const MIN_H: f32 = 600.;

impl ScanFill {
    fn screen(app: &App) -> (f32, f32) {
        app.primary_display()
            .map(|display| {
                let screen = display.bounds().size;
                (f32::from(screen.width), f32::from(screen.height))
            })
            .unwrap_or((1280., 800.))
    }

    fn min_size((screen_w, screen_h): (f32, f32)) -> (f32, f32) {
        (MIN_W.min(screen_w * 0.8), MIN_H.min(screen_h * 0.7))
    }

    pub fn bounds(app: &mut App) -> WindowBounds {
        let screen = Self::screen(app);
        let (min_w, min_h) = Self::min_size(screen);
        let fit = |screen: f32, min: f32| (screen * WINDOW_SCALE).max(min).min(screen * 0.9);
        let window = size(px(fit(screen.0, min_w)), px(fit(screen.1, min_h)));
        WindowBounds::Windowed(Bounds::centered(None, window, app))
    }

    pub fn window_options(app: &mut App) -> WindowOptions {
        let win_size = Self::bounds(app);
        let (min_w, min_h) = Self::min_size(Self::screen(app));
        WindowOptions {
            window_bounds: Some(win_size),
            window_min_size: Some(size(px(min_w), px(min_h))),
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
                        SplashEvent::Reset => {
                            match database::reset() {
                                Ok(Some(backup)) => {
                                    tracing::warn!(backup = %backup.display(), "database reset")
                                }
                                Ok(None) => tracing::warn!("database reset: nothing to move"),
                                Err(e) => tracing::error!("database reset failed: {e}"),
                            }
                            this.boot(window, cx_self);
                        }
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
