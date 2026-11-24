use crate::pages::{BulkBillsPage, SettingsPage, SingleBillPage};
use crate::state::{Route, RunState, Theme, alpha};
use gpui_kit::component::{Icon, Sizable, scroll::ScrollableElement};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AppContext, Context, Div, Entity, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

const APP_NAME: &str = "Scan & Fill";
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

const SIDEBAR_W: f32 = 232.0;
const HEADER_H: f32 = 56.0;

const BILL_ROUTES: [Route; 2] = [Route::SingleBill, Route::BulkBill];

pub struct DashboardScreen {
    route: Route,
    single_bill: Entity<SingleBillPage>,
    bulk_bills: Entity<BulkBillsPage>,
    settings: Entity<SettingsPage>,
}

impl DashboardScreen {
    pub fn build_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx_self| {
            cx_self
                .observe_global::<RunState>(|_, cx_self| cx_self.notify())
                .detach();
            Self {
            route: Route::SingleBill,
            single_bill: SingleBillPage::build_view(cx_self),
            bulk_bills: BulkBillsPage::build_view(window, cx_self),
            settings: SettingsPage::build_view(window, cx_self),
            }
        })
    }

    fn nav_item(&self, route: Route, theme: &Theme, cx_self: &mut Context<Self>) -> Stateful<Div> {
        let active = self.route == route;
        let locked = !active && RunState::is_running(cx_self);

        let (text, icon, bg) = if active {
            (theme.text_main, theme.accent, theme.bg_selected)
        } else {
            (theme.text_muted, theme.text_muted, alpha(theme.bg_panel, 0.0))
        };
        let hover_bg = theme.bg_hover;
        let hover_text = theme.text_main;

        div()
            .id(route.id())
            .h(px(34.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(10.))
            .rounded(px(6.))
            .when(!locked, |el| el.cursor_pointer())
            .when(locked, |el| el.opacity(0.4))
            .bg(bg)
            .text_size(px(14.))
            .text_color(text)
            .when(active, |el| el.font_weight(FontWeight::MEDIUM))
            .when(!active && !locked, |el| {
                el.hover(move |s| s.bg(hover_bg).text_color(hover_text))
            })
            .child(Icon::new(route.icon()).small().text_color(icon))
            .child(route.title())
            .on_click(cx_self.listener(move |this, _, _, cx_self| {
                if this.route != route && !RunState::is_running(cx_self) {
                    this.route = route;
                    cx_self.notify();
                }
            }))
    }
}

impl Render for DashboardScreen {
    fn render(&mut self, _window: &mut Window, cx_self: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx_self.global::<Theme>();

        let brand = div()
            .px(px(10.))
            .pt(px(8.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(div().w(px(20.)).h(px(2.)).rounded_full().bg(theme.accent))
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_main)
                    .child(APP_NAME),
            );

        let bills_nav = div()
            .mt(px(28.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .px(px(10.))
                    .pb(px(6.))
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child("Bills"),
            )
            .children(
                BILL_ROUTES
                    .into_iter()
                    .map(|route| self.nav_item(route, &theme, cx_self)),
            );

        let footer = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(self.nav_item(Route::Settings, &theme, cx_self))
            .child(
                div()
                    .px(px(10.))
                    .pt(px(12.))
                    .text_size(px(12.))
                    .text_color(theme.text_muted)
                    .child(VERSION),
            );

        let sidebar = div()
            .w(px(SIDEBAR_W))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .px(px(12.))
            .py(px(16.))
            .bg(theme.bg_panel)
            .border_r_1()
            .border_color(theme.border_subtle)
            .child(brand)
            .child(bills_nav)
            .child(div().flex_1())
            .child(footer);

        let header = div()
            .h(px(HEADER_H))
            .flex_none()
            .px(px(32.))
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border_subtle)
            .text_size(px(16.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text_main)
            .child(self.route.title());

        let page = match self.route {
            Route::SingleBill => self.single_bill.clone().into_any_element(),
            Route::BulkBill => self.bulk_bills.clone().into_any_element(),
            Route::Settings => self.settings.clone().into_any_element(),
        };

        let body = div()
            .flex_1()
            .min_h_0()
            .p(px(32.))
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(page)
                    .overflow_x_scrollbar(),
            );

        let content = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .child(header)
            .child(body);

        div()
            .size_full()
            .flex()
            .bg(theme.bg_app)
            .child(sidebar)
            .child(content)
    }
}
