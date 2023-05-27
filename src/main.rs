use std::env;
use std::fmt::Write as _;
use std::io::{self, BufWriter, Write as _};
use std::thread;
use std::time::{Duration, Instant};

use pam::Authenticator;

const MAX_WIDTH: usize = 1000;
const MAX_HEIGHT: usize = 1000;

fn main() {
    let (width, height) =
        term_size::dimensions().expect("unable to get terminal size, is this a terminal?");

    let width = width as isize;
    let height = (height as isize) - 1;

    let center_x = width / 2;
    let center_y = height / 2;

    let margin_x = width / 4;
    let margin_y = height / 4;

    let odd_height = height % 2 != 0;
    let max_f = center_x - 9 - (center_y - margin_y) - (margin_x / 2);
    let v = (200000 / max_f) as u64;

    let mut colors = [[false; MAX_HEIGHT]; MAX_WIDTH];

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            let xi = x as usize;
            let yi = y as usize;

            colors[xi][yi] = (rand::random::<f32>() * 20.0).floor() == 19.0;
        }
    }

    let mut special = [[false; MAX_HEIGHT]; MAX_WIDTH];

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            let xi = x as usize;
            let yi = y as usize;

            if y > margin_y && y < height - margin_y - if odd_height { 1 } else { 0 } {
                let offset = if y <= center_y {
                    y - margin_y
                } else if odd_height {
                    (height - margin_y) - y - 1
                } else {
                    (height - margin_y) - y
                };

                if x == center_x - 3 - offset
                    || x == center_x - 9 - offset
                    || x == center_x + 3 + offset
                    || x == center_x + 9 + offset
                {
                    special[xi][yi] = true;
                }
            }
        }
    }

    let mut locked_frame = String::new();

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            let xi = x as usize;
            let yi = y as usize;

            let color = if special[xi][yi] {
                "\x1b[48;5;15m"
            } else if colors[xi][yi] {
                "\x1b[48;5;220m"
            } else {
                "\x1b[48;5;226m"
            };

            let ch = if !special[xi][yi] && colors[xi][yi] {
                "*"
            } else {
                " "
            };

            write!(locked_frame, "\x1b[0m{}{}\x1b[0m", color, ch)
                .expect("can't write to stdout buffer");
        }

        writeln!(locked_frame).expect("can't write to stdout buffer");
    }

    let mut unlocking_frames = Vec::new();

    for f in 0..max_f {
        let mut frame = String::new();

        for y in 0..height {
            #[allow(clippy::needless_range_loop)]
            for x in 0..width {
                let off = if x <= center_x { f } else { -f };

                let xi = x as usize;
                let yi = y as usize;

                let xi_off = if (x <= center_x && x + off <= center_x)
                    || (x > center_x && x + off > center_x)
                {
                    (x + off) as usize
                } else {
                    0
                };

                let rect_part_horizontal = (y == margin_y - 5
                    || y == height - margin_y - if odd_height { 1 } else { 0 } + 5)
                    && (center_x - x).abs() <= off.abs();

                let rect_part_vertical = (y >= margin_y - 5
                    && y <= height - margin_y - if odd_height { 1 } else { 0 } + 5)
                    && (center_x - x).abs() == off.abs();

                let rect_part = rect_part_horizontal || rect_part_vertical;

                let color = if special[xi_off][yi] || rect_part {
                    "\x1b[48;5;15m"
                } else if colors[xi][yi] {
                    "\x1b[48;5;76m"
                } else {
                    "\x1b[48;5;82m"
                };

                let ch = if !special[xi][yi] && colors[xi][yi] {
                    "*"
                } else {
                    " "
                };

                write!(frame, "\x1b[0m{}{}\x1b[0m", color, ch)
                    .expect("can't write to stdout buffer");
            }

            writeln!(frame).expect("can't write to stdout buffer");
        }

        unlocking_frames.push(frame);
    }

    let mut w = BufWriter::with_capacity(MAX_WIDTH * MAX_HEIGHT * 10, io::stdout());

    write!(w, "\x1b[H").expect("can't write to stdout buffer");
    write!(w, "{}", locked_frame).expect("can't write to stdout buffer");
    w.flush().expect("can't flush frame to stdout");

    let user = env::var("USER").expect("environment variable USER is unset");

    let mut passwd = String::new();
    io::stdin()
        .read_line(&mut passwd)
        .expect("can't read user input");

    // Remove trailing newline.
    passwd.pop();

    let mut auth =
        Authenticator::with_password("system-auth").expect("can't initialize PAM conversation");

    auth.get_handler().set_credentials(user, passwd);
    auth.authenticate().expect("authentication failed");
    auth.open_session().expect("can't open PAM session");

    for f in unlocking_frames.iter() {
        let start = Instant::now();

        write!(w, "\x1b[H").expect("can't write to stdout buffer");
        write!(w, "{}", f).expect("can't write to stdout buffer");
        w.flush().expect("can't flush frame to stdout");

        let dur = Instant::now().duration_since(start);
        thread::sleep(Duration::from_micros(v).saturating_sub(dur));
    }
}
