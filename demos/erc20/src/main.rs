use crossterm::style::Color::Yellow;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::{
    cursor,
    event::{self, Event},
    terminal, ExecutableCommand,
};
use rand::Rng;
use std::io::{stdout, Write};
use std::time;

const RECT_WIDTH: u16 = 15;
const RECT_HEIGHT: u16 = 5;
const NAMES: &[&str] = &[
    "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy", "Karl",
    "Laura", "Mallory", "Niaj", "Olivia", "Peggy", "Quentin", "Rupert", "Sybil", "Trent", "Uma",
    "Victor", "Wendy", "Xavier", "Yvonne", "Zara",
];

#[derive(Debug)]
enum ExecutionType {
    Cpu,
    Fpga,
}

fn main() {
    let mut stdout = stdout();
    let mut counter = 0;
    let mut execution_type = ExecutionType::Cpu; // Default is cpu

    // Enable raw mode for better key capture
    crossterm::terminal::enable_raw_mode().unwrap();
    // Hide the cursor at the start
    stdout.execute(cursor::Hide).unwrap();

    loop {
        let (cols, rows) = terminal::size().expect("Failed name_to get terminal size");
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .unwrap();

        let x1_initial = cols / 4 - RECT_WIDTH / 2;
        let y1_initial = rows / 2 - RECT_HEIGHT / 2;
        let x2_initial = cols * 3 / 4 - RECT_WIDTH / 2;
        let y2_initial = rows / 2 - RECT_HEIGHT / 2;

        let mut from = String::new();
        let mut amount = String::new();
        let mut to = String::new();

        let (x1, y1, x2, y2, name_fromm, name_to, from, amount, to, color) = match counter {
            0 => (
                x1_initial,
                y1_initial,
                x2_initial,
                y2_initial,
                "Alice",
                "Bob",
                "$50",
                "$40",
                "$30",
                Color::Green,
            ),
            1 => (
                x1_initial,
                y1_initial,
                x2_initial,
                y2_initial,
                "Alice",
                "Bob",
                "###",
                "###",
                "###",
                Color::Red,
            ),
            2 => (
                x1_initial,
                y1_initial,
                x2_initial,
                y2_initial,
                "Alice",
                "Bob",
                "Enc($50)",
                "Enc($40)",
                "Enc($30)",
                Color::Red,
            ),
            _ => {
                let mut rng = rand::thread_rng();
                let mut x1;
                let mut x2;
                let mut y1;
                let mut y2;
                // Ensure |x1 - x2| is even
                loop {
                    x1 = rng.gen_range(1..(cols / 2).saturating_sub(RECT_WIDTH));
                    x2 = rng.gen_range((cols / 2)..cols.saturating_sub(RECT_WIDTH));
                    if (x1 as i32 - x2 as i32).abs() % 2 == 0 {
                        break;
                    }
                }
                // Ensure |y1 - y2| >= 4
                loop {
                    y1 = rng.gen_range(1..rows.saturating_sub(RECT_HEIGHT));
                    y2 = rng.gen_range(1..rows.saturating_sub(RECT_HEIGHT));
                    if (y1 as i32 - y2 as i32).abs() >= 4 {
                        break;
                    }
                }
                from = format!("Enc(${})", rng.gen_range(20..100));
                amount = format!("Enc(${})", rng.gen_range(1..100));
                to = format!("Enc(${})", rng.gen_range(1..100));

                // Pick two different random names
                let idx1 = rng.gen_range(0..NAMES.len());
                let mut idx2;
                loop {
                    idx2 = rng.gen_range(0..NAMES.len());
                    if idx2 != idx1 {
                        break;
                    }
                }
                let name_fromm = NAMES[idx1];
                let name_to = NAMES[idx2];

                (
                    x1,
                    y1,
                    x2,
                    y2,
                    name_fromm,
                    name_to,
                    from.as_str(),
                    amount.as_str(),
                    to.as_str(),
                    Color::Red,
                )
            }
        };

        draw_rectangle(&mut stdout, x1, y1, RECT_WIDTH, RECT_HEIGHT);
        draw_rectangle(&mut stdout, x2, y2, RECT_WIDTH, RECT_HEIGHT);
        draw_arrow(
            &mut stdout,
            x1 + RECT_WIDTH,
            y1 + RECT_HEIGHT / 2,
            x2,
            y2 + RECT_HEIGHT / 2,
        );
        draw_text(&mut stdout, x1, y1, name_fromm, Color::White);
        draw_text(&mut stdout, x2, y2, name_to, Color::White);
        draw_text(&mut stdout, x1, y1 + 1, from, color);
        draw_text(&mut stdout, (x1 + x2) / 2, (y1 + y2) / 2, amount, color);
        draw_text(&mut stdout, x2, y2 + 1, to, color);

        stdout.flush().unwrap();

        // Move cursor to the last row to avoid overwriting drawings
        stdout.execute(cursor::MoveTo(0, rows - 1)).unwrap();

        // Print execution_type at the bottom if counter > 2
        if counter > 2 {
            stdout.execute(SetForegroundColor(Yellow)).unwrap();
            print!("{}", format!("{:?}", execution_type).to_uppercase());
            stdout.execute(ResetColor).unwrap();
        }

        // Wait for a key press for the first 4 iterations, otherwise just iterate continuously and handle key
        if counter < 3 {
            loop {
                if event::poll(time::Duration::from_millis(100)).unwrap() {
                    if let Event::Key(key_event) = event::read().unwrap() {
                        match key_event {
                            _ => break,
                        }
                    }
                }
            }
        } else {
            if event::poll(time::Duration::from_millis(1)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event {
                        event::KeyEvent {
                            code: event::KeyCode::Char('f'),
                            ..
                        } => {
                            execution_type = ExecutionType::Fpga;
                        }
                        event::KeyEvent {
                            code: event::KeyCode::Char('c'),
                            ..
                        } => {
                            execution_type = ExecutionType::Cpu;
                        }
                        event::KeyEvent {
                            code: event::KeyCode::Char('e'),
                            ..
                        } => {
                            crossterm::terminal::disable_raw_mode().unwrap();
                            stdout.execute(cursor::Show).unwrap();
                            return;
                        }
                        _ => {}
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        counter += 1;
    }
}

fn draw_rectangle(stdout: &mut std::io::Stdout, x: u16, y: u16, width: u16, height: u16) {
    stdout.execute(cursor::MoveTo(x, y)).unwrap();
    println!("┌{}┐", "─".repeat((width - 2) as usize));
    for row in 1..height - 1 {
        stdout.execute(cursor::MoveTo(x, y + row)).unwrap();
        println!("│{}│", " ".repeat((width - 2) as usize));
    }
    stdout.execute(cursor::MoveTo(x, y + height - 1)).unwrap();
    println!("└{}┘", "─".repeat((width - 2) as usize));
}

fn draw_arrow(stdout: &mut std::io::Stdout, x1: u16, y1: u16, x2: u16, y2: u16) {
    let mid_x = (x1 + x2) / 2;
    let mut x = x1;
    let mut y = y1;

    // Draw horizontal line from left rectangle to midpoint (─)
    while x < mid_x {
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("─");
        x += 1;
    }

    if y == y2 {
        print!("─");
    }
    // Draw corner ┐ or └
    else if y < y2 {
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("┐");
        // Draw vertical line down (│)
        while y < y2 - 1 {
            y += 1;
            stdout.execute(cursor::MoveTo(x, y)).unwrap();
            print!("│");
        }
        y += 1;
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("└");
    } else if y > y2 {
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("┘");
        // Draw vertical line up (│)
        while y > y2 + 1 {
            y -= 1;
            stdout.execute(cursor::MoveTo(x, y)).unwrap();
            print!("│");
        }
        y -= 1;
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("┌");
    }

    // Draw horizontal line from midpoint to right rectangle (─)
    while x < x2 - 1 {
        x += 1;
        stdout.execute(cursor::MoveTo(x, y)).unwrap();
        print!("─");
    }

    // Draw arrow head
    stdout.execute(cursor::MoveTo(x2 - 1, y2)).unwrap();
    print!(">");
}

fn draw_text(stdout: &mut std::io::Stdout, x: u16, y: u16, text: &str, color: Color) {
    // Center the text within the rectangle
    let text_width = text.len() as u16;
    let center_x = x + RECT_WIDTH / 2;
    let center_y = y + RECT_HEIGHT / 2;
    let start_x = if center_x > text_width / 2 {
        center_x - text_width / 2
    } else {
        0
    };
    stdout.execute(cursor::MoveTo(start_x, center_y)).unwrap();
    stdout.execute(SetForegroundColor(color)).unwrap();
    print!("{text}");
    stdout.execute(ResetColor).unwrap();
}
