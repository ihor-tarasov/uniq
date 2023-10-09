use std::io::Write;

fn main() {
    let mut line = String::new();
    loop {
        line.clear();
        print!("-> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        uniq::utils::eval(line.as_str());
    }
}
