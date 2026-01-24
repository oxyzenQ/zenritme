use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn beep(times: u32) {
    let times = times.max(1);
    let mut stdout = io::stdout();
    for i in 0..times {
        let played = play_pipewire_bell();
        if !played {
            let _ = stdout.write_all(b"\x07");
            let _ = stdout.flush();
        }
        if std::env::var("ZENRITME_VISUAL_BELL").is_ok() {
            visual_bell(&mut stdout);
        }
        if i + 1 < times {
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
    }
}

pub fn sound_test() {
    println!("zenritme sound test");
    println!("- Terminal bell: sends \\x07 (often muted in Alacritty)");
    println!("- PipeWire: tries `pw-play` (set ZENRITME_SOUND_FILE to override)");
    println!("- Visual bell: set ZENRITME_VISUAL_BELL=1");
    println!("Example: ZENRITME_VISUAL_BELL=1 zenritme --sound-test");
    println!("Example: ZENRITME_SOUND_FILE=/usr/share/sounds/freedesktop/stereo/bell.oga zenritme --sound-test");
    match pick_sound_file() {
        Some(f) => println!("PipeWire sound file: {}", f),
        None => println!("PipeWire sound file: (none found; using terminal bell fallback)"),
    }
    beep(1);
    std::thread::sleep(std::time::Duration::from_millis(250));
    beep(2);
    std::thread::sleep(std::time::Duration::from_millis(250));
    beep(3);
}

fn pick_sound_file() -> Option<String> {
    let candidates = [
        std::env::var("ZENRITME_SOUND_FILE").ok(),
        Some("/usr/share/sounds/freedesktop/stereo/window-attention.oga".to_string()),
        Some("/usr/share/sounds/freedesktop/stereo/complete.oga".to_string()),
        Some("/usr/share/sounds/freedesktop/stereo/bell.oga".to_string()),
        Some("/usr/share/sounds/freedesktop/stereo/bell.wav".to_string()),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|p| std::path::Path::new(p).exists())
}

fn play_pipewire_bell() -> bool {
    let Some(file) = pick_sound_file() else {
        return false;
    };

    Command::new("pw-play")
        .arg(file)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn visual_bell(stdout: &mut io::Stdout) {
    let _ = stdout.write_all(b"\x1b[?5h");
    let _ = stdout.flush();
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = stdout.write_all(b"\x1b[?5l");
    let _ = stdout.flush();
}
