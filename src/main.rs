mod cli;
mod engine;
mod mode;
mod render;
mod sound;

use std::process;

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
            let mut engine = engine::Engine::new(mode);
            loop {
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
