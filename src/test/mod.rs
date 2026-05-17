mod file;
mod parse;

use file::find_test_files;
use parse::parse_all_files;

pub fn test() {
    let files = find_test_files("./vitest", None);

    println!("Collected {} test files", files.len(),);

    parse_all_files(
        files,
        |path, ast| {
            println!("{:?} -> {} top-level nodes", path, ast.program.body.len(),);
        },
        |path, err| {
            eprintln!("ERROR {:?}: {}", path, err,);
        },
    );
}
