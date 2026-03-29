use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = mgfy::main() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
