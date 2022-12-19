//! A simple module for user inputs.
//! Nothing too crazy going on here.

use std::io::{stdin, Write};

use owo_colors::OwoColorize;

/// Draws a nice little prompt indicator indicating to the user
/// that the calculator is ready to take inputs.
fn prompt_indicator() {
    // Notice how we use `print!` and not `println!` here.
    // This is because we want the user input to be on the
    // same line as the prompt indicator.
    print!("{}", "calc❯ ".green().bold());
    std::io::stdout()
        .flush()
        .expect("failed to write to standard output");
}

/// Simple utility that reads user inputs.
fn read_user_input() -> String {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("failed to read from standard input");

    input
}

/// Draws the prompt indicator and reads the user input.
pub fn prompt() -> String {
    prompt_indicator();
    read_user_input()
}
