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

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
    Process,
    FEditing,
    FProcess,
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
    pub messages: Vec<(String, String, String, String)>,
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
    pub fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let current_index = self.character_index;
        let is_cursor_leftmost = current_index == 0;
        if is_cursor_leftmost {
            return;
        }

        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = self.input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
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

        let mut max_diff = 0;
        let mut matched_name = "None".to_string();

        let time_string = format!("{:.5}", sec);

        let comment = if fpga_enable & enc_struct.input.starts_with("p:") {
            "Plaintext query and FPGA Acceleration".to_owned()
        } else if fpga_enable {
            "FPGA Acceleration".to_owned()
        } else if enc_struct.input.starts_with("p:") {
            "Plaintext query".to_owned()
        } else {
            "Normal execution".to_owned()
        };

        for i in 0..enc_struct.db_size {
            let enc_score = result.get(&i).unwrap();

            let diff = i64::abs_diff(
                NAME_LIST[i].len().try_into().unwrap(),
                enc_struct.query.len().try_into().unwrap(),
            ) as i64;

            if i64::abs_diff(*enc_score, NAME_LIST[i].len().try_into().unwrap()) as i64 - diff
                > max_diff
            {
                matched_name = NAME_LIST[i].to_string();
                max_diff =
                    i64::abs_diff(*enc_score, NAME_LIST[i].len().try_into().unwrap()) as i64 - diff;
            }
        }

        if max_diff <= 5 {
            self.messages.push((
                enc_struct.query.clone(),
                "No".to_owned(),
                time_string,
                "Normal execution".to_owned(),
            ));
        } else {
            self.messages
                .push((enc_struct.query.clone(), matched_name, time_string, comment));
        }

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

        let mut padded_lines = Vec::new();
        for _ in 0..top_padding {
            padded_lines.push(Line::from(""));
        }
        padded_lines.extend(text.lines.clone());

        let paragraph = Paragraph::new(padded_lines)
            .block(block)
            .centered()
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, middle_block_area);
    }
}
