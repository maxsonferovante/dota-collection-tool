//! CLI surface smoke test: every subcommand reports help.

use clap::Parser;

use dct_cli::cli::Cli;

#[test]
fn every_subcommand_has_help() {
    for sub in ["install", "token", "up", "down", "status", "logs", "export"] {
        let err = Cli::try_parse_from(["dct", sub, "--help"]).unwrap_err();
        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::DisplayHelp,
            "missing --help for {sub}"
        );
    }
}
