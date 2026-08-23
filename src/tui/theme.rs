use ratatui::style::{Color, Modifier, Style};
use std::sync::OnceLock;

/// The color palette for a single theme.
#[derive(Debug, Clone)]
pub struct ThemeData {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub secondary: Color,
    pub text: Color,
    pub muted: Color,
    pub success: Color,
}

/// Available application themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    SmokedAmber,
    Bluish,
    Greenish,
    Metallic,
    Dracula,
}

impl ThemeKind {
    pub const ALL: [Self; 5] = [
        Self::SmokedAmber,
        Self::Bluish,
        Self::Greenish,
        Self::Metallic,
        Self::Dracula,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::SmokedAmber => "Smoked Amber",
            Self::Bluish => "Bluish",
            Self::Greenish => "Greenish",
            Self::Metallic => "Metallic",
            Self::Dracula => "Dracula",
        }
    }

    pub fn data(self) -> ThemeData {
        match self {
            Self::SmokedAmber => ThemeData {
                name: self.name(),
                background: Color::Rgb(9, 7, 6),
                surface: Color::Rgb(31, 20, 15),
                surface_alt: Color::Rgb(43, 26, 18),
                accent: Color::Rgb(255, 154, 64),
                accent_soft: Color::Rgb(191, 101, 42),
                secondary: Color::Rgb(255, 102, 91),
                text: Color::Rgb(255, 242, 226),
                muted: Color::Rgb(171, 126, 94),
                success: Color::Rgb(119, 214, 166),
            },
            Self::Bluish => ThemeData {
                name: self.name(),
                background: Color::Rgb(8, 10, 18),
                surface: Color::Rgb(16, 22, 38),
                surface_alt: Color::Rgb(22, 30, 50),
                accent: Color::Rgb(88, 166, 255),
                accent_soft: Color::Rgb(55, 120, 200),
                secondary: Color::Rgb(128, 180, 255),
                text: Color::Rgb(220, 230, 255),
                muted: Color::Rgb(120, 140, 180),
                success: Color::Rgb(100, 220, 170),
            },
            Self::Greenish => ThemeData {
                name: self.name(),
                background: Color::Rgb(6, 14, 6),
                surface: Color::Rgb(16, 32, 16),
                surface_alt: Color::Rgb(22, 44, 22),
                accent: Color::Rgb(100, 220, 100),
                accent_soft: Color::Rgb(60, 160, 60),
                secondary: Color::Rgb(150, 240, 150),
                text: Color::Rgb(220, 255, 220),
                muted: Color::Rgb(100, 160, 100),
                success: Color::Rgb(80, 200, 120),
            },
            Self::Metallic => ThemeData {
                name: self.name(),
                background: Color::Rgb(14, 14, 16),
                surface: Color::Rgb(30, 30, 34),
                surface_alt: Color::Rgb(42, 42, 48),
                accent: Color::Rgb(200, 200, 220),
                accent_soft: Color::Rgb(140, 140, 160),
                secondary: Color::Rgb(170, 180, 210),
                text: Color::Rgb(230, 230, 240),
                muted: Color::Rgb(120, 120, 140),
                success: Color::Rgb(120, 220, 170),
            },
            Self::Dracula => ThemeData {
                name: self.name(),
                background: Color::Rgb(40, 42, 54),
                surface: Color::Rgb(68, 71, 90),
                surface_alt: Color::Rgb(80, 84, 106),
                accent: Color::Rgb(189, 147, 249),
                accent_soft: Color::Rgb(139, 92, 200),
                secondary: Color::Rgb(255, 121, 198),
                text: Color::Rgb(248, 248, 242),
                muted: Color::Rgb(140, 142, 160),
                success: Color::Rgb(80, 250, 123),
            },
        }
    }
}

/// The global theme accessor.
pub struct Theme;

static CURRENT_THEME: OnceLock<std::sync::Mutex<ThemeKind>> = OnceLock::new();

fn theme_lock() -> &'static std::sync::Mutex<ThemeKind> {
    CURRENT_THEME.get_or_init(|| std::sync::Mutex::new(ThemeKind::SmokedAmber))
}

impl Theme {
    /// Returns the current `ThemeData` (the dynamic version).
    pub fn current() -> ThemeData {
        let lock = theme_lock().lock().unwrap_or_else(|e| e.into_inner());
        lock.data()
    }

    /// Returns the current `ThemeKind`.
    pub fn current_kind() -> ThemeKind {
        let lock = theme_lock().lock().unwrap_or_else(|e| e.into_inner());
        *lock
    }

    /// Sets the active theme globally.
    pub fn set(kind: ThemeKind) {
        let mut lock = theme_lock().lock().unwrap_or_else(|e| e.into_inner());
        *lock = kind;
    }

    pub fn block_title() -> Style {
        Style::default()
            .fg(Theme::current().accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Theme::current().accent_soft)
    }

    pub fn active_tab() -> Style {
        let t = Theme::current();
        Style::default()
            .fg(t.background)
            .bg(t.accent)
            .add_modifier(Modifier::BOLD)
    }
}
