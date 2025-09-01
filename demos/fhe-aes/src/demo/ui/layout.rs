use std::io::{Result, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};

use crate::demo::app::App;
use crate::demo::constants::{FIGURE_HEIGHT, FIGURE_WIDTH};

pub(crate) trait LayoutManager {
    fn offsets_for(&mut self, size: (u16, u16)) -> (u16, u16);
    fn terminal_too_small(&mut self, size: (u16, u16)) -> bool;

    fn warn_resize(&mut self, size: (u16, u16)) -> Result<()>;
}

impl LayoutManager for App {
    fn offsets_for(&mut self, (cols, rows): (u16, u16)) -> (u16, u16) {
        (
            cols.saturating_sub(FIGURE_WIDTH) / 2,
            rows.saturating_sub(FIGURE_HEIGHT) / 2,
        )
    }

    fn terminal_too_small(&mut self, (cols, rows): (u16, u16)) -> bool {
        cols < FIGURE_WIDTH || rows < FIGURE_HEIGHT
    }

    fn warn_resize(&mut self, (cols, rows): (u16, u16)) -> Result<()> {
        queue!(
            self.stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(format!(
                "Terminal size: ({cols}×{rows}) — the demo requires at least \
                 {FIGURE_WIDTH}×{FIGURE_HEIGHT}"
            ))
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}
