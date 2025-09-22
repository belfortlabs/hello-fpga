use std::time::Duration;

use crossterm::style::Color;

pub(crate) const TERMINAL_POLL_MS: u64 = 50;

pub(crate) const OPERAND_RESULT_BOX_WIDTH: u16 = 33;
pub(crate) const OPERAND_RESULT_BOX_HEIGHT: u16 = 3;

pub(crate) const CLIENT_SERVER_BOX_WIDTH: u16 = 58;
pub(crate) const CLIENT_SERVER_BOX_HEIGHT: u16 = 21;

pub(crate) const CLIENT_SERVER_BOX_GAP: u16 = 1;
pub(crate) const OPERAND_RESULT_BOX_GAP: u16 = 9;
pub(crate) const OPERAND_LOG_BOX_GAP: u16 = 1;

pub(crate) const FIGURE_MARGIN_X: u16 = 2;
pub(crate) const FIGURE_MARGIN_Y: u16 = 2;

pub(crate) const HINT_INDENT: u16 = 1;
pub(crate) const INSTRUCTION_INDENT: u16 = 1;

pub(crate) const HINT_LINES: u16 = 1;
pub(crate) const INSTRUCTION_LINES: u16 = 2;

pub(crate) const FIGURE_WIDTH: u16 =
    OPERAND_RESULT_BOX_WIDTH * 3 + OPERAND_RESULT_BOX_GAP * 2 + FIGURE_MARGIN_X * 2;

pub(crate) const FIGURE_HEIGHT: u16 = OPERAND_RESULT_BOX_HEIGHT
    + CLIENT_SERVER_BOX_HEIGHT
    + OPERAND_LOG_BOX_GAP
    + HINT_LINES
    + INSTRUCTION_LINES
    + FIGURE_MARGIN_Y * 2;

pub(crate) const OPERAND_ONE_FROM_LEFT: u16 = FIGURE_MARGIN_X;

pub(crate) const OPERAND_TWO_FROM_LEFT: u16 =
    FIGURE_MARGIN_X + OPERAND_RESULT_BOX_WIDTH + OPERAND_RESULT_BOX_GAP;

pub(crate) const RESULT_BOX_FROM_LEFT: u16 =
    FIGURE_MARGIN_X + OPERAND_RESULT_BOX_WIDTH * 2 + OPERAND_RESULT_BOX_GAP * 2;

pub(crate) const PLUS_FROM_LEFT: u16 =
    FIGURE_MARGIN_X + OPERAND_RESULT_BOX_WIDTH + OPERAND_RESULT_BOX_GAP / 2;

pub(crate) const EQUALITY_FROM_LEFT: u16 = FIGURE_MARGIN_X
    + OPERAND_RESULT_BOX_WIDTH * 2
    + OPERAND_RESULT_BOX_GAP
    + OPERAND_RESULT_BOX_GAP / 2;

pub(crate) const CLIENT_BOX_FROM_LEFT: u16 = FIGURE_MARGIN_X;

pub(crate) const SERVER_BOX_FROM_LEFT: u16 =
    FIGURE_MARGIN_X + CLIENT_SERVER_BOX_WIDTH + CLIENT_SERVER_BOX_GAP;

pub(crate) const HINT_FROM_LEFT: u16 = FIGURE_MARGIN_X + HINT_INDENT;
pub(crate) const INSTRUCTION_FROM_LEFT: u16 = FIGURE_MARGIN_X + INSTRUCTION_INDENT;

pub(crate) const OPERAND_AND_RESULT_BOXES_FROM_TOP: u16 = FIGURE_MARGIN_Y;

pub(crate) const PLUS_FROM_TOP: u16 =
    OPERAND_AND_RESULT_BOXES_FROM_TOP + OPERAND_RESULT_BOX_HEIGHT / 2;
pub(crate) const EQUALITY_FROM_TOP: u16 = PLUS_FROM_TOP;

pub(crate) const CLIENT_BOX_FROM_TOP: u16 =
    OPERAND_AND_RESULT_BOXES_FROM_TOP + OPERAND_RESULT_BOX_HEIGHT + OPERAND_LOG_BOX_GAP;

pub(crate) const SERVER_BOX_FROM_TOP: u16 = CLIENT_BOX_FROM_TOP;

pub(crate) const HINT_FROM_TOP: u16 = CLIENT_BOX_FROM_TOP + CLIENT_SERVER_BOX_HEIGHT;

pub(crate) const INSTRUCTION_FROM_TOP: u16 = HINT_FROM_TOP + 1;

pub(crate) const CLIENT_SERVER_BOX_PAD: u16 = 3;

pub(crate) const STEPS_COLOR: Color = Color::Rgb {
    r: 255,
    g: 64,
    b: 255,
};

pub(crate) const INDENTED_STEPS_COLOR: Color = Color::Rgb {
    r: 153,
    g: 153,
    b: 153,
};

pub(crate) const NEWS_COLOR: Color = Color::Rgb {
    r: 102,
    g: 255,
    b: 153,
};

pub(crate) const RESULT_COLOR: Color = NEWS_COLOR;

pub(crate) const ERROR_COLOR: Color = Color::Rgb {
    r: 240,
    g: 43,
    b: 43,
};

pub(crate) const HINT_COLOR: Color = Color::Rgb {
    r: 153,
    g: 153,
    b: 153,
};

pub(crate) const INST_COLOR: Color = Color::Rgb {
    r: 153,
    g: 153,
    b: 153,
};

pub(crate) const PLUS_COLOR: Color = Color::Rgb {
    r: 153,
    g: 153,
    b: 153,
};

pub(crate) const EQUALITY_COLOR: Color = Color::Rgb {
    r: 153,
    g: 153,
    b: 153,
};

pub(crate) const INPUTS_COLOR: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 102,
};

pub(crate) const TIME_COLOR: Color = Color::Rgb {
    r: 255,
    g: 165,
    b: 0,
};

pub(crate) const SPIN_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub(crate) const SPIN_INTERVAL: Duration = Duration::from_millis(80);
