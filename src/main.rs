mod duration;

use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use duration::{format_clock, parse_duration};

// Width of the branch in characters.
const BRANCH_WIDTH: usize = 50;
// How often we redraw the screen.
const TICK: Duration = Duration::from_millis(250);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print_help();
        return;
    }

    let total = match parse_duration(&args.join("")) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("sloth: {e}");
            eprintln!("try: sloth 25m, sloth 40s, sloth 1.5h, sloth 1h30m");
            std::process::exit(1);
        }
    };

    run(total);
}

fn print_help() {
    println!("sloth - a lazy timer 🦥");
    println!();
    println!("usage: sloth <time>");
    println!();
    println!("examples:");
    println!("  sloth 25m     25 minutes");
    println!("  sloth 40s     40 seconds");
    println!("  sloth 5.5h    5 and a half hours");
    println!("  sloth 1h30m   units can be mixed");
    println!("  sloth 10      a plain number means minutes");
}

fn run(total: Duration) {
    let start = Instant::now();
    let mut first_frame = true;

    loop {
        let elapsed = start.elapsed().min(total);
        let progress = elapsed.as_secs_f64() / total.as_secs_f64();
        let done = elapsed >= total;

        let frame = draw_frame(progress, total - elapsed, elapsed, done);
        show(&frame, first_frame);
        first_frame = false;

        if done {
            break;
        }
        thread::sleep(TICK);
    }

    // Ring the terminal bell.
    print!("\x07");
    let _ = io::stdout().flush();
}

/// Build the lines for one frame of the animation.
fn draw_frame(progress: f64, left: Duration, elapsed: Duration, done: bool) -> Vec<String> {
    let face = if done {
        "(^o^)"
    } else if blink(elapsed) {
        "(-.-)"
    } else {
        "(o.o)"
    };
    let face_width = 5;

    // The sloth hangs under the branch. Its claws grip the branch above its head.
    let max_pos = BRANCH_WIDTH - face_width;
    let pos = (progress * max_pos as f64).round() as usize;

    let branch: String = (0..BRANCH_WIDTH)
        .map(|i| if i == pos + 1 || i == pos + 3 { '┳' } else { '━' })
        .collect();
    let body = format!("{}{}", " ".repeat(pos), face);

    let percent = (progress * 100.0).floor() as u32;
    let status = if done {
        "Time is up! Have a rest. 🌿".to_string()
    } else {
        // Round up, so a 3s timer shows 3, 2, 1 and not 2, 1, 0.
        let left = Duration::from_secs(left.as_secs_f64().ceil() as u64);
        format!("{} left  ·  {percent}%", format_clock(left))
    };

    vec![format!("🌳{branch}🌳"), format!("  {body}"), String::new(), format!("  {status}")]
}

/// The sloth keeps its eyes closed most of the time and opens them for a moment now and then.
fn blink(elapsed: Duration) -> bool {
    elapsed.as_millis() % 6000 >= 750
}

/// Print a frame, drawing over the previous one.
fn show(lines: &[String], first: bool) {
    let mut out = io::stdout().lock();
    if !first {
        // Move the cursor up to the start of the previous frame.
        let _ = write!(out, "\x1b[{}A", lines.len());
    }
    for line in lines {
        // "\r\x1b[2K" clears the line before we draw it.
        let _ = writeln!(out, "\r\x1b[2K{line}");
    }
    let _ = out.flush();
}
