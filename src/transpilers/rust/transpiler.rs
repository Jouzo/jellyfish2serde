use crate::transpilers::common::Transpiler;
use swc_ecma_ast::Module;

pub struct RustTranspiler {
    module: Module,
}
impl RustTranspiler {
    pub fn new(module: Module) -> Self {
        RustTranspiler { module }
    }
}
impl Transpiler for RustTranspiler {
    fn transpile(self) -> Vec<String> {
        vec![]
    }
}
