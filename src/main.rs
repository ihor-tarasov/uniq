use std::io::Write;

use uniq::{
    compiler::{Chunk, Compiler},
    library,
    natives::Natives,
    utils,
    vm::{State, Value},
    SliceRead,
};

fn run_repl(natives: &Natives) {
    let mut sources = Vec::new();
    let mut compiler = Compiler::new(0, natives);
    let mut stack = utils::new_stack::<256>();
    let mut state = State::new(&mut stack, natives);
    loop {
        let mut code = String::new();
        print!("-> ");
        loop {
            let mut line = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut line).unwrap();
            code.push_str(line.as_str());

            let source_id = sources.len();
            let mut read = SliceRead::from(code.as_str());
            match utils::compile_repl(&mut compiler, source_id, &mut read) {
                Ok(_) => {
                    sources.push(code);
                    match utils::run_repl(&mut state, sources.as_slice(), compiler.chunk()) {
                        Ok(_) => (),
                        Err(error) => {
                            eprintln!("{error}");
                        }
                    }
                    break;
                }
                Err(error) => {
                    if let Some(error) = error {
                        eprintln!("{error}");
                        break;
                    } else {
                        print!("-| ");
                    }
                }
            }
        }
    }
}

fn compile_files(paths: &[String], natives: &Natives) -> Result<Chunk, String> {
    let mut compiler = Compiler::new(0, natives);

    for (source_id, path) in paths.iter().enumerate() {
        utils::compile_file(&mut compiler, source_id, path.as_str())?;
    }

    Ok(compiler.into_chunk())
}

fn run_chunk(paths: &[String], chunk: Chunk, natives: &Natives) {
    let mut stack = vec![Value::Void; 1048576];
    let mut state = State::new(&mut stack, natives);
    match utils::run_file(&mut state, paths, &chunk) {
        Ok(_) => (),
        Err(error) => eprintln!("{error}"),
    }
}

fn run_files(paths: Vec<String>, natives: &Natives) {
    match compile_files(paths.as_slice(), natives) {
        Ok(chunk) => run_chunk(paths.as_slice(), chunk, natives),
        Err(error) => eprintln!("{error}"),
    }
}

fn main() {
    let natives = library::load();
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        run_repl(&natives);
    } else {
        run_files(paths, &natives);
    }
}
