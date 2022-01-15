use std::env::args;
use std::io::{Read, stdin};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};

fn main() {
    // A number can be passed to change the rate of speech.
    let rate_arg = args()
        .find(|arg| f32::from_str(arg).is_ok())
        .map(|arg| f32::from_str(&arg).unwrap());

    let mut tts = tts::Tts::default().expect("Cannot open Tts engine.");

    if let Some(speed) = rate_arg {
        if tts.set_rate(speed).is_err() {
            println!("Please enter a rate within range. The max rate is {}.0", tts.max_rate());
            return;
        }
    }

    let pair = Arc::new((Mutex::new(0), Condvar::new()));
    let pair2 = Arc::clone(&pair);

    // Decrement counter on each speech end to avoid leaving early
    tts.on_utterance_end(Some(Box::new(move |_| {
        let (count, cond) = &*pair;
        let mut count = count.lock().unwrap();

        // Exit program when all lines are said
        *count -= 1;
        if *count == 0 {
            cond.notify_one()
        }
    })))
    .unwrap();

    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut line = String::new();
    let (count, cond) = &*pair2;
    let mut count = count.lock().unwrap();

    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }

        // This spawns in a new thread, so we must wait for it to finish before exiting.
        *count += 1;
        tts.speak(&line, false).expect("Failed to speak");

        line.clear();
    }

    cond.wait(count);
}
