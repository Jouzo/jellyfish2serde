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

    fn load_test_data(filename: &str) -> Module {
        generate_ast_structure(&filename.to_string())
    }

    fn load_test_expected_results(filename: &str) -> Result<String, std::io::Error> {
        Ok(fs::read_to_string(filename)?)
    }

    macro_rules! test_proto_transpiler {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result<(), std::io::Error>{
                    let (test_input, test_expected_result) = $value;
                    let test_module = load_test_data(test_input);
                    let expected_result = load_test_expected_results(test_expected_result)?;
                    let pt = ProtoTranspiler::new(test_module);
                    let output_code = pt.transpile().join("\n");
                    assert_eq!(output_code.clone().retain(|c| !c.is_whitespace()), expected_result.clone().retain(|c| !c.is_whitespace()));
                    Ok(())
                }
            )*
        }
    }
    test_proto_transpiler! {
        test_handle_basic_types: ("data/test/basic_types.ts", "data/test/expected_basic_types.proto"),
        test_handle_array_types: ("data/test/array_types.ts", "data/test/expected_array_types.proto"),
        test_handle_map_types: ("data/test/map_types.ts", "data/test/expected_map_types.proto"),
        test_handle_nested_types: ("data/test/nested_types.ts", "data/test/expected_nested_types.proto"),
    }
}
