use convert_case::{Case, Casing};
use std::fmt::Display;
use swc_ecma_ast::{
    BindingIdent, Expr, TsArrayType, TsEntityName, TsFnParam, TsIndexSignature, TsInterfaceDecl,
    TsKeywordType, TsKeywordTypeKind, TsPropertySignature, TsType, TsTypeAnn, TsTypeElement,
    TsTypeParamDecl, TsTypeRef,
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
    fn map_type_rust(kind: TsKeywordTypeKind) -> String {
        match kind {
            TsKeywordTypeKind::TsStringKeyword => String::from("String"),
            TsKeywordTypeKind::TsNumberKeyword => String::from("u64"),
            TsKeywordTypeKind::TsBooleanKeyword => String::from("bool"),
            _ => panic!("MISSING KIND"),
        }
    }
    fn map_type_proto(kind: TsKeywordTypeKind) -> String {
        match kind {
            TsKeywordTypeKind::TsStringKeyword => String::from("string"),
            TsKeywordTypeKind::TsNumberKeyword => String::from("int64"),
            TsKeywordTypeKind::TsBooleanKeyword => String::from("bool"),
            _ => panic!("MISSING KIND"),
        }
    }

    pub fn map_type(&self) -> fn(TsKeywordTypeKind) -> String {
        match self {
            ConversionType::Rust => ConversionType::map_type_rust,
            ConversionType::Protobuf => ConversionType::map_type_proto,
        }
    }

    fn map_ts_type_rust(&self, type_ann: TsType) -> (String, String) {
        match type_ann {
            TsType::TsTypeRef(TsTypeRef {
                type_name: TsEntityName::Ident(ident),
                ..
            }) => (
                String::from(""),
                self.map_non_ts_keywords(ident.sym.to_string()),
            ),

            TsType::TsKeywordType(keyword) => (String::from(""), self.map_type()(keyword.kind)),
            TsType::TsArrayType(TsArrayType {
                elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
                ..
            }) => (String::from(""), format!("Vec<{}>", self.map_type()(kind))),
            TsType::TsArrayType(TsArrayType {
                elem_type:
                    box TsType::TsTypeRef(TsTypeRef {
                        type_name: TsEntityName::Ident(ident),
                        ..
                    }),
                ..
            }) => (String::from(""), format!("Vec<{}>", ident.sym)),
            _ => {
                // println!("type_ann : {:#?}", type_ann);
                // panic!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
                (
                    String::from(""),
                    format!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE"),
                )
            }
        }
    }
    fn map_ts_type_proto(&self, type_ann: TsType) -> (String, String) {
        match type_ann {
            TsType::TsTypeRef(TsTypeRef {
                type_name: TsEntityName::Ident(ident),
                ..
            }) => (
                String::from(""),
                self.map_non_ts_keywords(ident.sym.to_string()),
            ),

            TsType::TsKeywordType(keyword) => (String::from(""), self.map_type()(keyword.kind)),
            TsType::TsArrayType(TsArrayType {
                elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
                ..
            }) => (String::from("repeated"), self.map_type()(kind)),
            TsType::TsArrayType(TsArrayType {
                elem_type:
                    box TsType::TsTypeRef(TsTypeRef {
                        type_name: TsEntityName::Ident(ident),
                        ..
                    }),
                ..
            }) => (String::from("repeated"), ident.sym.to_string()),
            _ => {
                // println!("type_ann : {:#?}", type_ann);
                // panic!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE")
                (
                    String::from(""),
                    format!("UNINPLEMENTED TSTYPE FOR PROPERTY INTERFACE"),
                )
            }
        }
    }
    pub fn map_ts_types(&self, type_ann: TsType) -> (String, String) {
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
    pub fn extract_key_value_from_ts_type_element(
        &self,
        property: TsTypeElement,
    ) -> (String, String, String, bool, bool) {
        match property {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                key: box Expr::Ident(id),
                type_ann: Some(TsTypeAnn { box type_ann, .. }),
                optional,
                ..
            }) => {
                let (prefix, value) = self.map_ts_types(type_ann);
                (prefix, id.sym.to_string(), value, optional, false)
            }
            // return (key, val, optional)

            // Handle conversion for hashmap type such as
            // ```
            // export interface MasternodeResult<T> {
            //     [id: string]: T
            // }
            // ```
            // to
            // ```
            // pub struct MasternodeResult<T>(HashMap<String, T>);
            // ```
            TsTypeElement::TsIndexSignature(TsIndexSignature {
                params,
                type_ann: Some(TsTypeAnn { box type_ann, .. }),
                ..
            }) => {
                if let Some(TsFnParam::Ident(BindingIdent {
                    type_ann:
                        Some(TsTypeAnn {
                            type_ann: box TsType::TsKeywordType(keyword),
                            ..
                        }),
                    ..
                })) = params.get(0)
                {
                    let (prefix, value) = self.map_ts_types(type_ann);
                    return (prefix, self.map_type()(keyword.kind), value, false, true);
                } else {
                    unreachable!("found an unexpected ts type!");
                }
            }
            _ => unreachable!("found an unexpected ts type!"),
        }
    }
}

pub struct Param {
    pub prefix: String,
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
        write!(
            f,
            "{} {} {}",
            self.prefix,
            self.key.to_case(Case::Snake),
            self.val.clone()
        )
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
