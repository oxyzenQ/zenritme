mod cli;
mod engine;
mod mode;
mod render;
mod sound;

use std::io::{self, Read, Write};
use std::process::{self, Command, Stdio};
use std::sync::mpsc;

struct TerminalGuard {
    tty: Option<std::fs::File>,
}

impl TerminalGuard {
    fn new() -> Self {
        let tty = std::fs::File::open("/dev/tty").ok();
        if let Some(t) = tty.as_ref().and_then(|f| f.try_clone().ok()) {
            let _ = Command::new("stty")
                .arg("-icanon")
                .arg("-echo")
                .arg("min")
                .arg("1")
                .arg("time")
                .arg("0")
                .stdin(Stdio::from(t))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        Self { tty }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if let Some(t) = self.tty.as_ref().and_then(|f| f.try_clone().ok()) {
            let _ = Command::new("stty")
                .arg("sane")
                .stdin(Stdio::from(t))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

fn setup_input() -> (TerminalGuard, Option<mpsc::Receiver<u8>>) {
    let guard = TerminalGuard::new();
    let Ok(mut tty) = std::fs::File::open("/dev/tty") else {
        return (guard, None);
    };

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = [0u8; 1];
        while let Ok(()) = tty.read_exact(&mut buf) {
            let _ = tx.send(buf[0]);
        }
    });

    (guard, Some(rx))
}

fn main() {
    let cmd = match cli::parse_args(std::env::args().skip(1)) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", err);
            eprintln!("\n{}", cli::usage());
            process::exit(2);
        }
    };

    match cmd {
        cli::Command::Help => {
            println!("{}", cli::usage());
        }
        cli::Command::SoundTest => {
            sound::sound_test();
        }
        cli::Command::Run(mode) => {
            let (_term, rx) = setup_input();
            let mut engine = engine::Engine::new(mode);
            loop {
                if let Some(rx) = rx.as_ref() {
                    while let Ok(b) = rx.try_recv() {
                        if b == b'q' || b == b'Q' || b == 3 {
                            let mut stdout = io::stdout();
                            let _ = stdout.write_all(b"\x1b[2J\x1b[H");
                            let _ = stdout.flush();
                            return;
                        }

                        if b == 27 {
                            match rx.try_recv() {
                                Ok(next) if next == b'[' || next == b'O' => {
                                    while let Ok(_more) = rx.try_recv() {}
                                }
                                Ok(_) | Err(mpsc::TryRecvError::Empty) => {
                                    let mut stdout = io::stdout();
                                    let _ = stdout.write_all(b"\x1b[2J\x1b[H");
                                    let _ = stdout.flush();
                                    return;
                                }
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    let mut stdout = io::stdout();
                                    let _ = stdout.write_all(b"\x1b[2J\x1b[H");
                                    let _ = stdout.flush();
                                    return;
                                }
                            }
                        }
                    }
                }

                engine.tick();
                let elapsed = engine.elapsed();
                let remaining = engine.remaining();

                render::draw(engine.mode(), elapsed, remaining);

                if let Some(ev) = engine.take_event() {
                    match ev {
                        engine::EngineEvent::Completed => {
                            sound::beep(3);
                            break;
                        }
                        engine::EngineEvent::PhaseSwitched => {
                            sound::beep(1);
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
