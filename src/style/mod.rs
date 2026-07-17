//! Style definitions: charsets, border styles, line styles, alignment, and color.

use serde::{Deserialize, Serialize};

/// The character set to use for rendering.
///
/// The user must always explicitly choose one — there is no default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Charset {
    /// 7-bit ASCII only (`-`, `|`, `+`, `^`, `v`, `<`, `>`, etc.)
    Ascii,
    /// Unicode box-drawing characters and arrows
    Unicode,
}

/// Border style definitions.
///
/// Each style maps to character pairs for ASCII and Unicode rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    /// Single-line border
    Single,
    /// Double-line border
    Double,
    /// Rounded corners
    Rounded,
    /// Dashed lines
    Dashed,
    /// Bold lines
    Bold,
}

/// The glyphs used for drawing a specific border style.
pub struct BorderGlyphs {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub tee_right: char,
    pub tee_left: char,
    pub tee_down: char,
    pub tee_up: char,
    pub cross: char,
}

impl BorderStyle {
    /// Return the glyphs for this border style in the given charset.
    pub fn glyphs(self, charset: Charset) -> BorderGlyphs {
        match (self, charset) {
            (BorderStyle::Single, Charset::Unicode) => BorderGlyphs {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
                tee_right: '├',
                tee_left: '┤',
                tee_down: '┬',
                tee_up: '┴',
                cross: '┼',
            },
            (BorderStyle::Double, Charset::Unicode) => BorderGlyphs {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
                tee_right: '╠',
                tee_left: '╣',
                tee_down: '╦',
                tee_up: '╩',
                cross: '╬',
            },
            (BorderStyle::Rounded, Charset::Unicode) => BorderGlyphs {
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
                horizontal: '─',
                vertical: '│',
                tee_right: '├',
                tee_left: '┤',
                tee_down: '┬',
                tee_up: '┴',
                cross: '┼',
            },
            (BorderStyle::Dashed, Charset::Unicode) => BorderGlyphs {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '╌',
                vertical: '╎',
                tee_right: '├',
                tee_left: '┤',
                tee_down: '┬',
                tee_up: '┴',
                cross: '┼',
            },
            (BorderStyle::Bold, Charset::Unicode) => BorderGlyphs {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
                tee_right: '┣',
                tee_left: '┫',
                tee_down: '┳',
                tee_up: '┻',
                cross: '╋',
            },
            (BorderStyle::Single, Charset::Ascii) => BorderGlyphs {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
                tee_right: '+',
                tee_left: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            },
            (BorderStyle::Double, Charset::Ascii) => BorderGlyphs {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '=',
                vertical: '|',
                tee_right: '+',
                tee_left: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            },
            (BorderStyle::Rounded, Charset::Ascii) => BorderGlyphs {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
                tee_right: '+',
                tee_left: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            },
            (BorderStyle::Dashed, Charset::Ascii) => BorderGlyphs {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
                tee_right: '+',
                tee_left: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            },
            (BorderStyle::Bold, Charset::Ascii) => BorderGlyphs {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '#',
                vertical: '#',
                tee_right: '+',
                tee_left: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            },
        }
    }
}

/// Line and arrow styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    /// Simple lines (`-->`, `<--`, `→`, `←`)
    Simple,
    /// Bold lines (`==>`, `⇒`, `⇐`)
    Bold,
    /// Box-drawing connector lines
    BoxDrawing,
}

/// Horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

/// Combined alignment (horizontal + vertical).
pub struct Alignment {
    pub horizontal: HAlign,
    pub vertical: VAlign,
}

impl Alignment {
    pub const TOP_LEFT: Self = Self { horizontal: HAlign::Left, vertical: VAlign::Top };
    pub const CENTER: Self = Self { horizontal: HAlign::Center, vertical: VAlign::Middle };
}

/// ANSI terminal color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    /// Return the ANSI foreground escape code for this color.
    pub fn fg_code(self) -> &'static str {
        match self {
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::BrightBlack => "\x1b[90m",
            Color::BrightRed => "\x1b[91m",
            Color::BrightGreen => "\x1b[92m",
            Color::BrightYellow => "\x1b[93m",
            Color::BrightBlue => "\x1b[94m",
            Color::BrightMagenta => "\x1b[95m",
            Color::BrightCyan => "\x1b[96m",
            Color::BrightWhite => "\x1b[97m",
        }
    }
}

/// Padding configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padding {
    pub horizontal: usize,
    pub vertical: usize,
}

impl Default for Padding {
    fn default() -> Self {
        Self { horizontal: 1, vertical: 0 }
    }
}
