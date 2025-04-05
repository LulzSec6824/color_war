use std::io;

fn main() -> io::Result<()> {
    println!("Enter a number:");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let number: f64 = input.trim().parse()
        .expect("Please enter a valid number");

    Ok(())
}