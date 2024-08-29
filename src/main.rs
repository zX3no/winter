use std::io::{stdin, stdout};
use winter::*;

fn main() {
    let mut winter = Winter::new();

    //Prevents panic messages from being hidden.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        uninit(&mut stdout(), &mut stdin());
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    loop {
        {
            let buf = winter.buffer();
            let area = buf.area;
            block().draw(area, buf);
        }

        if let Some((event, state)) = winter.poll() {
            // println!("{}", event);
            if let Event::Resize(width, height) = event {
                //TODO: Cleanup.
                winter.viewport = Rect::new(0, 0, width, height);
                winter.resize();
            }

            if event == Event::Char('c') && state.control() || event == Event::Escape {
                break;
            }
        }

        winter.draw();
    }
}
