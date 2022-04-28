pub mod common;
pub mod proto;
pub mod rust;

use common::Transpiler;
use proto::transpiler::ProtoTranspiler;
use rust::transpiler::RustTranspiler;

use std::path::Path;

use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_ast::Module;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax};

pub fn transpiler_factory(
    module: Module,
    conversion_type: String,
) -> Box<(dyn Transpiler + 'static)> {
    match conversion_type.to_lowercase().as_str() {
        "proto" => Box::new(ProtoTranspiler::new(module)),
        _ => Box::new(RustTranspiler::new(module)),
    }
}

pub fn generate_ast_structure(filename: &String) -> Module {
    let source_map: Lrc<SourceMap> = Default::default();
    let handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map.clone()));
    let source_file = source_map
        .load_file(Path::new(filename))
        .unwrap_or_else(|_| panic!("failed to load {}", filename));
    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    for parser_error in parser.take_errors() {
        parser_error.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable death
            e.into_diagnostic(&handler).emit();
        })
        .expect("failed to parse module");

    return module;
}
