fn main() {
    match mgfy::main() {
        Err(e) => eprintln!("Error: {}", e),
        _ => {}
    }
}
