use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Available themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeName {
    Default,
    Nord,
    Gruvbox,
    Dracula,
}

impl ThemeName {
    pub fn all() -> &'static [ThemeName] {
        &[
            ThemeName::Default,
            ThemeName::Nord,
            ThemeName::Gruvbox,
            ThemeName::Dracula,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemeName::Default => "Default",
            ThemeName::Nord => "Nord",
            ThemeName::Gruvbox => "Gruvbox",
            ThemeName::Dracula => "Dracula",
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::Default
    }
}

/// A complete color palette for the TUI.
#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub accent: Color,
    pub success: Color,
    pub warn: Color,
    pub error: Color,
    pub dim: Color,
    pub text: Color,
    pub bg: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
}

impl ThemePalette {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Default => Self::default_theme(),
            ThemeName::Nord => Self::nord(),
            ThemeName::Gruvbox => Self::gruvbox(),
            ThemeName::Dracula => Self::dracula(),
        }
    }

    fn default_theme() -> Self {
        Self {
            accent: Color::Rgb(100, 200, 255),
            success: Color::Rgb(100, 255, 150),
            warn: Color::Rgb(255, 200, 80),
            error: Color::Rgb(255, 100, 100),
            dim: Color::DarkGray,
            text: Color::White,
            bg: Color::Reset,
            highlight_fg: Color::Black,
            highlight_bg: Color::Rgb(100, 200, 255),
        }
    }

    fn nord() -> Self {
        Self {
            accent: Color::Rgb(136, 192, 208),    // Nord8 - frost
            success: Color::Rgb(163, 190, 140),    // Nord14 - aurora green
            warn: Color::Rgb(235, 203, 139),       // Nord13 - aurora yellow
            error: Color::Rgb(191, 97, 106),       // Nord11 - aurora red
            dim: Color::Rgb(76, 86, 106),          // Nord3
            text: Color::Rgb(216, 222, 233),       // Nord4 - snow storm
            bg: Color::Rgb(46, 52, 64),            // Nord0 - polar night
            highlight_fg: Color::Rgb(46, 52, 64),  // Nord0
            highlight_bg: Color::Rgb(136, 192, 208), // Nord8
        }
    }

    fn gruvbox() -> Self {
        Self {
            accent: Color::Rgb(131, 165, 152),     // aqua
            success: Color::Rgb(184, 187, 38),     // green
            warn: Color::Rgb(250, 189, 47),        // yellow
            error: Color::Rgb(251, 73, 52),        // red
            dim: Color::Rgb(146, 131, 116),        // gray
            text: Color::Rgb(235, 219, 178),       // fg
            bg: Color::Rgb(40, 40, 40),            // bg
            highlight_fg: Color::Rgb(40, 40, 40),  // bg
            highlight_bg: Color::Rgb(131, 165, 152), // aqua
        }
    }

    fn dracula() -> Self {
        Self {
            accent: Color::Rgb(139, 233, 253),     // cyan
            success: Color::Rgb(80, 250, 123),     // green
            warn: Color::Rgb(241, 250, 140),       // yellow
            error: Color::Rgb(255, 85, 85),        // red
            dim: Color::Rgb(98, 114, 164),         // comment
            text: Color::Rgb(248, 248, 242),       // fg
            bg: Color::Rgb(40, 42, 54),            // bg
            highlight_fg: Color::Rgb(40, 42, 54),  // bg
            highlight_bg: Color::Rgb(139, 233, 253), // cyan
        }
    }

    // Style helpers
    pub fn title(&self) -> Style {
        Style::new().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn highlight(&self) -> Style {
        Style::new()
            .fg(self.highlight_fg)
            .bg(self.highlight_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::new().fg(self.success).add_modifier(Modifier::BOLD)
    }

    pub fn warn_style(&self) -> Style {
        Style::new().fg(self.warn)
    }

    pub fn error_style(&self) -> Style {
        Style::new().fg(self.error).add_modifier(Modifier::BOLD)
    }

    pub fn dim_style(&self) -> Style {
        Style::new().fg(self.dim)
    }

    pub fn normal_style(&self) -> Style {
        Style::new().fg(self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_load() {
        for name in ThemeName::all() {
            let palette = ThemePalette::from_name(*name);
            // Sanity check — accent should never equal error
            assert_ne!(
                format!("{:?}", palette.accent),
                format!("{:?}", palette.error),
                "Theme {:?} has accent == error",
                name,
            );
        }
    }

    #[test]
    fn test_theme_round_trip() {
        for name in ThemeName::all() {
            let json = serde_json::to_string(name).unwrap();
            let deserialized: ThemeName = serde_json::from_str(&json).unwrap();
            assert_eq!(*name, deserialized);
        }
    }
}
