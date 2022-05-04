#![feature(box_patterns)]

mod transpilers;

use crate::transpilers::{
    common::Transpiler, generate_ast_structure, proto::transpiler::ProtoTranspiler,
};

fn main() -> Result<(), ()> {
    // grab argument and fail if there is no more arguments
    let args: Vec<String> = std::env::args().collect();
    let input_file = &args[1];

    let ast_module = generate_ast_structure(input_file);
    let transpiler = ProtoTranspiler::new(ast_module);
    let output = transpiler.transpile().join("\n");
    println!("{}", output);

    Ok(())
}
