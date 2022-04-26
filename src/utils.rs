use convert_case::{Case, Casing};
use std::fmt::Display;
use swc_ecma_ast::{
    TsArrayType, TsEntityName, TsKeywordType, TsKeywordTypeKind, TsType, TsTypeRef,
};

pub enum ConversionType {
    Rust,
    Protobuf,
}

impl ConversionType {
    pub fn get(&self) -> ConversionType {
        match self {
            ConversionType::Protobuf => ConversionType::Protobuf,
            ConversionType::Rust => ConversionType::Rust,
        }
    }

    pub fn map_type(&self) -> fn(TsKeywordTypeKind) -> String {
        match self {
            ConversionType::Rust => map_type_rust,
            ConversionType::Protobuf => map_type_proto,
        }
    }
    /*

    */
    fn map_ts_type_rust(&self, type_ann: TsType) -> String {
        match type_ann {
            TsType::TsTypeRef(TsTypeRef {
                type_name: TsEntityName::Ident(ident),
                ..
            }) => self.map_non_ts_keywords(ident.sym.to_string()),

            TsType::TsKeywordType(keyword) => self.map_type()(keyword.kind),
            TsType::TsArrayType(TsArrayType {
                elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
                ..
            }) => format!("Vec<{}>", self.map_type()(kind)),
            TsType::TsArrayType(TsArrayType {
                elem_type:
                    box TsType::TsTypeRef(TsTypeRef {
                        type_name: TsEntityName::Ident(ident),
                        ..
                    }),
                ..
            }) => format!("Vec<{}>", ident.sym),
            _ => {
                // println!("type_ann : {:#?}", type_ann);
                // panic!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
                format!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
            }
        }
    }
    fn map_ts_type_proto(&self, type_ann: TsType) -> String {
        match type_ann {
            TsType::TsTypeRef(TsTypeRef {
                type_name: TsEntityName::Ident(ident),
                ..
            }) => self.map_non_ts_keywords(ident.sym.to_string()),

            TsType::TsKeywordType(keyword) => self.map_type()(keyword.kind),
            TsType::TsArrayType(TsArrayType {
                elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
                ..
            }) => format!("repeated {}", self.map_type()(kind)),
            TsType::TsArrayType(TsArrayType {
                elem_type:
                    box TsType::TsTypeRef(TsTypeRef {
                        type_name: TsEntityName::Ident(ident),
                        ..
                    }),
                ..
            }) => format!("repeated {}", ident.sym),
            _ => {
                // println!("type_ann : {:#?}", type_ann);
                // panic!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
                format!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
            }
        }
    }
    pub fn map_ts_types(&self, type_ann: TsType) -> String {
        match self {
            ConversionType::Rust => self.map_ts_type_rust(type_ann),
            ConversionType::Protobuf => self.map_ts_type_proto(type_ann),
        }
    }
    pub fn map_non_ts_keywords(&self, keyword: String) -> String {
        match self {
            ConversionType::Rust => match keyword.as_str() {
                "BigNumber" => String::from("Decimal"),
                _ => keyword,
            },
            ConversionType::Protobuf => match keyword.as_str() {
                "BigNumber" => String::from("int64"),
                _ => keyword,
            },
        }
    }
}
pub fn map_type_rust(kind: TsKeywordTypeKind) -> String {
    match kind {
        TsKeywordTypeKind::TsStringKeyword => String::from("String"),
        TsKeywordTypeKind::TsNumberKeyword => String::from("u64"),
        TsKeywordTypeKind::TsBooleanKeyword => String::from("bool"),
        _ => panic!("MISSING KIND"),
    }
}
pub fn map_type_proto(kind: TsKeywordTypeKind) -> String {
    match kind {
        TsKeywordTypeKind::TsStringKeyword => String::from("string"),
        TsKeywordTypeKind::TsNumberKeyword => String::from("int64"),
        TsKeywordTypeKind::TsBooleanKeyword => String::from("bool"),
        _ => panic!("MISSING KIND"),
    }
}

pub struct Param {
    pub key: String,
    pub val: String,
    pub optional: bool,
    pub conversion_type: ConversionType,
}

impl Param {
    fn fmt_rust(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.val.clone();
        let opt_val = if self.optional {
            format!("Option<{}>", val)
        } else {
            val
        };
        let str = format!("{}: {}", self.key.to_case(Case::Snake), opt_val);
        write!(f, "{},", str)
    }
    fn fmt_proto(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key.to_case(Case::Snake), self.val.clone())
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.conversion_type {
            ConversionType::Protobuf => self.fmt_proto(f),
            ConversionType::Rust => self.fmt_rust(f),
        }
    }
}
