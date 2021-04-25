use std::io::{stdin,stdout,Write};

fn main() {
    let mut finished = false;
    while !finished {
        println!("What would you like to do?");
        println!("1. Get my messages!");
        println!("2. Write a message!");
        println!("3. Quit!");

        let input = get_input(String::from("> "));
        match &input[..] {
            "1" => println!("Get selected"),
            "2" => println!("Read selected"),
            "3" => finished = true,
            _ => println!("Invalid selection")
        }
    }
}

fn get_input(prompt: String) -> String {
    print!("{}", prompt);
    stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read user inputted line");
    input.trim().to_string()
}
