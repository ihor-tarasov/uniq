use std::io::Write;

use uniq::library;

fn main() {
    let natives = library::load();
    let mut code = String::new();
    let mut line = String::new();
    loop {
        code.clear();
        print!("-> ");
        loop {
            line.clear();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut line).unwrap();
            code.push_str(line.as_str());
            
            if uniq::utils::eval_eof(code.as_str(), &natives) {
                print!("-| ");
            } else {
                break;
            }
        }
    }
}
