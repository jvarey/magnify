fn main() {
    if let Err(e) = mgfy::main() {
        eprintln!("Error: {}", e);
    }
}
