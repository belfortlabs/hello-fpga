use std::io::{Result, Stdout, Write};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, Print, PrintStyledContent, Stylize, style},
};

use crate::demo::constants::{CLIENT_SERVER_BOX_PAD, INDENTED_STEPS_COLOR, INPUTS_COLOR};

pub(crate) struct TextBox {
    origin: (u16, u16),
    width: u16,
    height: u16,
    label: &'static str,
    current_line: u16,
    input_buffer: Option<String>,
}

impl TextBox {
    pub(crate) fn empty() -> Self {
        Self {
            origin: (0, 0),
            width: 0,
            height: 0,
            label: "",
            current_line: 0,
            input_buffer: None,
        }
    }

    pub(crate) fn new(origin: (u16, u16), width: u16, height: u16, label: &'static str) -> Self {
        Self {
            origin,
            width,
            height,
            label,
            current_line: 0,
            input_buffer: Some(String::from("")),
        }
    }

    pub(crate) fn get_origin(&mut self) -> (u16, u16) {
        self.origin
    }

    pub(crate) fn set_origin(&mut self, origin: (u16, u16)) {
        self.origin = origin;
    }

    pub(crate) fn get_mut_buf(&mut self) -> &mut String {
        self.input_buffer.as_mut().unwrap()
    }

    pub(crate) fn get_width(&self) -> u16 {
        self.width
    }

    pub(crate) fn right_edge_x(&self) -> u16 {
        self.origin.0 + self.width - CLIENT_SERVER_BOX_PAD
    }

    pub(crate) fn current_line_idx(&self) -> u16 {
        self.current_line.saturating_sub(1)
    }

    pub(crate) fn draw(&self, stdout: &mut Stdout) -> Result<()> {
        let (x, y) = self.origin;

        stdout
            .queue(MoveTo(x, y))?
            .queue(PrintStyledContent(style("┌").with(INDENTED_STEPS_COLOR)))?
            .queue(PrintStyledContent(
                style("─".repeat((self.width - 2) as usize)).with(INDENTED_STEPS_COLOR),
            ))?
            .queue(PrintStyledContent(style("┐").with(INDENTED_STEPS_COLOR)))?;

        for row in 1..self.height - 1 {
            stdout
                .queue(MoveTo(x, y + row))?
                .queue(PrintStyledContent(style("│").with(INDENTED_STEPS_COLOR)))?
                .queue(MoveTo(x + self.width - 1, y + row))?
                .queue(PrintStyledContent(style("│").with(INDENTED_STEPS_COLOR)))?;
        }

        stdout
            .queue(MoveTo(x, y + self.height - 1))?
            .queue(PrintStyledContent(style("└").with(INDENTED_STEPS_COLOR)))?
            .queue(PrintStyledContent(
                style("─".repeat((self.width - 2) as usize)).with(INDENTED_STEPS_COLOR),
            ))?
            .queue(PrintStyledContent(style("┘").with(INDENTED_STEPS_COLOR)))?;

        stdout
            .queue(MoveTo(x + 2, y))?
            .queue(PrintStyledContent(
                style(self.label).with(INDENTED_STEPS_COLOR),
            ))?
            .flush()?;

        Ok(())
    }

    pub(crate) fn clear_inside(&mut self, stdout: &mut Stdout) -> Result<()> {
        let (x, y) = self.origin;
        for row in 1..self.height - 1 {
            stdout
                .queue(MoveTo(x + 1, y + row))?
                .queue(Print(" ".repeat((self.width - 2) as usize)))?;
        }
        self.current_line = 0;
        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn log(&mut self, stdout: &mut Stdout, msg: &str, color: Color) -> Result<()> {
        let (x, y) = self.origin;
        let max_len = (self.width - 3) as usize;

        for raw in msg.split('\n') {
            let mut line = raw.trim_end();

            while !line.is_empty() {
                if self.current_line >= self.height - 2 {
                    self.clear_inside(stdout)?;
                }

                let printable = if line.len() > max_len {
                    &line[..max_len]
                } else {
                    line
                };

                let padded = format!("{:<width$}", printable, width = max_len);
                stdout
                    .queue(MoveTo(x + 2, y + 1 + self.current_line))?
                    .queue(PrintStyledContent(style(padded).with(color)))?;
                stdout.flush()?;

                self.current_line += 1;
                line = &line[printable.len()..];
            }
        }

        Ok(())
    }

    pub(crate) fn render(&self, out: &mut Stdout) -> Result<()> {
        let (x, y) = self.origin;
        let max = (self.width - 3) as usize;
        let buf = self.input_buffer.as_ref().unwrap();
        let printable = format!("{:<width$}", &buf[..buf.len().min(max)], width = max);

        out.queue(MoveTo(x + 2, y + 1))?
            .queue(PrintStyledContent(style(printable).with(INPUTS_COLOR)))?
            .flush()
    }
}
