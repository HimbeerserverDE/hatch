use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::Duration;

const MAX_WIDTH: usize = 1000;
const MAX_HEIGHT: usize = 1000;

fn main() {
    let (width, height) =
        term_size::dimensions().expect("unable to get terminal size, is this a terminal?");

    let height = height - 1;

    let center_x = width / 2;
    let center_y = height / 2;

    let mut colors = [[false; MAX_HEIGHT]; MAX_WIDTH];

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            colors[x][y] = (rand::random::<f32>() * 20.0).floor() == 19.0;
        }
    }

    let mut special = [[false; MAX_HEIGHT]; MAX_WIDTH];

    let margin_y = height / 4;

    for y in 0..height {
        #[allow(clippy::needless_range_loop)]
        for x in 0..width {
            if y > margin_y && y <= height - margin_y {
                let offset = if y <= center_y {
                    y - margin_y
                } else {
                    (height - margin_y) - y + 1
                };

                if x == center_x - 3 - offset {
                    special[x][y] = true;
                }
            }
        }
    }

    let mut w = BufWriter::with_capacity(MAX_WIDTH * MAX_HEIGHT * 10, io::stdout());

    loop {
        write!(w, "\x1b[H").expect("can't write to stdout buffer");

        for y in 0..height {
            #[allow(clippy::needless_range_loop)]
            for x in 0..width {
                let color = if colors[x][y] {
                    "\x1b[48;5;214m"
                } else {
                    "\x1b[48;5;208m"
                };

                let ch = if special[x][y] {
                    "-"
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
