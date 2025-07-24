use enc_struct::EncStruct;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Gauge, List, ListItem, Paragraph};

use std::collections::HashMap;
use std::time::Instant;
use std::{error::Error, io};
use tfhe::shortint::prelude::*;

use pad::PadStr;

mod app;
mod data;
mod enc_struct;
mod util;
use crate::app::App;
use crate::app::InputMode;

use tfhe::integer::fpga::BelfortServerKey;

use tfhe::integer::ServerKey as IntegerServerKey;

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
    // security = 132 bits, p-fail = 2^-71.625
    let mut v0_11_param_message_leuvenshtein =
        tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS.clone();
    v0_11_param_message_leuvenshtein.message_modulus = MessageModulus(16);
    v0_11_param_message_leuvenshtein.carry_modulus = CarryModulus(1);

    let params: ClassicPBSParameters = v0_11_param_message_leuvenshtein;
    let cks: ClientKey = ClientKey::new(params);
    let sks: ServerKey = ServerKey::new(&cks);
    let integer_server_key: IntegerServerKey =
        tfhe::integer::ServerKey::new_radix_server_key_from_shortint(sks.clone());
    let mut fpga_key = BelfortServerKey::from(&integer_server_key);

    let db_size = data::NAME_LIST.len();

    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(db_size);

    for i in 0..db_size {
        db_len.insert(i, data::NAME_LIST[i].len());
    }

    let db_max_size = *db_len.values().into_iter().max().unwrap();

    let mut max_factor = std::cmp::max(db_max_size, 25);
    max_factor += 1;

    let mut db_processed: HashMap<usize, HashMap<char, Vec<Ciphertext>>> = HashMap::new();

    let ascii_collection = (20..126).collect::<Vec<u8>>();

    for k in 0..db_size {
        let t = data::NAME_LIST[k].pad_to_width(max_factor - 1);
        let m = t.len();
        let mut peq = HashMap::new();
        let mut peq_plain = HashMap::new();

        for i in &ascii_collection {
            let s = *i as char;
            let mut bitvec = vec![0u8; m];

            for j in 0..m {
                let pj = t.chars().nth(j).unwrap();
                if s == pj {
                    bitvec[j] = 9;
                }
            }

            let vec_enc = bitvec
                .iter()
                .map(|c| cks.encrypt(*c as u64)) // Encrypts
                .collect::<Vec<tfhe::shortint::Ciphertext>>();

            peq_plain.insert(s, bitvec);
            peq.insert(s, vec_enc);
        }
        db_processed.insert(k, peq);
    }

    let mut enc_struct = EncStruct {
        input: String::new(),
        query: String::new(),
        max_factor,
        db_size: data::NAME_LIST.len(),
        th: 0,
        time: Instant::now(),
        q_enc: Vec::new(),
        q2_enc: Vec::new(),
        db_enc_matrix: Vec::new(),
        db1_enc_matrix: Vec::new(),
        db_enc_map: db_processed,
        sks,
        cks,
        fpga_key: &mut fpga_key,
        one_enc_vec: Vec::new(),
        v_matrices: Vec::new(),
        h_matrices: Vec::new(),
        lut_1eq_vec_sw: Vec::new(),
        lut_eq_vec_sw: Vec::new(),
        lut_min_vec_sw: Vec::new(),
        lut_1eq_vec_fpga: Vec::new(),
        lut_eq_vec_fpga: Vec::new(),
        lut_min_vec_fpga: Vec::new(),
    };

    #[cfg(feature = "fpga")]
    {
        enc_struct.fpga_key.connect();
    }

    loop {
        terminal.draw(|f| ui(f, &app, &enc_struct))?;

        if matches!(app.input_mode, InputMode::Process) {
            if enc_struct.input.starts_with("p:") {
                if app.progress_done.len() == 0 {
                    app.process_plain_query_enc_db(&mut enc_struct);
                    app.progress_done.push(0);
                    app.process_plain_part_i(1, &mut enc_struct, false);
                } else if app.progress_done.len() >= enc_struct.max_factor {
                    app.post_process(&mut enc_struct, false);
                    app.input_mode = InputMode::Normal;
                } else {
                    app.process_plain_part_i(app.progress_done.len(), &mut enc_struct, false);
                }
            } else {
                if app.progress_done.len() == 0 {
                    app.process_enc_query_enc_db(&mut enc_struct);
                    app.progress_done.push(0);
                    app.process_part_i(1, &mut enc_struct, false);
                } else if app.progress_done.len() >= enc_struct.max_factor {
                    app.post_process(&mut enc_struct, false);
                    app.input_mode = InputMode::Normal;
                } else {
                    app.process_part_i(app.progress_done.len(), &mut enc_struct, false);
                }
            }
        } else if matches!(app.input_mode, InputMode::FProcess) {
            if app.input.starts_with("p:") {
                if app.progress_done.len() == 0 {
                    app.process_plain_query_enc_db(&mut enc_struct);

                    app.progress_done.push(0);
                    app.process_plain_part_i(1, &mut enc_struct, true);
                } else if app.progress_done.len() >= enc_struct.max_factor {
                    app.post_process(&mut enc_struct, true);

                    app.input_mode = InputMode::Normal;
                } else {
                    app.process_plain_part_i(app.progress_done.len(), &mut enc_struct, true);
                }
            } else {
                if app.progress_done.len() == 0 {
                    app.process_enc_query_enc_db(&mut enc_struct);

                    app.progress_done.push(0);
                    app.process_part_i(1, &mut enc_struct, true);
                } else if app.progress_done.len() >= enc_struct.max_factor {
                    app.post_process(&mut enc_struct, true);

                    app.input_mode = InputMode::Normal;
                } else {
                    app.process_part_i(app.progress_done.len(), &mut enc_struct, true);
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
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        enc_struct.input = app.input.clone();

                        if enc_struct.input.starts_with("p:") {
                            enc_struct.query = enc_struct.input.chars().skip(2).collect();
                        } else {
                            enc_struct.query = enc_struct.input.clone();
                        }
                        app.input_mode = InputMode::Process;
                        // }
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
                },

                InputMode::FEditing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        enc_struct.input = app.input.clone();

                        if enc_struct.input.starts_with("p:") {
                            enc_struct.query = enc_struct.input.chars().skip(2).collect();
                        } else {
                            enc_struct.query = enc_struct.input.clone();
                        }
                        app.input_mode = InputMode::FProcess;
                        // }
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
                },
                InputMode::Editing => {}
                InputMode::Process => {}
                InputMode::FEditing => {}
                InputMode::FProcess => {}
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
    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, help_area);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
            InputMode::Process => Style::default().fg(Color::Blue),
            InputMode::FEditing => Style::default().fg(Color::Yellow),
            InputMode::FProcess => Style::default().fg(Color::Blue),
        })
        .block(Block::bordered().title(" Input "));
    f.render_widget(input, input_area);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + app.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            ));
        }
        InputMode::Process => {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
        }
        InputMode::FEditing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                Position::new(
                    input_area.x + app.character_index as u16 + 1,
                    // Move one line down, from the border to the input line
                    input_area.y + 1,
                ),
            );
        }
        InputMode::FProcess => {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
        }
    }

    let total_round: usize = enc_struct.max_factor;

    let done = app.progress_done.len();
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

            let span3: Vec<Span<'_>>;

            if &m.3 == "Normal execution" {
                span3 = vec![<String as Clone>::clone(&m.3).blue().bold()];
            } else if &m.3 == "plaintext query" {
                span3 = vec![<String as Clone>::clone(&m.3).magenta().bold()];
            } else if &m.3 == "FPGA Acceleration" {
                span3 = vec![<String as Clone>::clone(&m.3).yellow().bold()];
            } else {
                span3 = vec![
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
                ];
            }

            let mut line_part_1 = vec![
                string_build.into(),
                span1,
                "\" matches with \"".into(),
                span2,
                string2_build.into(),
            ];
            line_part_1.append(&mut span3.clone());

            let line: Line<'_> = Line::from(line_part_1);

            ListItem::new(line)
        })
        .collect();
    let messages = List::new(messages).block(Block::bordered().title(" Messages "));
    f.render_widget(messages, messages_area);
}
