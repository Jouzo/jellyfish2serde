use crate::transpilers::common::Transpiler;
use crate::transpilers::proto::interface::handle_interface;
use swc_ecma_ast::ModuleDecl::ExportDecl;
use swc_ecma_ast::ModuleItem::ModuleDecl;
use swc_ecma_ast::{Decl, Module};

pub struct ProtoTranspiler {
    module: Module,
}

impl ProtoTranspiler {
    pub fn new(module: Module) -> Self {
        ProtoTranspiler { module }
    }
}

impl Transpiler for ProtoTranspiler {
    fn transpile(self) -> Vec<String> {
        let mut transpiled_code: Vec<String> = vec![];
        for ts_property in self.module.body {
            if let ModuleDecl(ExportDecl(export)) = ts_property {
                match export.decl {
                    Decl::TsInterface(interface) => {
                        // handle interface
                        transpiled_code.push(handle_interface(interface).as_ref().to_string());
                    }
                    _ => unreachable!("atm we only handle typescript interfaces"),
                };
            }
        }
        transpiled_code
    }
}

#[cfg(test)]
mod test_handle_interface {
    use super::ProtoTranspiler;
    use crate::transpilers::{common::Transpiler, generate_ast_structure};
    use std::fs;
    use swc_ecma_ast::Module;

    fn load_test_data(filename: String) -> Module {
        generate_ast_structure(&filename.to_string())
    }

    fn load_test_expected_results(filename: String) -> Result<String, std::io::Error> {
        Ok(fs::read_to_string(filename)?)
    }

    fn basic_types_module() -> Module {
        let filename = "data/test/basic_types.ts";
        load_test_data(filename.to_string())
    }

    fn basic_types_expected_results() -> String {
        let filename = "data/test/expected_basic_types.proto";
        load_test_expected_results(filename.to_string()).unwrap()
    }

    fn array_types_module() -> Module {
        let filename = "data/test/array_type.ts";
        load_test_data(filename.to_string())
    }

    fn array_types_expected_results() -> String {
        let filename = "data/test/expected_array_type.proto";
        load_test_expected_results(filename.to_string()).unwrap()
    }

    #[test]
    fn test_handle_basic_types() {
        let pt = ProtoTranspiler::new(basic_types_module());
        let output_code = pt.transpile().join("\n");
        assert_eq!(output_code, basic_types_expected_results());
    }

    #[test]
    fn test_handle_array_types() {
        let pt = ProtoTranspiler::new(array_types_module());
        let output_code = pt.transpile().join("\n");
        assert_eq!(output_code, array_types_expected_results());
    }
}
