use std::time::Duration;

use crate::{
    database::{self, Session},
    screens::{DashboardScreen, LoginScreen, SplashEvent, SplashScreen},
    state::{Database, Io, Screen},
};
use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, IntoElement, Render, SharedString, TitlebarOptions,
    WindowBounds, WindowOptions, px, size,
};
use shared::DbError;

const SPLASH_MIN: Duration = Duration::from_secs(10);
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

    pub fn show_auth(&mut self, cx_self: &mut Context<Self>) {
        self.screen = Screen::Auth(LoginScreen::build_view(cx_self));
        cx_self.notify();
    }

    pub fn show_dashboard(&mut self, cx_self: &mut Context<Self>) {
        self.screen = Screen::Dashboard(DashboardScreen::build_view(cx_self));
        cx_self.notify();
    }

    pub fn boot(&mut self, cx_self: &mut Context<Self>) {
        let Screen::Splash(splash) = self.screen.clone() else {
            return;
        };

        let job = Io::spawn(cx_self, async move {
            let db = database::open().await?;
            let token = Session::get_session(db.clone()).await.ok().flatten();
            Ok::<_, DbError>((db, token))
        });

        cx_self
            .spawn(async move |this, cx| {
                let timer = cx.background_executor().timer(SPLASH_MIN);
                let result = job.await;

                let detail = match result {
                    Ok(Ok((db, token))) => {
                        timer.await;
                        let _ = this.update(cx, |this, cx_self| {
                            cx_self.set_global(Database::new(db));
                            match token {
                                Some(_) => this.show_dashboard(cx_self),
                                None => this.show_auth(cx_self),
                            }
                        });
                        return;
                    }
                    Ok(Err(e)) => {
                        eprintln!("[db] {e}");
                        e.to_string()
                    }
                    Err(e) => {
                        eprintln!("[boot] background task failed: {e}");
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

    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            let splash = SplashScreen::build_view(cx_self);
            let handle_splash = splash.clone();
            let mut app = ScanFill {
                screen: Screen::Splash(splash),
            };

            cx_self
                .subscribe(
                    &handle_splash,
                    |this: &mut ScanFill, _, event: &SplashEvent, cx_self| match event {
                        SplashEvent::Retry => this.boot(cx_self),
                    },
                )
                .detach();

            app.boot(cx_self);
            app
        })
    }
}

impl Render for ScanFill {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        _cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl IntoElement {
        self.screen.element()
    }
}

