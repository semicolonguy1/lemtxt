use clap::Parser;
use ratatui::crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};
use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;
use std::collections::HashMap;

const TICK_RATE: Duration = Duration::from_millis(500);

#[derive(Parser, Debug)]
#[command(name = "lem")]
struct Args {
    #[arg(short, long)]
    f: String,
}

struct App {
    running: bool,
    ticks: u64,
    key_presses: u64,
    text: String,
    mode: String,
    ptr: u64,
    esc_text: String,
    table: HashMap<&'static str, Box<dyn Fn(&[&str]) -> Result<()>>>,
}


fn idk() -> String {
    let args = Args::parse();
    let content = fs::read_to_string(args.f).expect("idk");

    content
}

fn getfilename() -> String {
    let args = Args::parse();
    let name = args.f;

    name
}

fn cmd_write(contents: &str) -> Result<()> {
    let mut file = File::create(getfilename())?;
    file.write_all(contents.as_bytes())
}

fn quit(running: &mut bool) {
    *running = false;
}

fn main() -> std::io::Result<()> {

    execute!(std::io::stdout(), SetCursorStyle::SteadyBlock)?;
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

impl App {
    fn new() -> Self {
        let mut table: HashMap<&'static str, Box<dyn Fn(&[&str]) -> Result<()>>> = HashMap::new();

        table.insert(
            "w",
            Box::new(|args: &[&str]| {
                let content = if args.len() > 1 { args[1] } else { "" };
                cmd_write(content)
            }),
        );

        Self {
            running: true,
            ticks: 0,
            key_presses: 0,
            text: idk(),
            mode: String::from("normal"),
            ptr: 0,
            esc_text: String::from(""),
            table,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut last_tick = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key, &mut self.text.clone());
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, text: &mut String) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        self.key_presses += 1;

        if self.mode == "normal" {
            match key.code {
                KeyCode::Esc => self.mode = String::from("command"),
                KeyCode::Tab => self.running = false,
                KeyCode::Char('q') => {
                    self.text.insert_str(self.ptr.clone() as usize, "q");
                    self.ptr += 1
                }
                KeyCode::Backspace => {
                    self.text.remove(self.ptr.clone() as usize - 1);
                    self.ptr -= 1;
                }

                KeyCode::Char(c) => {
                    self.text.insert(self.ptr.clone() as usize, c);
                    self.ptr += 1;
                }
                KeyCode::Enter => {
                    self.text.insert_str(self.ptr.clone() as usize, "\n");
                    self.ptr += 1;
                }
                KeyCode::Left => {if self.ptr != 0 {self.ptr -= 1}},
                KeyCode::Right => {if self.ptr != self.text.len() as u64 {self.ptr += 1}},
                _ => {}
            }
        }

        if self.mode == "command" {
            match key.code {
                KeyCode::Char('w') => {
                    cmd_write(&self.text);
                }
                // KeyCode::Backspace => {
                //     self.esc_text.pop();
                // }
                KeyCode::Tab => {
                    self.mode = String::from("normal");
                }

                // KeyCode::Char(c) => {
                //     self.esc_text.push(c);
                // }

                // KeyCode::Enter => {
                //     let parts: Vec<&str> = self.esc_text.split_whitespace().collect();

                //     if !parts.is_empty() {
                //         let cmd_name = parts[0];


                //         if cmd_name == "w" {
                //             let content_to_write = if parts.len() > 1 {
                //                 parts[1..].join(" ") // إذا كتب نصاً بعد w
                //             } else {
                //                 self.text.clone()   
                //             };

                //             let mut file = File::create(getfilename()).unwrap();
                //             let _ = file.write_all(content_to_write.as_bytes());
                //         } 

                //         if cmd_name == "q" {
                //             self.running = false;
                //         }
                //         else if let Some(func) = self.table.get(cmd_name) {
                //             let _ = func(&parts);
                //         } else {
                //             // println!("Command not found: {}", cmd_name);
                //         }
                //     }

                //     self.esc_text.clear();
                //     self.mode = String::from("normal");
                // }
                _ => {}
            }
        }
    }

    fn on_tick(&mut self) {
        self.ticks += 1;
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new(self.text.clone()), frame.area());

        let before: String = self.text.chars().take(self.ptr.clone() as usize).collect();
        let y = before.matches('\n').count() as u16;
        let last_line = before.rsplit('\n').next().unwrap_or("");
        let x = UnicodeWidthStr::width(last_line) as u16;

        frame.set_cursor_position((x, y));
    }
}
