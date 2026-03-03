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

use crate::enc_struct::EncStruct;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Gauge, List, ListItem, Paragraph};

use std::{error::Error, io};
use tfhe::shortint::prelude::*;

mod algorithm;
mod app;
mod data;
mod enc_struct;
mod util;
use crate::algorithm::myers::*;
use crate::app::App;
use crate::app::InputMode;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let area = terminal.size()?;

    let min_width = 100;
    let min_height = 35;

    if area.width < min_width || area.height < min_height {
        let msg = format!(
            "Terminal too small ({}x{}). Minimum size: {}x{}",
            area.width, area.height, min_width, min_height
        );
        return Err(io::Error::new(io::ErrorKind::Unsupported, msg));
    }

    terminal.draw(|frame| app.draw(frame))?;

    // security = 132 bits, p-fail = 2^-71.625
    let mut params = tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS.clone();
    params.message_modulus = MessageModulus(16);
    params.carry_modulus = CarryModulus(1);

    let cks: ClientKey = ClientKey::new(params);

    let db_max_size = data::NAME_LIST.iter().map(|s| s.len()).max().unwrap_or(0) + 1;

    let db_processed = data::process_db(&cks, db_max_size);

    let mut enc_struct = EncStruct::new(db_max_size, db_processed, cks);

    loop {
        terminal.draw(|f| ui(f, &app, &enc_struct))?;

        let fpga = matches!(app.input_mode, InputMode::FProcess);
        if fpga || matches!(app.input_mode, InputMode::Process) {
            if enc_struct.input.starts_with("p:") {
                if app.progress_done == 0 {
                    process_plain_query_enc_db(&mut enc_struct);
                    app.progress_done += 1;
                    process_plain_part_i(1, &mut enc_struct, fpga);
                    app.progress_done += 1;
                } else if app.progress_done >= enc_struct.max_factor {
                    let result = decrypt_and_compute_results(&mut enc_struct);
                    app.post_process(result, &mut enc_struct, fpga);
                } else {
                    process_plain_part_i(app.progress_done, &mut enc_struct, fpga);
                    app.progress_done += 1;
                }
            } else {
                if app.progress_done == 0 {
                    process_enc_query_enc_db(&mut enc_struct);
                    app.progress_done += 1;
                    process_part_i(1, &mut enc_struct, fpga);
                    app.progress_done += 1;
                } else if app.progress_done >= enc_struct.max_factor {
                    let result = decrypt_and_compute_results(&mut enc_struct);
                    app.post_process(result, &mut enc_struct, fpga);
                } else {
                    process_part_i(app.progress_done, &mut enc_struct, fpga);
                    app.progress_done += 1;
                }
            }
        } else if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        #[cfg(feature = "fpga")]
                        enc_struct.fpga_key.disconnect();
                        return Ok(());
                    }
                    KeyCode::Char('f') => {
                        app.input_mode = InputMode::FEditing;
                    }
                    _ => {}
                },
                InputMode::Editing | InputMode::FEditing if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Enter => {
                            enc_struct.input = app.input.clone();
                            enc_struct.query = if enc_struct.input.starts_with("p:") {
                                enc_struct.input.chars().skip(2).collect()
                            } else {
                                enc_struct.input.clone()
                            };
                            app.input_mode = match app.input_mode {
                                InputMode::Editing => InputMode::Process,
                                _ => InputMode::FProcess,
                            };
                        }
                        KeyCode::Char(to_insert) => {
                            app.enter_char(to_insert);
                        }
                        KeyCode::Backspace => {
                            app.delete_char();
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App, enc_struct: &EncStruct) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(1),
    ]);
    let [help_area, input_area, progress_area, messages_area] = vertical.areas(f.area());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit; ".into(),
                "e".bold(),
                " to start CPU-based query; ".into(),
                "f".bold(),
                " to start ".into(),
                "FPGA-accelerated".bold().italic().yellow(),
                " query.".into(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop entering, ".into(),
                "Enter".bold(),
                " to process the message".into(),
            ],
            Style::default(),
        ),
        InputMode::Process => (
            vec!["Processing...".into()],
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ),
        InputMode::FEditing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop entering, ".into(),
                "Enter".bold(),
                " to process the message with FPGA".into(),
            ],
            Style::default(),
        ),
        InputMode::FProcess => (
            vec!["Processing with FPGA...".into()],
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ),
    };
    f.render_widget(
        Paragraph::new(Text::from(Line::from(msg)).patch_style(style)),
        help_area,
    );

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing | InputMode::FEditing => Style::default().fg(Color::Yellow),
            InputMode::Process | InputMode::FProcess => Style::default().fg(Color::Blue),
        })
        .block(Block::bordered().title(" Input "));
    f.render_widget(input, input_area);

    #[allow(clippy::cast_possible_truncation)]
    if matches!(app.input_mode, InputMode::Editing | InputMode::FEditing) {
        f.set_cursor_position(Position::new(
            input_area.x + app.character_index as u16 + 1,
            input_area.y + 1,
        ));
    }

    let total_round: usize = enc_struct.max_factor;
    let done = app.progress_done;
    #[allow(clippy::cast_precision_loss)]
    let progress = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green))
        .label(format!("{done}/{total_round}"))
        .ratio(done as f64 / total_round as f64);
    f.render_widget(progress, progress_area);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let span1 = <String as Clone>::clone(&m.0).red().bold();

            if m.1 == "No" {
                return ListItem::new(Line::from(vec![
                    Span::raw(format!("{}) No match found for ", i)),
                    span1,
                ]));
            }

            let string_build = format!("{}) Query: \"", i);
            let string2_build = format!("\" in {} s ", m.2);
            let span2 = <String as Clone>::clone(&m.1).green().bold();

            let span3: Vec<Span<'_>> = match m.3.as_str() {
                "Normal execution" => vec![<String as Clone>::clone(&m.3).blue().bold()],
                "plaintext query" => vec![<String as Clone>::clone(&m.3).magenta().bold()],
                "FPGA Acceleration" => vec![<String as Clone>::clone(&m.3).yellow().bold()],
                _ => vec![
                    Span::styled(
                        "plaintext query",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::from(" and "),
                    Span::styled(
                        "FPGA Acceleration",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ],
            };

            let mut line_parts = vec![
                string_build.into(),
                span1,
                "\" matches with \"".into(),
                span2,
                string2_build.into(),
            ];
            line_parts.extend(span3);

            ListItem::new(Line::from(line_parts))
        })
        .collect();
    let messages = List::new(messages).block(Block::bordered().title(" Messages "));
    f.render_widget(messages, messages_area);
}
