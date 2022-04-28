pub mod interface;
pub trait Transpiler {
    fn transpile(self) -> Vec<String>;
}
