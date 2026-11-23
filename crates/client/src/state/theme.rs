use gpui_kit::{
    App, Background, Global, Rgba, Window, WindowAppearance,
    component::{Theme as KitTheme, ThemeMode},
    linear_color_stop, linear_gradient, px, rgba,
};

pub const fn hex(rgb: u32) -> Rgba {
    Rgba {
        r: ((rgb >> 16) & 0xff) as f32 / 255.0,
        g: ((rgb >> 8) & 0xff) as f32 / 255.0,
        b: (rgb & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

/// Same color, different opacity (0.0 - 1.0).
pub const fn alpha(c: Rgba, a: f32) -> Rgba {
    Rgba { a, ..c }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub is_dark: bool,

    // Surface hierarchy (back -> front)
    pub bg_app: Rgba,
    pub bg_sunken: Rgba,
    pub bg_panel: Rgba,
    pub bg_surface: Rgba,
    pub bg_overlay: Rgba,
    pub bg_hover: Rgba,
    pub bg_active: Rgba,
    pub bg_selected: Rgba,
    pub bg_disabled: Rgba,
    pub bg_scrim: Rgba, // dim layer behind modals

    // Borders
    pub border: Rgba,
    pub border_subtle: Rgba,
    pub border_strong: Rgba,
    pub border_focused: Rgba,
    pub focus_ring: Rgba,

    // Typography
    pub text_main: Rgba,
    pub text_muted: Rgba,
    pub text_disabled: Rgba,
    pub text_on_accent: Rgba,
    pub text_link: Rgba,
    pub text_selection: Rgba,

    // Accent
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_active: Rgba,
    pub accent_subtle: Rgba,

    // Semantics (solid + tinted background for banners, badges, toasts)
    pub success: Rgba,
    pub success_subtle: Rgba,
    pub warning: Rgba,
    pub warning_subtle: Rgba,
    pub error: Rgba,
    pub error_subtle: Rgba,
    pub info: Rgba,
    pub info_subtle: Rgba,

    // Gradient (light mode uses a darker pair)
    pub grad_start: Rgba,
    pub grad_end: Rgba,
    pub text_on_gradient: Rgba,

    // Effects
    pub shadow: Rgba,
    pub scrollbar_thumb: Rgba,
    pub scrollbar_thumb_hover: Rgba,

    // Spectrum
    pub cyan: Rgba,
    pub green: Rgba,
    pub orange: Rgba,
    pub purple: Rgba,
    pub red: Rgba,
    pub blue: Rgba,
}

impl Global for Theme {}

/// `cx.app_theme()` from anywhere you have an `App` / `Context<T>`.
/// (Named so it won't clash with gpui-component's own `ActiveTheme::theme()`.)
pub trait AppThemeExt {
    fn app_theme(&self) -> &Theme;
}

impl AppThemeExt for App {
    fn app_theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl Theme {
    /// Call once at startup, after `gpui_kit::init` and before opening windows.
    pub fn provide(cx: &mut App) {
        Self::set(Self::for_appearance(cx.window_appearance()), cx);
    }

    /// Call once per window if you want the theme to follow the OS
    /// when the user switches between light and dark.
    pub fn follow_system(window: &mut Window) {
        window
            .observe_window_appearance(|window, cx| {
                Self::set(Self::for_appearance(window.appearance()), cx);
            })
            .detach();
    }

    /// The one way to change the theme: stores ours and copies it into
    /// gpui-kit's, so stock components and our own views always match.
    pub fn set(theme: Self, cx: &mut App) {
        cx.set_global(theme);
        theme.apply_to_components(cx);
    }

    /// Copies our palette onto gpui-kit's component theme.
    ///
    /// `KitTheme::change` reloads gpui-kit's built-in light/dark theme and
    /// overwrites earlier edits, so it must run first and `update` second.
    /// `update` also refreshes every window.
    fn apply_to_components(&self, cx: &mut App) {
        let t = *self;
        let mode = if t.is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };

        KitTheme::change(mode, None, cx);
        KitTheme::update(cx, |k| {
            // Surfaces and text
            k.background = t.bg_app.into();
            k.foreground = t.text_main.into();
            k.muted = t.bg_panel.into();
            k.muted_foreground = t.text_muted.into();
            k.popover = t.bg_overlay.into();
            k.popover_foreground = t.text_main.into();
            k.title_bar = t.bg_app.into();
            k.title_bar_border = t.border_subtle.into();

            // Borders, inputs, focus
            k.border = t.border.into();
            k.input = t.border_strong.into();
            k.ring = t.focus_ring.into();
            k.caret = t.accent.into();
            k.selection = t.text_selection.into();

            // Primary = our accent (filled buttons, switches, sliders)
            k.primary = t.accent.into();
            k.primary_hover = t.accent_hover.into();
            k.primary_active = t.accent_active.into();
            k.primary_foreground = t.text_on_accent.into();
            k.button_primary = t.accent.into();
            k.button_primary_hover = t.accent_hover.into();
            k.button_primary_active = t.accent_active.into();
            k.button_primary_foreground = t.text_on_accent.into();

            // Secondary = quiet surface buttons
            k.secondary = t.bg_surface.into();
            k.secondary_hover = t.bg_hover.into();
            k.secondary_active = t.bg_active.into();
            k.secondary_foreground = t.text_main.into();

            // gpui-kit's "accent" is the hover/highlight background, not a brand color
            k.accent = t.bg_hover.into();
            k.accent_foreground = t.text_main.into();

            // Lists
            k.list_hover = t.bg_hover.into();
            k.list_active = t.bg_selected.into();
            k.list_active_border = t.accent.into();

            // Links
            k.link = t.text_link.into();
            k.link_hover = t.accent_hover.into();
            k.link_active = t.accent_active.into();

            // Status
            k.danger = t.error.into();
            k.success = t.success.into();
            k.warning = t.warning.into();
            k.info = t.info.into();

            // Scrollbars
            k.scrollbar_thumb = t.scrollbar_thumb.into();
            k.scrollbar_thumb_hover = t.scrollbar_thumb_hover.into();

            // Shape
            k.radius = px(6.);
            k.radius_lg = px(10.);
        });
    }

    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        }
    }

    /// 135deg, like the web's `linear-gradient(135deg, start, end)`
    pub fn gradient(&self) -> Background {
        linear_gradient(
            135.,
            linear_color_stop(self.grad_start, 0.),
            linear_color_stop(self.grad_end, 1.),
        )
    }

    pub fn dark() -> Self {
        let accent = hex(0xa78bfa);
        let success = hex(0x6fcf8f);
        let warning = hex(0xe0b45c);
        let error = hex(0xf08a80);
        let info = hex(0x7cb7ff);
        let white = hex(0xffffff);
        let black = hex(0x000000);

        Self {
            is_dark: true,

            bg_app: hex(0x1a1820),
            bg_sunken: hex(0x15131a),
            bg_panel: hex(0x1d1b24),
            bg_surface: hex(0x22202a),
            bg_overlay: hex(0x292631),
            bg_hover: hex(0x2c2935),
            bg_active: hex(0x34313e),
            bg_selected: alpha(accent, 0.16),
            bg_disabled: alpha(white, 0.03),
            bg_scrim: alpha(black, 0.60),

            border: hex(0x302d3a),
            border_subtle: hex(0x2a2733),
            border_strong: hex(0x5a546a),
            border_focused: accent,
            focus_ring: alpha(accent, 0.35),

            text_main: hex(0xece9f3),
            text_muted: hex(0xa59fb5),
            text_disabled: hex(0x6b6578),
            text_on_accent: hex(0x1a1033),
            text_link: hex(0xb9a2fc),
            text_selection: alpha(accent, 0.28),

            accent,
            accent_hover: hex(0xb9a2fc),
            accent_active: hex(0x9370f0),
            accent_subtle: alpha(accent, 0.10),

            success,
            success_subtle: alpha(success, 0.12),
            warning,
            warning_subtle: alpha(warning, 0.12),
            error,
            error_subtle: alpha(error, 0.12),
            info,
            info_subtle: alpha(info, 0.12),

            grad_start: hex(0xb9a2fc),
            grad_end: hex(0x7c4dd6),
            text_on_gradient: hex(0x1a1033),

            shadow: alpha(black, 0.45),
            scrollbar_thumb: alpha(white, 0.10),
            scrollbar_thumb_hover: alpha(white, 0.20),

            cyan: hex(0x67e8f9),
            green: success,
            orange: hex(0xff9e64),
            purple: accent,
            red: error,
            blue: info,
        }
    }

    pub fn light() -> Self {
        let accent = hex(0x5b2bb0);
        let success = hex(0x2e7d4f);
        let warning = hex(0x94650f);
        let error = hex(0xb42318);
        let info = hex(0x2563eb);
        let black = hex(0x000000);

        Self {
            is_dark: false,

            bg_app: hex(0xffffff),
            bg_sunken: hex(0xf6f5f9),
            bg_panel: hex(0xf8f7fb),
            bg_surface: hex(0xffffff),
            bg_overlay: hex(0xffffff),
            bg_hover: hex(0xf2f0f7),
            bg_active: hex(0xe9e6f1),
            bg_selected: alpha(accent, 0.10),
            bg_disabled: alpha(black, 0.03),
            bg_scrim: alpha(black, 0.30),

            border: hex(0xe6e3ee),
            border_subtle: hex(0xefedf4),
            border_strong: hex(0xb4adc4),
            border_focused: accent,
            focus_ring: alpha(accent, 0.25),

            text_main: hex(0x1b1530),
            text_muted: hex(0x5e5873),
            text_disabled: hex(0xa8a3b6),
            text_on_accent: hex(0xffffff),
            text_link: accent,
            text_selection: alpha(accent, 0.16),

            accent,
            accent_hover: hex(0x4c2296),
            accent_active: hex(0x3f1c7e),
            accent_subtle: alpha(accent, 0.07),

            success,
            success_subtle: alpha(success, 0.10),
            warning,
            warning_subtle: alpha(warning, 0.12),
            error,
            error_subtle: alpha(error, 0.08),
            info,
            info_subtle: alpha(info, 0.08),

            grad_start: hex(0x7c4dd6),
            grad_end: accent,
            text_on_gradient: hex(0xffffff),

            shadow: alpha(black, 0.08),
            scrollbar_thumb: alpha(black, 0.14),
            scrollbar_thumb_hover: alpha(black, 0.26),

            cyan: hex(0x0e7490),
            green: success,
            orange: hex(0xc2410c),
            purple: accent,
            red: error,
            blue: info,
        }
    }
}

// Keep `rgba` imported for callers who want to build ad-hoc colors from this module.
#[allow(dead_code)]
pub fn rgba_u32(v: u32) -> Rgba {
    rgba(v)
}
