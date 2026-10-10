//! Logging setup for the `descar` binary.
//!
//! Only the binary installs a subscriber. Library crates emit `tracing` events and never
//! configure where the events go.
//!
//! Contract:
//! - log events are plain text lines written to stderr;
//! - stdout is never used for log events;
//! - the flags `-v`, `-vv`, `-vvv` and `-q` select the maximum level (see [`level_filter`]).

use descar_cli::cli::LoggingArgs;
use std::error::Error;
use std::io;
use tracing::level_filters::LevelFilter;

/// Maximum level used when the user passes neither `-v` nor `-q`.
///
/// Assumption to confirm: the requested mapping starts at `-v` = `WARN`, so the level below
/// `WARN` that still reports failures is `ERROR`.
pub const DEFAULT_LEVEL: LevelFilter = LevelFilter::ERROR;

/// Maps the CLI flags to the maximum level of log events.
///
/// | Flags              | Level                 |
/// |--------------------|-----------------------|
/// | `-q` (any `-v`)    | off                   |
/// | none               | [`DEFAULT_LEVEL`]     |
/// | `-v`               | `WARN`                |
/// | `-vv`              | `DEBUG`               |
/// | `-vvv` or more     | `TRACE`               |
///
/// `-q` has precedence over `-v`.
#[must_use]
pub const fn level_filter(verbose: u8, quiet: bool) -> LevelFilter {
    if quiet {
        return LevelFilter::OFF;
    }

    match verbose {
        0 => DEFAULT_LEVEL,
        1 => LevelFilter::WARN,
        2 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

/// Installs the global subscriber for the selected level.
///
/// When the level is off, no subscriber is installed, so every event has no cost.
/// Colors are used only when stderr is a terminal and the user did not disable colors.
///
/// # Errors
///
/// Returns an error when a global subscriber is already installed.
pub fn init(args: &LoggingArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let level = level_filter(args.verbose, args.quiet);
    if level == LevelFilter::OFF {
        return Ok(());
    }

    let with_location = level == LevelFilter::TRACE;

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(io::stderr)
        .with_ansi(console::colors_enabled_stderr())
        .with_target(true)
        .with_file(with_location)
        .with_line_number(with_location)
        .without_time()
        .try_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_verbosity_to_level() {
        let cases = [
            (0, LevelFilter::ERROR),
            (1, LevelFilter::WARN),
            (2, LevelFilter::DEBUG),
            (3, LevelFilter::TRACE),
            (4, LevelFilter::TRACE),
            (u8::MAX, LevelFilter::TRACE),
        ];

        for (verbose, expected) in cases {
            assert_eq!(level_filter(verbose, false), expected, "verbose = {verbose}");
        }
    }

    #[test]
    fn quiet_turns_logging_off_for_every_verbosity() {
        for verbose in [0, 1, 2, 3, u8::MAX] {
            assert_eq!(level_filter(verbose, true), LevelFilter::OFF, "verbose = {verbose}");
        }
    }

    #[test]
    fn default_level_is_the_level_selected_without_flags() {
        assert_eq!(level_filter(0, false), DEFAULT_LEVEL);
    }

    #[test]
    fn quiet_installs_no_subscriber_and_succeeds_every_time() {
        let args = LoggingArgs { verbose: 3, quiet: true };

        assert!(init(&args).is_ok());
        assert!(init(&args).is_ok());
    }

    #[test]
    fn second_installation_of_a_subscriber_is_reported_as_error() {
        let args = LoggingArgs { verbose: 1, quiet: false };

        assert!(init(&args).is_ok());
        assert!(init(&args).is_err());
    }
}
