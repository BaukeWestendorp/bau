use crate::source::{CodeRange, Source};

use colored::Colorize;

pub fn print_error(source: &Source, range: Option<&CodeRange>, message: &str) {
    // Show error message
    eprintln!("{}: {}", "error".bright_red(), message);

    // If there is no range associated with the error, don't show the source code
    if range.is_none() {
        return;
    }
    let range = range.unwrap();

    let max_line_number_len = source.lines().len().to_string().len();

    // Show the line(s) of code that caused the error
    let lines = source.text()[range.span.start..range.span.end].lines();
    let line_count = lines.clone().count();
    let mut cursor = 0;
    for (line_number, line) in lines.clone().enumerate() {
        if line_number == 0 {
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line,
                range.coords.column,
                line.len(),
            )
        } else if line_number == line_count - 1 {
            let len = range.span.len() - cursor;
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line + line_number,
                0,
                len,
            )
        } else {
            print_source_line(
                source,
                max_line_number_len,
                range.coords.line + line_number,
                0,
                line.len(),
            )
        }
        cursor += line.len() + 1;
    }

    // Print a underline to show where the error occurred
    let underline_length = match line_count {
        1 => range.span.len(),
        _ => lines.map(|line| line.len()).max().unwrap(),
    };
    print_line_gutter(max_line_number_len, None);
    eprintln!(
        "{}",
        format!(
            "{}{} {}",
            " ".repeat(range.coords.column),
            "^".repeat(usize::max(1, underline_length)),
            message,
        )
        .bright_red()
    );
}

fn print_line_gutter(max_line_number_len: usize, line_number: Option<usize>) {
    match line_number {
        Some(line_number) => {
            let padding = max_line_number_len - line_number.to_string().len();
            eprint!(" {}{}", " ".repeat(padding), line_number);
        }
        None => {
            eprint!(" {}", " ".repeat(max_line_number_len));
        }
    }
    eprint!(" {} ", "|".bright_red());
}

fn print_source_line(
    source: &Source,
    max_line_number_len: usize,
    line_number: usize,
    column: usize,
    len: usize,
) {
    let line_number = match line_number >= source.lines().len() {
        true => source.lines().len() - 1,
        false => line_number,
    };
    let (start, end) = source.lines()[line_number].split_at(column);
    let (mid_error, end) = end.split_at(len);
    print_line_gutter(max_line_number_len, Some(line_number + 1));
    eprintln!("{}{}{}", start.white(), mid_error.bright_red(), end.white());
}
