use crossterm::style::Color;

use super::worker::LogKind;

pub(crate) struct Spinner {
    id: u32,
    kind: LogKind,
    line_idx: u16,
    frame: usize,
    color: Color,
}

#[derive(Clone)]
pub(crate) struct CompletedMark {
    kind: LogKind,
    line_idx: u16,
    ok: bool,
}

impl Spinner {
    pub(crate) fn new(id: u32, kind: LogKind, line_idx: u16, color: Color) -> Self {
        Self {
            id,
            kind,
            line_idx,
            frame: 0,
            color,
        }
    }

    pub(crate) fn get_id(&self) -> u32 {
        self.id
    }

    pub(crate) fn get_kind(&self) -> LogKind {
        self.kind
    }

    pub(crate) fn get_line_idx(&self) -> u16 {
        self.line_idx
    }

    pub(crate) fn get_frame(&self) -> usize {
        self.frame
    }

    pub(crate) fn get_color(&self) -> Color {
        self.color
    }

    pub(crate) fn set_frame(&mut self, frame: usize) {
        self.frame = frame;
    }
}

impl CompletedMark {
    pub(crate) fn new(kind: LogKind, line_idx: u16, ok: bool) -> Self {
        Self { kind, line_idx, ok }
    }

    pub(crate) fn get_kind(&self) -> LogKind {
        self.kind
    }

    pub(crate) fn get_line_idx(&self) -> u16 {
        self.line_idx
    }

    pub(crate) fn is_ok(&self) -> bool {
        self.ok
    }
}
