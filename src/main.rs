use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::Duration;

const MAX_WIDTH: usize = 1000;
const MAX_HEIGHT: usize = 1000;

fn main() {
    let (width, height) =
        term_size::dimensions().expect("unable to get terminal size, is this a terminal?");

    let width = width as isize;
    let height = (height as isize) - 1;

    let center_x = width / 2;
    let center_y = height / 2;

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

    let margin_y = height / 4;

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            let xi = x as usize;
            let yi = y as usize;

            let odd_height = height % 2 != 0;

            if y > margin_y && y < height - margin_y - if odd_height { 1 } else { 0 } {
                let offset = if y < center_y {
                    y - margin_y
                } else if y > center_y {
                    if odd_height {
                        (height - margin_y) - y - 1
                    } else {
                        (height - margin_y) - y
                    }
                } else {
                    y - margin_y
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

    let mut unlock_frame = Some(0isize);
    let mut w = BufWriter::with_capacity(MAX_WIDTH * MAX_HEIGHT * 10, io::stdout());

    loop {
        write!(w, "\x1b[H").expect("can't write to stdout buffer");

        for y in 0..height {
            #[allow(clippy::needless_range_loop)]
            for x in 0..width {
                let xi = x as usize;
                let yi = y as usize;

                let color = if special[xi][yi] {
                    "\x1b[48;5;15m"
                } else if colors[xi][yi] {
                    if unlock_frame.is_some() {
                        "\x1b[48;5;76m"
                    } else {
                        "\x1b[48;5;220m"
                    }
                } else {
                    if unlock_frame.is_some() {
                        "\x1b[48;5;82m"
                    } else {
                        "\x1b[48;5;226m"
                    }
                };

                let ch = if special[xi][yi] {
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

        thread::sleep(Duration::from_millis(150));
    }
}
