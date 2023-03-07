fn main() -> anyhow::Result<()> {
    let image_path = env::args_os().nth(1).context("no first arg")?;
    let mut image = image::open(image_path).context("failed to load image")?;

    let max_width = 512;
    let max_height = 64;

    if image.width() > max_width || image.height() > max_height {
        let new_width = image.width().min(max_width);
        let new_height = image.height().min(max_height);
        image = imageops::resize(&image, new_width, new_height, FILTER).into();
    }

    let image = imageops::grayscale(&image);

    let mut stdout = io::stdout().lock();

    terminal::enable_raw_mode().context("failed to enter raw mode")?;
    let _raw_mode_guard = Guard::new((), |()| drop(terminal::disable_raw_mode()));

    stdout.execute(terminal::EnterAlternateScreen)?;
    let mut alternate_screen_guard = Guard::new(&mut stdout, |stdout| {
        drop(stdout.execute(terminal::LeaveAlternateScreen))
    });
    let stdout = &mut **alternate_screen_guard.state;

    let mut width = 80 * 2;
    let mut height = 48 * 4;
    loop {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::MoveTo(0, 0))?;

        let mut new_image = imageops::resize(&image, width, height, FILTER);
        imageops::dither(&mut new_image, &BiLevel);
        let rows = height / 4;
        let cols = width / 2;
        let mut row_bytes = vec![0; braille::Pattern::UTF8_LEN * (cols as usize) + 2];
        row_bytes[braille::Pattern::UTF8_LEN * (cols as usize)] = b'\r';
        row_bytes[braille::Pattern::UTF8_LEN * (cols as usize) + 1] = b'\n';
        for row in 0..rows {
            for col in 0..cols {
                let mut pattern = braille::Pattern::EMPTY;
                for x in 0..2 {
                    for y in 0..4 {
                        let pixel = *new_image.get_pixel(col * 2 + x, row * 4 + y);
                        pattern.set(x, y, pixel.0[0] != 0);
                    }
                }
                let slice = &mut row_bytes[col as usize * braille::Pattern::UTF8_LEN..]
                    [..braille::Pattern::UTF8_LEN];
                pattern.encode_utf8(slice.try_into().unwrap());
            }
            stdout.write_all(&row_bytes)?;
        }

        writeln!(stdout, "hi :3      ({width}x{height})\r")?;

        loop {
            let Event::Key(e) = event::read()? else { continue };
            match e.code {
                event::KeyCode::Up => {
                    height = (height - 4).max(4);
                }
                event::KeyCode::Down => {
                    height += 4;
                }
                event::KeyCode::Left => width = (width - 4).max(4),
                event::KeyCode::Right => width += 4,
                event::KeyCode::Esc => return Ok(()),
                _ => continue,
            }
            break;
        }
    }
}

const FILTER: imageops::FilterType = imageops::FilterType::Lanczos3;

use guard::Guard;
mod guard {
    pub struct Guard<S, F: FnOnce(S)> {
        pub state: ManuallyDrop<S>,
        pub function: ManuallyDrop<F>,
    }

    impl<S, F: FnOnce(S)> Guard<S, F> {
        pub fn new(state: S, function: F) -> Self {
            Self {
                state: ManuallyDrop::new(state),
                function: ManuallyDrop::new(function),
            }
        }
    }

    impl<S, F: FnOnce(S)> Drop for Guard<S, F> {
        fn drop(&mut self) {
            let state = unsafe { ManuallyDrop::take(&mut self.state) };
            let function = unsafe { ManuallyDrop::take(&mut self.function) };
            function(state)
        }
    }

    use std::mem::ManuallyDrop;
}

mod braille;

use anyhow::Context;
use crossterm::cursor;
use crossterm::event;
use crossterm::event::Event;
use crossterm::terminal;
use crossterm::ExecutableCommand as _;
use crossterm::QueueableCommand as _;
use image::imageops;
use image::imageops::BiLevel;
use std::env;
use std::io;
use std::io::Write;
