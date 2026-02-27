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
    pub progress_done: Vec<u8>,
}

impl App {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
            progress_done: Vec::new(),
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

    pub fn post_process(&mut self, enc_struct: &mut EncStruct, fpga_enable: bool) {
        // Computation of the rest of the matrix
        let mut h_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(enc_struct.db_size);
        let mut v_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(enc_struct.db_size);

        for k in 0..enc_struct.db_size {
            let mut h_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(enc_struct.max_factor);
            let mut v_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(enc_struct.max_factor);

            for i in 0..enc_struct.max_factor {
                let mut h_vec: Vec<i64> = Vec::with_capacity(enc_struct.max_factor);
                let mut v_vec: Vec<i64> = Vec::with_capacity(enc_struct.max_factor);

                for j in 0..enc_struct.max_factor {
                    let h_dec: u64 = enc_struct.cks.decrypt(&enc_struct.h_matrices[k][i][j]);
                    let v_dec: u64 = enc_struct.cks.decrypt(&enc_struct.v_matrices[k][i][j]);
                    h_vec.push(decode_matrix_value(h_dec));
                    v_vec.push(decode_matrix_value(v_dec));
                }

                h_dec_matrix.push(h_vec);
                v_dec_matrix.push(v_vec);
            }
            h_dec_matrices.push(h_dec_matrix);
            v_dec_matrices.push(v_dec_matrix);
        }

        let mut result_map: HashMap<usize, i64> = HashMap::new();

        let m = enc_struct.max_factor - 1;
        for k in 0..enc_struct.db_size {
            result_map.insert(
                k,
                compute_diagonal_score(&h_dec_matrices[k], &v_dec_matrices[k], m),
            );
        }

        let sec = enc_struct.time.elapsed().as_secs_f64();

        let mut max_diff = 0;
        let mut matched_name = "None".to_string();

        let time_string = format!("{:.5}", sec);

        let comment = if fpga_enable & enc_struct.input.starts_with("p:") {
            "plaintext query and FPGA Acceleration".to_owned()
        } else if fpga_enable {
            "FPGA Acceleration".to_owned()
        } else if enc_struct.input.starts_with("p:") {
            "plaintext query".to_owned()
        } else {
            "Normal execution".to_owned()
        };

        for i in 0..enc_struct.db_size {
            let enc_score = result_map.get(&i).unwrap();

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

        self.progress_done.clear();
        self.input.clear();
        self.reset_cursor();
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

/// Maps a raw decrypted u64 to a signed i64, correcting for the 16-value bias used in the
/// FHE encoding (values > 8 represent negative numbers stored as their 16-complement).
fn decode_matrix_value(dec: u64) -> i64 {
    if dec > 8 {
        dec as i64 - 16
    } else {
        dec as i64
    }
}

/// Sums the diagonal entries of the decrypted H and V matrices to produce the final
/// Levenshtein similarity score for one database entry.
fn compute_diagonal_score(h: &[Vec<i64>], v: &[Vec<i64>], m: usize) -> i64 {
    let h_sum: i64 = (1..=m).map(|i| h[i][i]).sum();
    let v_sum: i64 = (0..m).map(|i| v[i + 1][i]).sum();
    h_sum + v_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_low_values_unchanged() {
        assert_eq!(decode_matrix_value(0), 0);
        assert_eq!(decode_matrix_value(1), 1);
        assert_eq!(decode_matrix_value(8), 8);
    }

    #[test]
    fn decode_high_values_biased() {
        assert_eq!(decode_matrix_value(9), -7);
        assert_eq!(decode_matrix_value(15), -1);
        assert_eq!(decode_matrix_value(16), 0);
    }

    #[test]
    fn diagonal_score_all_zero_matrices() {
        let m = 3;
        let h = vec![vec![0i64; m + 1]; m + 1];
        let v = vec![vec![0i64; m + 1]; m + 1];
        assert_eq!(compute_diagonal_score(&h, &v, m), 0);
    }

    #[test]
    fn diagonal_score_known_values() {
        let m = 2;
        let mut h = vec![vec![0i64; m + 1]; m + 1];
        let mut v = vec![vec![0i64; m + 1]; m + 1];
        // h diagonal: h[1][1]=1, h[2][2]=2  → sum 3
        h[1][1] = 1;
        h[2][2] = 2;
        // v sub-diagonal: v[1][0]=3, v[2][1]=4  → sum 7
        v[1][0] = 3;
        v[2][1] = 4;
        assert_eq!(compute_diagonal_score(&h, &v, m), 10);
    }

    #[test]
    fn diagonal_score_only_h_contribution() {
        let m = 2;
        let mut h = vec![vec![0i64; m + 1]; m + 1];
        let v = vec![vec![0i64; m + 1]; m + 1];
        h[1][1] = 5;
        h[2][2] = 3;
        assert_eq!(compute_diagonal_score(&h, &v, m), 8);
    }

    #[test]
    fn diagonal_score_only_v_contribution() {
        let m = 2;
        let h = vec![vec![0i64; m + 1]; m + 1];
        let mut v = vec![vec![0i64; m + 1]; m + 1];
        v[1][0] = 6;
        v[2][1] = 2;
        assert_eq!(compute_diagonal_score(&h, &v, m), 8);
    }
}
