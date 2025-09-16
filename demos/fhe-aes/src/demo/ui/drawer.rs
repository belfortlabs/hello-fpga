use std::io::{Result, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::{PrintStyledContent, Stylize, style};

use crossterm::{QueueableCommand, execute};

use crate::demo::constants::{
    EQUALITY_COLOR, ERROR_COLOR, HINT_COLOR, INDENTED_STEPS_COLOR, INPUTS_COLOR, INST_COLOR, NEWS_COLOR, OPERAND_RESULT_BOX_HEIGHT, PLUS_COLOR, RESULT_COLOR, SPIN_FRAMES
};
use crate::demo::ui::worker::LogKind;
use crate::demo::{
    app::App,
    constants::{
        EQUALITY_FROM_LEFT, EQUALITY_FROM_TOP, HINT_FROM_LEFT, HINT_FROM_TOP,
        INSTRUCTION_FROM_LEFT, INSTRUCTION_FROM_TOP, OPERAND_AND_RESULT_BOXES_FROM_TOP,
        OPERAND_ONE_FROM_LEFT, OPERAND_RESULT_BOX_WIDTH, OPERAND_TWO_FROM_LEFT, PLUS_FROM_LEFT,
        PLUS_FROM_TOP,
    },
};

pub(crate) trait Drawer {
    fn draw_static_operand(&mut self, label: &str, value: u128, origin: (u16, u16)) -> Result<()>;
    fn draw_operand(&mut self, which: u16, value: u128) -> Result<()>;

    fn draw_hint(&mut self) -> Result<()>;
    fn draw_instruction(&mut self) -> Result<()>;

    fn draw_plus(&mut self) -> Result<()>;
    fn draw_equality(&mut self) -> Result<()>;

    fn draw_mark_at(&mut self, kind: LogKind, line_idx: u16, ok: bool) -> Result<()>;

    fn redraw_static_ui(&mut self) -> Result<()>;
}

impl Drawer for App {
    fn draw_static_operand(&mut self, label: &str, value: u128, origin: (u16, u16)) -> Result<()> {
        let (x, y) = origin;
        let width: u16 = OPERAND_RESULT_BOX_WIDTH;
        let height: u16 = OPERAND_RESULT_BOX_HEIGHT;

        self.stdout
            .queue(MoveTo(x, y))?
            .queue(PrintStyledContent(style("┌").with(INDENTED_STEPS_COLOR)))?
            .queue(PrintStyledContent(
                style("─".repeat((width - 2) as usize)).with(INDENTED_STEPS_COLOR),
            ))?
            .queue(PrintStyledContent(style("┐").with(INDENTED_STEPS_COLOR)))?;

        for row in 1..height - 1 {
            self.stdout
                .queue(MoveTo(x, y + row))?
                .queue(PrintStyledContent(style("│").with(INDENTED_STEPS_COLOR)))?
                .queue(MoveTo(x + width - 1, y + row))?
                .queue(PrintStyledContent(style("│").with(INDENTED_STEPS_COLOR)))?;
        }

        self.stdout
            .queue(MoveTo(x, y + height - 1))?
            .queue(PrintStyledContent(style("└").with(INDENTED_STEPS_COLOR)))?
            .queue(PrintStyledContent(
                style("─".repeat((width - 2) as usize)).with(INDENTED_STEPS_COLOR),
            ))?
            .queue(PrintStyledContent(style("┘").with(INDENTED_STEPS_COLOR)))?;

        self.stdout
            .queue(MoveTo(x + 2, y))?
            .queue(PrintStyledContent(style(label).with(INDENTED_STEPS_COLOR)))?;

        self.stdout
            .queue(MoveTo(x + 2, y + 1))?
            .queue(PrintStyledContent((value.to_string()).with(INPUTS_COLOR)))?
            .flush()
    }

    fn draw_operand(&mut self, which: u16, value: u128) -> Result<()> {
        let from_left = if which == 1 {
            OPERAND_ONE_FROM_LEFT
        } else {
            OPERAND_TWO_FROM_LEFT
        };
        self.draw_static_operand(
            if which == 1 {
                "Operand 1:"
            } else {
                "Operand 2:"
            },
            value,
            (
                self.offsets.0 + from_left,
                self.offsets.1 + OPERAND_AND_RESULT_BOXES_FROM_TOP,
            ),
        )
    }

    fn draw_hint(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            MoveTo(
                HINT_FROM_LEFT + self.offsets.0,
                HINT_FROM_TOP + self.offsets.1
            ),
            PrintStyledContent(style("Press q to quit, r to restart.").with(HINT_COLOR))
        )
    }

    fn draw_instruction(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            MoveTo(
                INSTRUCTION_FROM_LEFT + self.offsets.0,
                INSTRUCTION_FROM_TOP + self.offsets.1
            ),
            PrintStyledContent(style("Press any key to begin entering operands. Press Enter after typing each operand.").with(INST_COLOR)),
            MoveTo(
                INSTRUCTION_FROM_LEFT + self.offsets.0,
                INSTRUCTION_FROM_TOP + self.offsets.1 + 1
            ), 
            PrintStyledContent(style("Overflow may occur if the sum exceeds 2¹²⁸ - 1.").with(INST_COLOR))
        )
    }

    fn draw_plus(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            MoveTo(
                PLUS_FROM_LEFT + self.offsets.0,
                PLUS_FROM_TOP + self.offsets.1
            ),
            PrintStyledContent(style("+").with(PLUS_COLOR))
        )
    }

    fn draw_equality(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            MoveTo(
                EQUALITY_FROM_LEFT + self.offsets.0,
                EQUALITY_FROM_TOP + self.offsets.1
            ),
            PrintStyledContent(style("=").with(EQUALITY_COLOR))
        )
    }

    fn draw_mark_at(&mut self, kind: LogKind, line_idx: u16, ok: bool) -> Result<()> {
        if self.too_small {
            return Ok(());
        }

        let (right_x, base_y) = match kind {
            LogKind::Client => (
                self.client_box.right_edge_x(),
                self.client_box.get_origin().1,
            ),
            LogKind::Server => (
                self.server_box.right_edge_x(),
                self.server_box.get_origin().1,
            ),
        };
        
        let y = base_y + 1 + line_idx;
        let mark = if ok { "✓" } else { "✗" };
        let color = if ok {
            NEWS_COLOR
        } else {
            ERROR_COLOR
        };

        self.stdout
            .queue(MoveTo(right_x, y))?
            .queue(crossterm::style::PrintStyledContent(
                style(mark).with(color),
            ))?
            .flush()?;
        Ok(())
    }

    fn redraw_static_ui(&mut self) -> Result<()> {
        self.build_ui_boxes()?;
        self.draw_hint()?;
        self.draw_instruction()?;

        if let Some(x) = self.x_val_opt {
            self.draw_operand(1, x)?;
        }
        if let Some(y) = self.y_val_opt {
            self.draw_operand(2, y)?;
        }

        let client_logs: Vec<_> = self.client_history.iter().cloned().collect();
        let server_logs: Vec<_> = self.server_history.iter().cloned().collect();
        for (line, color) in client_logs {
            self.client_box.log(&mut self.stdout, &line, color)?;
        }
        for (line, color) in server_logs {
            self.server_box.log(&mut self.stdout, &line, color)?;
        }

        if self.x_val_opt.is_some() {
            self.draw_plus()?;
        }

        let result_opt = self.result_opt.take();
        if let Some(res) = &result_opt {
            self.result_box.draw(&mut self.stdout)?;
            self.draw_equality()?;
            match res {
                Ok(v) => {
                    self
                        .result_box
                        .log(&mut self.stdout, &v.to_string(), RESULT_COLOR)?;
                }
                Err(_) => {
                    self.result_box.log(&mut self.stdout, "Error", ERROR_COLOR)?;
                }
            }
        }
        self.result_opt = result_opt;

        for m in self.finished_marks.clone() {
            self.draw_mark_at(m.get_kind(), m.get_line_idx(), m.is_ok())?;
        }
        
        if !self.too_small {
            for spinner in &self.spinners {
                let (right_x, base_y) = match spinner.get_kind() {
                    LogKind::Client => (self.client_box.right_edge_x(), self.client_box.get_origin().1),
                    LogKind::Server => (self.server_box.right_edge_x(), self.server_box.get_origin().1),
                };
                let y = base_y + 1 + spinner.get_line_idx();
                let ch = SPIN_FRAMES[spinner.get_frame() % SPIN_FRAMES.len()];
                self.stdout
                    .queue(MoveTo(right_x, y))?
                    .queue(crossterm::style::PrintStyledContent(style(ch).with(spinner.get_color())))?;
            }
            self.stdout.flush()?;
        }

        Ok(())
    }
}
