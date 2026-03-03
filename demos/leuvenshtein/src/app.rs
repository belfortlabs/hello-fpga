/*
* MIT License
*
* Copyright (c) 2025 KU Leuven - COSIC
* Author: Wouter Legiest
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software.
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/

use crate::data::*;
use crate::enc_struct::EncStruct;

use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph, Wrap},
    Frame,
};

pub const PLAINTEXT_PREFIX: &str = "p:";

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
    Process,
    FEditing,
    FProcess,
}

#[derive(Clone, Copy)]
pub enum ExecutionMode {
    Normal,
    Plaintext,
    Fpga,
    PlaintextAndFpga,
}

#[derive(Clone)]
pub struct QueryResult {
    pub query: String,
    pub matched_name: Option<String>,
    pub elapsed_secs: String,
    pub mode: ExecutionMode,
}

/// App holds the state of the application
#[derive(Clone)]
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<QueryResult>,
    pub progress_done: usize,
}

impl App {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
            progress_done: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        if self.character_index == 0 {
            return;
        }
        let byte_pos = self.byte_index();
        let prev_start = self.input[..byte_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.remove(prev_start);
        self.move_cursor_left();
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn post_process(
        &mut self,
        result: HashMap<usize, i64>,
        enc_struct: &mut EncStruct,
        fpga_enable: bool,
    ) {
        let sec = enc_struct.time.elapsed().as_secs_f64();
        let time_string = format!("{:.5}", sec);

        let mode = if fpga_enable && enc_struct.input.starts_with(PLAINTEXT_PREFIX) {
            ExecutionMode::PlaintextAndFpga
        } else if fpga_enable {
            ExecutionMode::Fpga
        } else if enc_struct.input.starts_with(PLAINTEXT_PREFIX) {
            ExecutionMode::Plaintext
        } else {
            ExecutionMode::Normal
        };

        let query_len: i64 = enc_struct.query.len().try_into().unwrap();
        let mut max_diff = 0i64;
        let mut best_match: Option<String> = None;

        for i in 0..enc_struct.db_size {
            let enc_score = *result.get(&i).unwrap();
            let name_len: i64 = NAME_LIST[i].len().try_into().unwrap();
            let diff = i64::abs_diff(name_len, query_len) as i64;
            let score_diff = i64::abs_diff(enc_score, name_len) as i64 - diff;
            if score_diff > max_diff {
                best_match = Some(NAME_LIST[i].to_string());
                max_diff = score_diff;
            }
        }

        self.messages.push(QueryResult {
            query: enc_struct.query.clone(),
            matched_name: if max_diff <= 5 { None } else { best_match },
            elapsed_secs: time_string,
            mode,
        });

        self.progress_done = 0;
        self.input.clear();
        self.reset_cursor();
        self.input_mode = InputMode::Normal;
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let central_layout = Layout::default()
            .direction(Direction::Vertical) // Divide vertically first
            .constraints([
                Constraint::Percentage(20), // Top 33%
                Constraint::Percentage(60), // Middle 34% (slightly larger to ensure no gaps with rounding)
                Constraint::Percentage(20), // Bottom 33%
            ])
            .split(area); // Split the 'content' area

        let middle_row = central_layout[1]; // Get the middle horizontal band

        let middle_column_layout = Layout::default()
            .direction(Direction::Horizontal) // Then divide horizontally within the middle row
            .constraints([
                Constraint::Percentage(25), // Left 25%
                Constraint::Percentage(50), // Middle 50%
                Constraint::Percentage(25), // Right 25%
            ])
            .split(middle_row);

        let middle_block_area = middle_column_layout[1];

        let block = Block::bordered().border_type(BorderType::Rounded);

        let text = Text::from(vec![
            Line::from("Created by Wouter Legiest, COSIC - KU Leuven"),
            Line::from(""),
            Line::from("Accelerated on FPGA by Belfort"),
            Line::from(""),
            Line::from(Span::styled(
                "Leuvenshtein Database Demo",
                Style::default().fg(Color::Yellow).bold(),
            )),
            Line::from(""), // Empty line for spacing
            Line::from(Span::styled(
                "Preprocessing",
                Style::default().italic().bold().slow_blink(),
            )),
            Line::from(vec![Span::styled(
                "Encrypting and processing the database",
                Style::default(),
            )]),
            Line::from(""),
            #[cfg(not(feature = "fpga"))]
            Line::from(vec![Span::styled(
                "NO FPGA SUPPORT, ADD `fpga` FEATURE",
                Style::default().fg(Color::Red).bold(),
            )]),
            Line::from(""),
            Line::from("This demo lets you search for movie characters, e.g. Biff Tannen, Hans Gruber, Indiana Jones, etc. "),
                Line::from("You can search your favourite character, even if your input contains typos; “Bilba Biggins” will match “Bilbo Baggins”"),
            Line::from(""),
        ]);

        let text_lines = text.lines.len() as u16;
        let available_height = middle_block_area.height;
        let top_padding = (available_height.saturating_sub(text_lines) as f32 * 0.33) as u16;

        let padded_lines: Vec<Line<'_>> = std::iter::repeat(Line::default())
            .take(top_padding as usize)
            .chain(text.lines)
            .collect();

        let paragraph = Paragraph::new(padded_lines)
            .block(block)
            .centered()
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, middle_block_area);
    }
}
