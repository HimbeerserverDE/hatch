use std::io::{self, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
    let max_f = center_x - 9 - (center_y - margin_y) - margin_x;
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

    let unlock_frame = Arc::new(Mutex::new(None));
    let mut w = BufWriter::with_capacity(MAX_WIDTH * MAX_HEIGHT * 10, io::stdout());

    let unlock_frame2 = unlock_frame.clone();
    thread::spawn(move || {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("can't read user input");

        *unlock_frame2.lock().unwrap() = Some(0isize);
    });

    loop {
        let mut unlock_frame = unlock_frame.lock().unwrap();

        let start = Instant::now();

        write!(w, "\x1b[H").expect("can't write to stdout buffer");

        for y in 0..height {
            #[allow(clippy::needless_range_loop)]
            for x in 0..width {
                let f = unlock_frame.unwrap_or(0);
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
                    if unlock_frame.is_some() {
                        "\x1b[48;5;76m"
                    } else {
                        "\x1b[48;5;220m"
                    }
                } else if unlock_frame.is_some() {
                    "\x1b[48;5;82m"
                } else {
                    "\x1b[48;5;226m"
                };

                let ch = if special[xi_off][yi] {
                    " "
                } else if (rand::random::<f32>() * 15.0).floor() == 14.0 {
                    "*"
                } else {
                    " "
                };

                write!(w, "\x1b[0m{}{}\x1b[0m", color, ch).expect("can't write to stdout buffer");
            }

            writeln!(w).expect("can't write to stdout buffer");
        }

        w.flush().expect("can't flush frame to stdout");

        if let Some(f) = *unlock_frame {
            if f >= max_f {
                return;
            }

            *unlock_frame = Some(f + 1);
        }

        drop(unlock_frame);

        let dur = Instant::now().duration_since(start);
        thread::sleep(Duration::from_micros(v).saturating_sub(dur));
    }
}
