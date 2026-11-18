use std::time::Duration;

use crate::{
    database::{self, Session},
    screens::{DashboardScreen, LoginScreen, SplashEvent, SplashExt, SplashScreen},
    state::{Database, Io, Screen},
};
use gpui_kit::{
    App, AppContext, AsyncApp, Bounds, Context, Entity, IntoElement, Render, SharedString,
    TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use tokio::runtime::Builder;

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

    pub fn show_auth(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Auth(LoginScreen::build_view(cx));
        cx.notify();
    }

    pub fn show_dashboard(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Dashboard(DashboardScreen::build_view(cx));
        cx.notify();
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        let Screen::Splash(splash) = self.screen.clone() else {
            return;
        };

        let job = Io::spawn(cx, async move {
            let db = database::open().await?;
            let token = Session::get_session(db.clone()).await.ok().flatten();
            Ok::<_, SharedString>((db, token))
        });

        cx.spawn(async move |this, cx| {
            let timer = cx.background_executor().timer(Duration::from_millis(1000));
            let Ok(Ok((db, token))) = job.await else {
                splash.set_error(cx, "Couldn't start Scan & Fill.");
                return;
            };

            let _ = this.update(cx, |this, cx| {
                cx.set_global(Database::new(db));
                match token {
                    Some(_) => this.show_dashboard(cx),
                    None => this.show_auth(cx),
                }
            });
            timer.await;
        })
        .detach();
    }

    pub fn build_view(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let splash = SplashScreen::build_view(cx);
            let handle_splash = splash.clone();
            let mut app = ScanFill {
                screen: Screen::Splash(splash),
            };

            cx.subscribe(
                &handle_splash,
                |this: &mut ScanFill, _, event: &SplashEvent, cx| match event {
                    SplashEvent::Retry => this.boot(cx),
                },
            )
            .detach();

            app.boot(cx);
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
