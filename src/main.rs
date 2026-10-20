// SPDX-License-Identifier: Apache-2.0
mod cli;
mod engine;
mod error;
mod model;
mod runtime;
mod store;

fn main() -> std::process::ExitCode {
    cli::main()
}
