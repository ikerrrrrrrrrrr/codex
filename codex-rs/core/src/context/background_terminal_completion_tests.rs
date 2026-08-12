use std::time::Duration;

use codex_utils_output_truncation::approx_token_count;
use pretty_assertions::assert_eq;

use super::*;

fn completion(command: Vec<String>, cwd: &str, output: String) -> BackgroundTerminalCompletion {
    BackgroundTerminalCompletion::new(
        42,
        command,
        PathUri::parse(cwd).expect("valid cwd URI"),
        0,
        Duration::from_secs(1),
        output,
    )
}

#[test]
fn output_keeps_last_hundred_lines_when_that_range_is_larger() {
    let output = (0..150)
        .map(|line| format!("line-{line:03}-{}", "x".repeat(48)))
        .collect::<Vec<_>>()
        .join("\n");
    let completion = completion(vec!["proof".to_string()], "file:///tmp", output);

    assert_eq!(completion.output.lines().count(), 101);
    assert!(completion.output.starts_with(TAIL_TRUNCATION_MARKER));
    assert!(completion.output.contains("line-050-"));
    assert!(
        completion
            .output
            .ends_with(&format!("line-149-{}", "x".repeat(48)))
    );
}

#[test]
fn output_keeps_minimum_token_tail_when_it_is_larger_than_hundred_lines() {
    let output = (0..2_000)
        .map(|line| format!("line-{line:04}"))
        .collect::<Vec<_>>()
        .join("\n");
    let completion = completion(vec!["proof".to_string()], "file:///tmp", output);

    assert!(completion.output.lines().count() > OUTPUT_TAIL_LINES + 1);
    assert!(approx_token_count(&completion.output) >= OUTPUT_TAIL_MIN_TOKENS);
    assert!(completion.output.ends_with("line-1999"));
}

#[test]
fn output_is_capped_even_for_one_large_line() {
    let completion = completion(
        vec!["proof".to_string()],
        "file:///tmp",
        format!(
            "old{}new",
            "x".repeat(approx_bytes_for_tokens(OUTPUT_TAIL_MAX_TOKENS) * 2)
        ),
    );

    assert!(approx_token_count(&completion.output) <= OUTPUT_TAIL_MAX_TOKENS);
    assert!(completion.output.starts_with(TAIL_TRUNCATION_MARKER));
    assert!(completion.output.ends_with("new"));
}

#[test]
fn command_cwd_and_complete_fragment_are_bounded() {
    let completion = completion(
        vec![format!(
            "run-{}",
            "\\\"".repeat(approx_bytes_for_tokens(COMMAND_MAX_TOKENS))
        )],
        &format!(
            "file:///{}",
            "x".repeat(approx_bytes_for_tokens(CWD_MAX_TOKENS) * 2)
        ),
        "\n".repeat(approx_bytes_for_tokens(OUTPUT_TAIL_MAX_TOKENS)),
    );

    assert!(approx_token_count(&completion.command) <= COMMAND_MAX_TOKENS);
    assert!(approx_token_count(&completion.render()) <= FRAGMENT_MAX_TOKENS);
}
