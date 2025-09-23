use crossterm::style::Color::*;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::{
    cursor,
    event::{self, Event},
    terminal, ExecutableCommand,
};
use rand::Rng;
use std::io::{stdout, Stdout, Write};
use std::time;
use std::time::Instant;

// Enable FPGA: Import the BelfortServerKey
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::prelude::*;
use tfhe::set_server_key;
use tfhe::{ClientKey, ConfigBuilder, FheUint64};

const RECT_WIDTH: u16 = 15;
const RECT_HEIGHT: u16 = 5;
const NAMES: &[&str] = &[
    "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy", "Karl",
    "Laura", "Mallory", "Niaj", "Olivia", "Peggy", "Quentin", "Rupert", "Sybil", "Trent", "Uma",
    "Victor", "Wendy", "Xavier", "Yvonne", "Zara",
];

const INSTRUCTIONS: &[&str] = &[
    "Press 'f' to switch to FPGA execution",
    "Press 'c' to switch to CPU execution",
    "Press 'q' to quit",
];

#[derive(Debug)]
enum ExecutionType {
    Cpu,
    Fpga,
}

fn main() {
    // Create Keys
    let config = ConfigBuilder::default().build();
    let client_key = ClientKey::generate(config);
    let server_key = client_key.generate_server_key();

    let mut fpga_key = BelfortServerKey::from(&server_key);
    fpga_key.connect();

    set_server_key(server_key.clone());

    let mut stdout = stdout();
    let mut counter = 0;
    let mut execution_type = ExecutionType::Cpu; // Default is cpu

    // Enable raw mode for better key capture
    crossterm::terminal::enable_raw_mode().unwrap();
    // Hide the cursor at the start
    stdout.execute(cursor::Hide).unwrap();

    loop {
        let (cols, rows) = terminal::size().expect("Failed name_to get terminal size");

        let (x1, y1, x2, y2, name_fromm, name_to, from, amount, to, color, exec_time) =
            get_transaction_display(counter, cols, rows, &client_key);

        clear_terminal(&mut stdout);

        draw_rectangle(&mut stdout, x1, y1, RECT_WIDTH, RECT_HEIGHT);
        draw_rectangle(&mut stdout, x2, y2, RECT_WIDTH, RECT_HEIGHT);
        draw_arrow(
            &mut stdout,
            x1 + RECT_WIDTH,
            y1 + RECT_HEIGHT / 2,
            x2,
            y2 + RECT_HEIGHT / 2,
        );
        draw_text(&mut stdout, x1, y1, name_fromm, White);
        draw_text(&mut stdout, x2, y2, name_to, White);
        draw_text(&mut stdout, x1, y1 + 1, from, color);
        draw_text(&mut stdout, (x1 + x2) / 2, (y1 + y2) / 2, amount, color);
        draw_text(&mut stdout, x2, y2 + 1, to, color);

        stdout.flush().unwrap();

        // Move cursor to the last row to avoid overwriting drawings
        stdout.execute(cursor::MoveTo(0, rows - 1)).unwrap();

        if counter > 2 {
            stdout.execute(SetForegroundColor(Yellow)).unwrap();
            print!(
                "{} {:?}",
                format!("{:?}", execution_type).to_uppercase(),
                exec_time
            );
            draw_instructions(&mut stdout, 0, &execution_type);
            stdout.execute(ResetColor).unwrap();
        }

        // Wait for a key press for the first 4 iterations, otherwise just iterate continuously and handle key
        if counter < 3 {
            stdout.execute(cursor::MoveTo(0, 0)).unwrap();
            stdout.execute(SetForegroundColor(Cyan)).unwrap();
            print!("Press any key until animation starts");
            stdout.flush().unwrap();
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
                    match key_event.code {
                        event::KeyCode::Char('f') => {
                            execution_type = ExecutionType::Fpga;
                            set_server_key(fpga_key.clone());
                        }
                        event::KeyCode::Char('c') => {
                            execution_type = ExecutionType::Cpu;
                            set_server_key(server_key.clone());
                        }
                        event::KeyCode::Char('q') => {
                            crossterm::terminal::disable_raw_mode().unwrap();
                            clear_terminal(&mut stdout);
                            stdout.execute(cursor::MoveTo(0, 0)).unwrap();
                            stdout.execute(cursor::Show).unwrap();
                            fpga_key.disconnect();
                            return;
                        }
                        _ => {}
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        counter += 1;
    }
}

fn get_transaction_display(
    counter: usize,
    cols: u16,
    rows: u16,
    client_key: &ClientKey,
) -> (
    u16,
    u16,
    u16,
    u16,
    &'static str,
    &'static str,
    &str,
    &str,
    &str,
    Color,
    time::Duration,
) {
    const INTRO_FROM_NAME: &str = "Alice";
    const INTRO_TO_NAME: &str = "Bob";
    const INTRO_FROM_AMOUNT: &str = "$50";
    const INTRO_TRANSFER_AMOUNT: &str = "$40";
    const INTRO_TO_AMOUNT: &str = "$30";
    const INTRO_MASKED: &str = " ### ";
    const INTRO_ENC_FROM_AMOUNT: &str = "Enc($50)";
    const INTRO_ENC_TRANSFER_AMOUNT: &str = " Enc($40) ";
    const INTRO_ENC_TO_AMOUNT: &str = "Enc($30)";

    let mut exec_time = time::Duration::from_secs(0);

    match counter {
        0 => (
            cols / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            cols * 3 / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            INTRO_FROM_NAME,
            INTRO_TO_NAME,
            INTRO_FROM_AMOUNT,
            INTRO_TRANSFER_AMOUNT,
            INTRO_TO_AMOUNT,
            Green,
            exec_time,
        ),
        1 => (
            cols / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            cols * 3 / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            INTRO_FROM_NAME,
            INTRO_TO_NAME,
            INTRO_MASKED,
            INTRO_MASKED,
            INTRO_MASKED,
            Red,
            exec_time,
        ),
        2 => (
            cols / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            cols * 3 / 4 - RECT_WIDTH / 2,
            rows / 2 - RECT_HEIGHT / 2,
            INTRO_FROM_NAME,
            INTRO_TO_NAME,
            INTRO_ENC_FROM_AMOUNT,
            INTRO_ENC_TRANSFER_AMOUNT,
            INTRO_ENC_TO_AMOUNT,
            Red,
            exec_time,
        ),
        _ => {
            let mut rng = rand::rng();
            let mut x1;
            let mut x2;
            let mut y1;
            let mut y2;
            // Ensure |x1 - x2| is even
            loop {
                x1 = rng.random_range(1..(cols / 2).saturating_sub(RECT_WIDTH));
                x2 = rng.random_range((cols / 2)..cols.saturating_sub(RECT_WIDTH));
                if (x1 as i32 - x2 as i32).abs() % 2 == 0 {
                    break;
                }
            }
            // Ensure |y1 - y2| >= 4
            loop {
                y1 = rng.random_range(1..rows.saturating_sub(RECT_HEIGHT));
                y2 = rng.random_range(1..rows.saturating_sub(RECT_HEIGHT));
                if (y1 as i32 - y2 as i32).abs() >= 4 {
                    break;
                }
            }

            let from: u64 = rng.random_range(20..100);
            let amount: u64 = rng.random_range(1..from);
            let to: u64 = rng.random_range(1..100);

            let encrypted_from = FheUint64::encrypt(from, client_key);
            let encrypted_transfer = FheUint64::encrypt(amount, client_key);
            let encrypted_to = FheUint64::encrypt(to, client_key);

            let time_start = Instant::now();

            let (_encrypted_new_to, _encrypted_new_from) =
                erc20_transaction(&encrypted_transfer, &encrypted_to, &encrypted_from);

            exec_time = time_start.elapsed();

            let str_from = format!("Enc(${})", from - amount);
            let str_amount = format!(" Enc(${}) ", amount);
            let str_to = format!("Enc(${})", to + amount);

            // Pick two different random names
            let idx1 = rng.random_range(0..NAMES.len());
            let mut idx2;
            loop {
                idx2 = rng.random_range(0..NAMES.len());
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
                Box::leak(str_from.into_boxed_str()),
                Box::leak(str_amount.into_boxed_str()),
                Box::leak(str_to.into_boxed_str()),
                Red,
                exec_time,
            )
        }
    }
}

fn erc20_transaction(
    amount: &FheUint64,
    balance_to: &FheUint64,
    balance_from: &FheUint64,
) -> (FheUint64, FheUint64) {
    let transfer_value = amount
        .le(balance_to)
        .select(amount, &FheUint64::encrypt_trivial(0u64));

    let new_balance_to = balance_to + &transfer_value;
    let new_balance_from = balance_from - &transfer_value;

    (new_balance_to, new_balance_from)
}

fn clear_terminal(stdout: &mut Stdout) {
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

fn draw_rectangle(stdout: &mut Stdout, x: u16, y: u16, width: u16, height: u16) {
    stdout.execute(cursor::MoveTo(x, y)).unwrap();
    println!("┌{}┐", "─".repeat((width - 2) as usize));
    for row in 1..height - 1 {
        stdout.execute(cursor::MoveTo(x, y + row)).unwrap();
        println!("│{}│", " ".repeat((width - 2) as usize));
    }
    stdout.execute(cursor::MoveTo(x, y + height - 1)).unwrap();
    println!("└{}┘", "─".repeat((width - 2) as usize));
}

fn draw_arrow(stdout: &mut Stdout, x1: u16, y1: u16, x2: u16, y2: u16) {
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

fn draw_text(stdout: &mut Stdout, x: u16, y: u16, text: &str, color: Color) {
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

fn draw_instructions(stdout: &mut Stdout, cols: u16, execution: &ExecutionType) {
    let instr_x = cols.saturating_sub(40); // 35 chars from the right edge
    stdout.execute(cursor::MoveTo(instr_x, 0)).unwrap();
    stdout.execute(SetForegroundColor(Cyan)).unwrap();
    match execution {
        ExecutionType::Cpu => println!("{}", INSTRUCTIONS[0]),
        ExecutionType::Fpga => println!("{}", INSTRUCTIONS[1]),
    };
    stdout.execute(cursor::MoveTo(instr_x, 1)).unwrap();
    println!("{}", INSTRUCTIONS[2]);
    stdout.execute(ResetColor).unwrap();
}
