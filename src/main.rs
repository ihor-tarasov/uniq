use std::io::{BufReader, Write};

use uniq::{library, natives::Natives, utils};

fn repl(natives: &Natives) {
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

            if uniq::utils::eval_eof(code.as_str(), natives) {
                print!("-| ");
            } else {
                break;
            }
        }
    }
}

fn run_file(path: &str, natives: &Natives) {
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut read = BufReader::new(file);
            match utils::compile(path, &mut read, natives) {
                Ok(chunk) => match utils::run(&[path], &mut read, &chunk, natives) {
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("{error}"),
                },
                Err(error) => eprintln!("{error}"),
            }
        }
        Err(error) => eprintln!("Unable to open file, IOError: {error}"),
    }
}

fn main() {
    let natives = library::load();
    let mut run_repl = true;
    for arg in std::env::args().skip(1) {
        run_repl = false;
        run_file(arg.as_str(), &natives);
    }
    if run_repl {
        repl(&natives);
    }
}
