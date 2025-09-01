use std::io::{self, Error, ErrorKind, Result, Stdout, Write, stdout};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crossterm::QueueableCommand;
use crossterm::cursor::{MoveTo, Show};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers, poll};
use crossterm::style::{Color, PrintStyledContent, Stylize, style};
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::{
    ExecutableCommand,
    cursor::Hide,
    event::{self, Event},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, size},
};

use crate::demo::{
    constants::{
        CLIENT_BOX_FROM_LEFT, CLIENT_BOX_FROM_TOP, CLIENT_SERVER_BOX_HEIGHT,
        CLIENT_SERVER_BOX_WIDTH, OPERAND_AND_RESULT_BOXES_FROM_TOP, OPERAND_ONE_FROM_LEFT,
        OPERAND_RESULT_BOX_HEIGHT, OPERAND_RESULT_BOX_WIDTH, OPERAND_TWO_FROM_LEFT,
        RESULT_BOX_FROM_LEFT, SERVER_BOX_FROM_LEFT, SERVER_BOX_FROM_TOP, TERMINAL_POLL_MS,
    },
    ui::{
        drawer::Drawer,
        layout::LayoutManager,
        widgets::{InputBox, OutputBox},
        worker::{WorkerManager, WorkerMsg},
    },
};

use super::constants::{CLIENT_SERVER_BOX_PAD, SPIN_FRAMES, SPIN_INTERVAL};
use super::ui::spinner::{CompletedMark, Spinner};
use super::ui::worker::LogKind;

pub struct App {
    pub(crate) stdout: Stdout,
    pub(crate) offsets: (u16, u16),
    pub(crate) last_size: (u16, u16),
    pub(crate) too_small: bool,

    pub(crate) client_box: OutputBox,
    pub(crate) server_box: OutputBox,
    pub(crate) result_box: OutputBox,

    pub(crate) client_history: Vec<(String, Color)>,
    pub(crate) server_history: Vec<(String, Color)>,
    pub(crate) x_val_opt: Option<u128>,
    pub(crate) y_val_opt: Option<u128>,
    pub(crate) result_opt: Option<Result<u128>>,
    pub(crate) current_job: Option<Receiver<WorkerMsg>>,

    pub(crate) spinners: Vec<Spinner>,
    pub(crate) finished_marks: Vec<CompletedMark>,
    pub(crate) last_spin: Instant,
}

impl App {
    pub fn init() -> Result<Self> {
        let original_size = size()?;
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;

        let mut app = Self {
            stdout,
            offsets: (0, 0),
            last_size: original_size,
            too_small: false,

            client_box: OutputBox::empty(),
            server_box: OutputBox::empty(),
            result_box: OutputBox::empty(),

            client_history: Vec::new(),
            server_history: Vec::new(),
            x_val_opt: None,
            y_val_opt: None,
            result_opt: None,
            current_job: None,

            spinners: Vec::new(),
            finished_marks: Vec::new(),
            last_spin: Instant::now(),
        };

        app.offsets = app.offsets_for(original_size);

        app.build_ui_boxes()?;

        app.too_small = app.terminal_too_small(original_size);

        if app.too_small {
            app.warn_resize(original_size)?;
        } else {
            app.draw_hint()?;
            app.draw_instruction()?;
        }

        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.maybe_handle_resize()?;

            if event::poll(Duration::from_millis(TERMINAL_POLL_MS))? {
                let ev = event::read()?;

                if self.should_exit(&ev) {
                    break;
                }
                if self.should_restart(&ev) {
                    self.reset_ui()?;
                    continue;
                }
                if let Event::Resize(cols, rows) = ev {
                    self.on_resize((cols, rows))?;
                    continue;
                }
                if self.too_small {
                    self.warn_resize(size()?)?;
                }
                if self.current_job.is_none() && self.result_opt.is_none() && !self.too_small {
                    if let Err(e) = self.prompt_operands_and_start_job() {
                        if e.kind() != ErrorKind::Other {
                            break;
                        }
                    }
                }
            }
            self.drain_worker_messages()?;
        }
        Ok(())
    }

    pub(crate) fn maybe_handle_resize(&mut self) -> Result<()> {
        if let Ok(new_size) = size() {
            if new_size != self.last_size {
                self.on_resize(new_size)?;
            }
        }
        Ok(())
    }

    pub(crate) fn on_resize(&mut self, new_size: (u16, u16)) -> Result<()> {
        self.stdout.execute(Clear(ClearType::All))?;

        self.last_size = new_size;
        self.offsets = self.offsets_for(new_size);
        self.too_small = self.terminal_too_small(new_size);

        if self.too_small {
            self.warn_resize(new_size)?;
        } else {
            self.redraw_static_ui()?;

            self.draw_hint()?;
            self.draw_instruction()?;
        }

        Ok(())
    }

    pub(crate) fn prompt_operands_and_start_job(&mut self) -> Result<()> {
        self.stdout.execute(Clear(ClearType::All))?;
        self.redraw_static_ui()?;

        let x = self.ask_for_operand(1, None)?;
        self.x_val_opt = Some(x);

        self.draw_plus()?;

        let y = self.ask_for_operand(2, Some(x))?;
        self.y_val_opt = Some(y);

        self.client_box.clear_inside(&mut self.stdout)?;
        self.server_box.clear_inside(&mut self.stdout)?;
        self.client_history.clear();
        self.server_history.clear();

        let (tx, rx) = mpsc::channel();

        self.spawn_worker(tx, x, y);
        self.current_job = Some(rx);
        Ok(())
    }

    pub(crate) fn ask_for_operand(&mut self, pos: u8, previous: Option<u128>) -> Result<u128> {
        match self.read_operands(pos.into(), previous) {
            Ok(v) => Ok(v),
            Err(ref e) if e.kind() == ErrorKind::Other => {
                self.reset_ui()?;
                Err(io::Error::new(ErrorKind::Other, "restart"))
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn read_operands(&mut self, which: u16, peer_val: Option<u128>) -> Result<u128> {
        let from_left = if which == 1 {
            OPERAND_ONE_FROM_LEFT
        } else {
            OPERAND_TWO_FROM_LEFT
        };

        let mut input_box = InputBox::new(
            (
                self.offsets.0 + from_left,
                self.offsets.1 + OPERAND_AND_RESULT_BOXES_FROM_TOP,
            ),
            OPERAND_RESULT_BOX_WIDTH,
            OPERAND_RESULT_BOX_HEIGHT,
            if which == 1 {
                "Operand 1:"
            } else {
                "Operand 2:"
            },
        );

        input_box.draw(&mut self.stdout)?;
        input_box.render(&mut self.stdout)?;

        if let Some(v) = peer_val {
            self.draw_operand(3 - which, v)?;
        }

        loop {
            if !poll(Duration::from_millis(TERMINAL_POLL_MS))? {
                continue;
            }

            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    if self.should_exit(&Event::Key(k.clone())) {
                        return Err(Error::new(ErrorKind::Interrupted, "User requested exit."));
                    }

                    if self.should_restart(&Event::Key(k.clone())) {
                        return Err(Error::new(ErrorKind::Other, "User requested restart."));
                    }

                    match k.code {
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            input_box.get_mut_buf().push(c);
                            input_box.render(&mut self.stdout)?;
                        }
                        KeyCode::Backspace => {
                            input_box.get_mut_buf().pop();
                            input_box.render(&mut self.stdout)?;
                        }
                        KeyCode::Enter => break,
                        _ => {}
                    }
                }

                Event::Resize(cols, rows) => {
                    self.stdout.execute(Clear(ClearType::All))?;
                    self.offsets = self.offsets_for((cols, rows));
                    if self.terminal_too_small((cols, rows)) {
                        self.warn_resize((cols, rows))?;
                        continue;
                    }

                    self.redraw_static_ui()?;

                    input_box.set_origin((
                        self.offsets.0 + from_left,
                        self.offsets.1 + OPERAND_AND_RESULT_BOXES_FROM_TOP,
                    ));
                    input_box.draw(&mut self.stdout)?;
                    input_box.render(&mut self.stdout)?;

                    if let Some(v) = peer_val {
                        self.draw_operand(3 - which, v)?;
                    }

                    for (line, color) in &mut *self.client_history {
                        let _ = self.client_box.log(&mut self.stdout, line, *color);
                    }

                    for (line, color) in &mut *self.server_history {
                        let _ = self.server_box.log(&mut self.stdout, line, *color);
                    }

                    self.draw_hint()?;
                    self.draw_instruction()?;
                }
                _ => {}
            }
        }

        if input_box.get_mut_buf().is_empty() {
            input_box.get_mut_buf().push('0');
            input_box.render(&mut self.stdout)?;
            return Ok(0);
        }

        input_box
            .get_mut_buf()
            .parse::<u128>()
            .map_err(|e| Error::new(ErrorKind::InvalidInput, e))
    }

    pub(crate) fn build_ui_boxes(&mut self) -> Result<()> {
        self.client_box = OutputBox::new(
            (
                CLIENT_BOX_FROM_LEFT + self.offsets.0,
                CLIENT_BOX_FROM_TOP + self.offsets.1,
            ),
            CLIENT_SERVER_BOX_WIDTH,
            CLIENT_SERVER_BOX_HEIGHT,
            "Client Side Operations:",
        );
        self.client_box.draw(&mut self.stdout)?;

        self.server_box = OutputBox::new(
            (
                SERVER_BOX_FROM_LEFT + self.offsets.0,
                SERVER_BOX_FROM_TOP + self.offsets.1,
            ),
            CLIENT_SERVER_BOX_WIDTH,
            CLIENT_SERVER_BOX_HEIGHT,
            "Server Side Operations:",
        );
        self.server_box.draw(&mut self.stdout)?;

        self.result_box = OutputBox::new(
            (
                RESULT_BOX_FROM_LEFT + self.offsets.0,
                OPERAND_AND_RESULT_BOXES_FROM_TOP + self.offsets.1,
            ),
            OPERAND_RESULT_BOX_WIDTH,
            OPERAND_RESULT_BOX_HEIGHT,
            "Result:",
        );

        Ok(())
    }

    pub(crate) fn reset_ui(&mut self) -> Result<()> {
        self.stdout.execute(Clear(ClearType::All))?;
        self.client_history.clear();
        self.server_history.clear();

        self.finished_marks.clear();
        self.spinners.clear();

        self.x_val_opt = None;
        self.y_val_opt = None;
        self.result_opt = None;
        self.current_job = None;

        self.offsets = self.offsets_for(size()?);
        self.build_ui_boxes()?;

        self.too_small = self.terminal_too_small(size()?);
        if self.too_small {
            self.warn_resize(size()?)?;
        } else {
            self.draw_hint()?;
            self.draw_instruction()?;
        }
        Ok(())
    }

    pub(crate) fn should_exit(&mut self, ev: &Event) -> bool {
        matches!(ev, Event::Key(k) if k.kind==KeyEventKind::Press && (
            k.code==KeyCode::Esc || k.code==KeyCode::Char('q') ||
            (k.code==KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL))
        ))
    }

    pub(crate) fn should_restart(&mut self, ev: &Event) -> bool {
        matches!(ev, Event::Key(k) if k.kind==KeyEventKind::Press && k.code==KeyCode::Char('r'))
    }

    pub(crate) fn animate_spinners(&mut self) -> Result<()> {
        if self.spinners.is_empty() || self.last_spin.elapsed() < SPIN_INTERVAL {
            return Ok(());
        }

        for spinner in &mut self.spinners {
            spinner.set_frame((spinner.get_frame() + 1) % SPIN_FRAMES.len());
        }
        self.last_spin = Instant::now();

        if self.too_small {
            return Ok(());
        }

        for spinner in &self.spinners {
            let (right_x, base_y) = match spinner.get_kind() {
                LogKind::Client => (
                    self.client_box.right_edge_x(),
                    self.client_box.get_origin().1,
                ),
                LogKind::Server => (
                    self.server_box.right_edge_x(),
                    self.server_box.get_origin().1,
                ),
            };
            let y = base_y + 1 + spinner.get_line_idx();
            let ch = SPIN_FRAMES[spinner.get_frame() % SPIN_FRAMES.len()];
            self.stdout
                .queue(MoveTo(right_x, y))?
                .queue(PrintStyledContent(style(ch).with(spinner.get_color())))?;
        }

        self.stdout.flush()?;

        Ok(())
    }

    pub(crate) fn compute_line_idx_from_history(
        history: &[(String, Color)],
        box_width: u16,
    ) -> u16 {
        let max_len_per_line = (box_width - CLIENT_SERVER_BOX_PAD) as usize;
        let mut num_lines: u16 = 0;

        for (msg, _) in history {
            for txt in msg.split('\n') {
                let stripped_txt = txt.trim_end();
                if stripped_txt.is_empty() {
                    num_lines += 1;
                    continue;
                }
                let chunks = (stripped_txt.len() + max_len_per_line - 1) / max_len_per_line;
                num_lines += chunks as u16;
            }
        }
        num_lines - 1
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.stdout.execute(Show);
        let _ = self.stdout.execute(LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
