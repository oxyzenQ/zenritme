use std::io::{self, Write};

pub fn beep(times: u32) {
    let times = times.max(1);
    let mut stdout = io::stdout();
    for i in 0..times {
        let _ = stdout.write_all(b"\x07");
        let _ = stdout.flush();
        if std::env::var("ZENTIME_VISUAL_BELL").is_ok() {
            visual_bell(&mut stdout);
        }
        if i + 1 < times {
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
    }
}

pub fn sound_test() {
    println!("zentime sound test");
    println!("If you don't hear beeps, your terminal bell may be disabled.");
    println!("Tip: try ZENTIME_VISUAL_BELL=1 zentime --sound-test");
    beep(1);
    std::thread::sleep(std::time::Duration::from_millis(250));
    beep(2);
    std::thread::sleep(std::time::Duration::from_millis(250));
    beep(3);
}

fn visual_bell(stdout: &mut io::Stdout) {
    let _ = stdout.write_all(b"\x1b[?5h");
    let _ = stdout.flush();
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = stdout.write_all(b"\x1b[?5l");
    let _ = stdout.flush();
}
