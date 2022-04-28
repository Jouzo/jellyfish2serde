#![feature(box_patterns)]

mod class;
mod enums;
mod interface;
mod transpilers;
mod utils;

use transpilers::{generate_ast_structure, transpiler_factory};

fn main() -> Result<(), ()> {
    // grab argument and fail if there is no more arguments
    let args: Vec<String> = std::env::args().collect();
    let input_file = &args[1];
    let transpile_type = &args[2];

    let ast_module = generate_ast_structure(input_file);
    //let transpiled_code = transpiler_factory(ast_module, transpile_type.to_string()).transpile();

    Ok(())
}
// use class::handle_class;
// use enums::handle_enum;
// use interface::handle_interface;
//
// use std::path::Path;
//
// use swc_common::sync::Lrc;
// use swc_common::{
//     errors::{ColorConfig, Handler},
//     SourceMap,
// };
//
// use swc_ecma_ast::ModuleDecl::ExportDecl;
// use swc_ecma_ast::ModuleItem::ModuleDecl;
// use swc_ecma_ast::{ClassDecl, Decl, ModuleItem};
//
// use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
//
// fn transpile(body: Vec<ModuleItem>) {
//     for item in body {
//         if let ModuleDecl(ExportDecl(export)) = item {
//             match export.decl {
//                 Decl::TsInterface(interface) => {
//                     handle_interface(interface);
//                 }
//                 Decl::Class(ClassDecl { class, .. }) => {
//                     handle_class(class);
//                 }
//                 Decl::TsEnum(_enum) => {
//                     handle_enum(_enum);
//                 }
//                 _ => (),
//             }
//         }
//     }
// }
//
// fn main() {
//     let cm: Lrc<SourceMap> = Default::default();
//     let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
//
//     let arg = std::env::args().nth(1);
//     let fm = cm
//         .load_file(Path::new(arg.as_ref().unwrap()))
//         .unwrap_or_else(|_| panic!("failed to load {}", arg.as_ref().unwrap()));
//
//     let lexer = Lexer::new(
//         Syntax::Typescript(Default::default()),
//         Default::default(),
//         StringInput::from(&*fm),
//         None,
//     );
//
//     let mut parser = Parser::new_from(lexer);
//
//     for e in parser.take_errors() {
//         e.into_diagnostic(&handler).emit();
//     }
//
//     let module = parser
//         .parse_module()
//         .map_err(|e| {
//             // Unrecoverable fatal error occurred
//             e.into_diagnostic(&handler).emit();
//         })
//         .expect("failed to parser module");
//
//     transpile(module.body)
// }
