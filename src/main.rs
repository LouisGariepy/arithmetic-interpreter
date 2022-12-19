use input::prompt;
use owo_colors::OwoColorize;
use parser::{ParseTree, Parser, ParserError};
use runtime::evaluate;
use tokenizer::Span;

// Module declarations
mod input;
mod parser;
mod runtime;
mod tokenizer;

fn main() {
    loop {
        // Get the user input and parse it
        let input = prompt();
        let parsed = Parser::new(&input).parse();

        match parsed {
            Ok(parse_tree) => match parse_tree {
                // Evaluate and print the result
                ParseTree::Expression(expr) => {
                    let evaluated = evaluate(expr);
                    println!("{evaluated}");
                }
                // Quit the calculator
                ParseTree::Quit => break,
                // Go to next prompt
                ParseTree::Empty => continue,
            },
            Err(e) => {
                // Display the error and go to next prompt
                println!("{}", format_error(e, &input));
                continue;
            }
        }
    }
}

/// Gets the string the the span points to.
/// If the span is `None`, returns `"<EOL>"` (end of line) instead
fn spanned_value(input: &str, span: Option<Span>) -> &str {
    span.map(|span| &input[span]).unwrap_or("<EOL>")
}

/// Unwraps an optional span. If the option was `None`,
/// creates a span of the last character of the input instead.
fn unwrap_span(input: &str, span: Option<Span>) -> Span {
    span.unwrap_or(Span {
        start: input.len() - 1,
        end: input.len(),
    })
}

fn format_error(error: ParserError, input: &str) -> String {
    // Create the error message and get the source span
    let (msg, span) = match error {
        ParserError::UnrecognizedSpecial(span) => (
            format!("expected `?quit`, found `{}`", spanned_value(input, span)),
            unwrap_span(input, span),
        ),
        ParserError::ExpectedBinaryOp(span) => (
            format!(
                "expected one of `+`, `-`, `*`, `/`, found `{}`",
                spanned_value(input, span)
            ),
            unwrap_span(input, span),
        ),
        ParserError::ExpectedExprStart(span) => (
            format!(
                "expected one of `-`, `(`, or a number, found `{}`",
                spanned_value(input, span)
            ),
            unwrap_span(input, span),
        ),
        ParserError::UnclosedParenthesis(span) => (
            format!("expected `)`, found `{}`", spanned_value(input, span)),
            unwrap_span(input, span),
        ),
    };

    // Format the first line, explaining the reason for the error
    let explanation_line = format!("{}: {}", "error".red(), msg);

    // Format the line representing the source input
    let src_line = format!("      {input}");

    // Format the underline representing where the error occured in the source
    let padding = " ".repeat(input[0..span.start].chars().count());
    let underline = "^".repeat(input[span].chars().count());
    let src_underline = format!("      {}{}", padding, underline.red().bold());

    // Format the whole error
    format!(
        "\
{}
{}{}",
        explanation_line.bold(),
        src_line.white(),
        src_underline
    )
}
