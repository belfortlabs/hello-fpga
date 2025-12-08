use std::io::{self, Result};
use std::sync::mpsc::Sender;
use std::thread;

use crossterm::style::Color;

use crate::demo::constants::{ERROR_COLOR, RESULT_COLOR};
use crate::demo::crypto::homomorphic_addition;
use crate::demo::{app::App, ui::drawer::Drawer};

use super::spinner::{CompletedMark, Spinner};

#[derive(Debug, Clone, Copy)]
pub(crate) enum LogKind {
    Client,
    Server,
}

#[derive(Debug)]
pub(crate) enum WorkerMsg {
    Log(LogKind, String, Color),
    StartStep {
        id: u32,
        kind: LogKind,
        label: String,
        color: Color,
    },
    EndStep {
        id: u32,
        ok: bool,
    },
    Done(Result<u128>),
}

pub(crate) trait WorkerManager {
    fn spawn_worker(&mut self, tx: Sender<WorkerMsg>, x: u128, y: u128);
    fn drain_worker_messages(&mut self) -> Result<()>;
    fn handle_result(&mut self, res: Result<u128>) -> Result<()>;
    fn remember_and_maybe_print(&mut self, kind: LogKind, msg: String, color: Color);
}

impl WorkerManager for App {
    fn spawn_worker(&mut self, tx: Sender<WorkerMsg>, x: u128, y: u128) {
        thread::spawn(move || {
            let mut next_step_id: u32 = 1;

            let mut start_step = |kind: LogKind, label: &str, color: Color| -> u32 {
                let id = next_step_id;
                next_step_id += 1;

                let _ = tx.send(WorkerMsg::StartStep {
                    id,
                    kind,
                    label: label.to_string(),
                    color,
                });

                id
            };

            let mut end_step = |id: u32, ok: bool| {
                let _ = tx.send(WorkerMsg::EndStep { id, ok });
            };

            let mut client_log = |m: &str, color: Color| {
                let _ = tx.send(WorkerMsg::Log(LogKind::Client, m.to_string(), color));
            };
            let mut server_log = |m: &str, color: Color| {
                let _ = tx.send(WorkerMsg::Log(LogKind::Server, m.to_string(), color));
            };

            let outcome = homomorphic_addition(
                x,
                y,
                &mut client_log,
                &mut server_log,
                &mut start_step,
                &mut end_step,
            );

            let _ = tx.send(WorkerMsg::Done(outcome));
        });
    }

    fn drain_worker_messages(&mut self) -> Result<()> {
        let mut outcome: Option<Result<u128>> = None;

        while let Some(msg) = self.current_job.as_ref().and_then(|rx| rx.try_recv().ok()) {
            match msg {
                WorkerMsg::Log(kind @ LogKind::Client, line, color)
                | WorkerMsg::Log(kind @ LogKind::Server, line, color) => {
                    self.remember_and_maybe_print(kind, line, color);
                }

                WorkerMsg::StartStep {
                    kind,
                    id,
                    label,
                    color,
                } => {
                    self.remember_and_maybe_print(kind, label, color);

                    let line_idx = if self.too_small {
                        match kind {
                            LogKind::Client => App::compute_line_idx_from_history(
                                &self.client_history,
                                self.client_box.get_width(),
                            ),
                            LogKind::Server => App::compute_line_idx_from_history(
                                &self.server_history,
                                self.server_box.get_width(),
                            ),
                        }
                    } else {
                        match kind {
                            LogKind::Client => self.client_box.current_line_idx(),
                            LogKind::Server => self.server_box.current_line_idx(),
                        }
                    };

                    self.spinners.push(Spinner::new(id, kind, line_idx, color));
                }

                WorkerMsg::EndStep { id, ok } => {
                    if let Some(pos) = self
                        .spinners
                        .iter()
                        .position(|spinner| spinner.get_id() == id)
                    {
                        let spinner = self.spinners.remove(pos);
                        self.finished_marks.push(CompletedMark::new(
                            spinner.get_kind(),
                            spinner.get_line_idx(),
                            ok,
                        ));
                        if !self.too_small {
                            self.draw_mark_at(spinner.get_kind(), spinner.get_line_idx(), ok)?;
                        }
                    }
                }

                WorkerMsg::Done(res) => outcome = Some(res),
            }
        }

        self.animate_spinners()?;

        if let Some(res) = outcome {
            self.handle_result(res)?;
        }
        Ok(())
    }

    fn handle_result(&mut self, res: Result<u128>) -> Result<()> {
        self.result_opt = Some(
            res.as_ref()
                .map_err(|e| io::Error::new(e.kind(), e.to_string()))
                .cloned(),
        );

        if !self.too_small {
            self.result_box.draw(&mut self.stdout)?;
            self.draw_equality()?;

            match res {
                Ok(v) => {
                    self.result_box
                        .log(&mut self.stdout, &v.to_string(), RESULT_COLOR)?;
                }
                Err(_) => {
                    self.result_box
                        .log(&mut self.stdout, "Error", ERROR_COLOR)?;
                }
            }
            self.draw_hint()?;
            self.draw_instruction()?;
        }
        self.current_job = None;
        Ok(())
    }

    fn remember_and_maybe_print(&mut self, kind: LogKind, msg: String, color: Color) {
        match kind {
            LogKind::Client => {
                self.client_history.push((msg.clone(), color));
                if !self.too_small {
                    let _ = self.client_box.log(&mut self.stdout, &msg, color);
                }
            }
            LogKind::Server => {
                self.server_history.push((msg.clone(), color));
                if !self.too_small {
                    let _ = self.server_box.log(&mut self.stdout, &msg, color);
                }
            }
        }
    }
}
