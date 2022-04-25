use convert_case::{Case, Casing};
use std::fmt::Display;
use swc_ecma_ast::TsKeywordTypeKind;

pub fn map_type(kind: TsKeywordTypeKind) -> String {
    match kind {
        TsKeywordTypeKind::TsStringKeyword => String::from("String"),
        TsKeywordTypeKind::TsNumberKeyword => String::from("u64"),
        TsKeywordTypeKind::TsBooleanKeyword => String::from("bool"),
        _ => panic!("MISSING KIND"),
    }
}

pub struct Param {
    pub key: String,
    pub val: String,
    pub optional: bool,
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.val.clone();
        let opt_val = if self.optional {
            format!("Option<{}>", val)
        } else {
            val
        };
        let str = format!("{}: {}", self.key.to_case(Case::Snake), opt_val);
        write!(f, "{},", str)
    }
}
