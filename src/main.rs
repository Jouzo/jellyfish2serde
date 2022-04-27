#![feature(box_patterns)]

mod class;
mod enums;
mod interface;
mod utils;

use class::handle_class;
use enums::handle_enum;
use interface::handle_interface;
use utils::ConversionType;

use std::path::Path;

use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};

use swc_ecma_ast::ModuleDecl::ExportDecl;
use swc_ecma_ast::ModuleItem::ModuleDecl;
use swc_ecma_ast::{ClassDecl, Decl, ModuleItem};

use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

fn transpile(body: Vec<ModuleItem>, con_type: ConversionType) {
    for item in body {
        let conversion_type = match con_type {
            ConversionType::Rust => ConversionType::Rust,
            ConversionType::Protobuf => ConversionType::Protobuf,
        };
        if let ModuleDecl(ExportDecl(export)) = item {
            match export.decl {
                Decl::TsInterface(interface) => {
                    handle_interface(interface, conversion_type);
                }
                Decl::Class(ClassDecl { class, .. }) => {
                    handle_class(class, conversion_type);
                }
                Decl::TsEnum(_enum) => {
                    handle_enum(_enum);
                }
                _ => (),
            }
        }
    }
}

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let args: Vec<String> = std::env::args().collect();
    // println!("input file: {}", &args[1]);
    // println!("conversion type: {}", &args[2]);

    let arg = args[1].as_str();
    let conversion_type = match args[2].as_str() {
        "proto" => ConversionType::Protobuf,
        _ => ConversionType::Rust,
    };
    let fm = cm
        .load_file(Path::new(&arg))
        .unwrap_or_else(|_| panic!("failed to load {}", &arg));

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit();
        })
        .expect("failed to parser module");

    transpile(module.body, conversion_type)
}
