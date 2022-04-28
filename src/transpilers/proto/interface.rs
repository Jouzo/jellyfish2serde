use crate::transpilers::common::interface::{Interface, Property};
use swc_ecma_ast::{
    BindingIdent, Expr, TsArrayType, TsFnParam, TsIndexSignature, TsInterfaceDecl, TsKeywordType,
    TsKeywordTypeKind, TsPropertySignature, TsType, TsTypeAnn, TsTypeElement,
};

// all the basic typescript types
enum TypescriptKeywordType {
    BOOLEAN,
    NUMBER,
    STRING,
}
// struct to hold the properties of an interface
struct TypescriptProperty {
    name: String,
    value: TypescriptKeywordType,
}

// basic types
impl TypescriptProperty {
    pub fn new(name: String, value: TsKeywordTypeKind) -> Self {
        let mut val = TypescriptKeywordType::STRING;
        match value {
            TsKeywordTypeKind::TsNumberKeyword => val = TypescriptKeywordType::NUMBER,
            TsKeywordTypeKind::TsBooleanKeyword => val = TypescriptKeywordType::BOOLEAN,
            TsKeywordTypeKind::TsStringKeyword => val = TypescriptKeywordType::STRING,
            _ => unreachable!("unhandled TypescriptProperty"),
        };
        TypescriptProperty { name, value: val }
    }
}

impl Property for TypescriptProperty {
    fn to_string(&self) -> String {
        match &self.value {
            TypescriptKeywordType::NUMBER => format!("int32 {}", self.name),
            TypescriptKeywordType::BOOLEAN => format!("bool {}", self.name),
            TypescriptKeywordType::STRING => format!("string {}", self.name),
        }
    }
}

// array type
struct TypescriptArrayProperty {
    property: TypescriptProperty,
}

impl TypescriptArrayProperty {
    pub fn new(name: String, value: TsKeywordTypeKind) -> Self {
        TypescriptArrayProperty {
            property: TypescriptProperty::new(name, value),
        }
    }
}

impl Property for TypescriptArrayProperty {
    fn to_string(&self) -> String {
        format!("repeated {}", self.property.to_string())
    }
}

pub fn ts_type_factory(name: String, ts_type: TsType) -> Box<(dyn Property + 'static)> {
    match ts_type {
        TsType::TsKeywordType(keyword) => Box::new(TypescriptProperty::new(name, keyword.kind)),
        TsType::TsArrayType(TsArrayType {
            elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
            ..
        }) => Box::new(TypescriptArrayProperty::new(name, kind)),
        _ => unreachable!("unhandled ts_keyword_type for ts_type_factory"),
    }
}

// hash type
// struct TypescriptMapProperty {
//     key: TypescriptProperty,
//     value: Box<dyn Property>,
// }
// impl TypescriptMapProperty {
//     pub fn new(name: String, ts_type_key: TsKeywordTypeKind, ts_type_value: TsType) -> Self {
//         TypescriptMapProperty {
//             key: TypescriptProperty::new(String::from(""), ts_type_key),
//             value: ts_type_factory(String::from(""), ts_type_value),
//         }
//     }
// }
// impl Property for TypescriptMapProperty {
//     fn to_string(&self) -> String {
//
//     }
// }
struct TypescriptInterface {
    name: String,
    properties: Vec<Box<dyn Property>>,
    generics: Option<Vec<String>>,
}

impl TypescriptInterface {
    pub fn new(n: String, p: Vec<Box<dyn Property>>, g: Option<Vec<String>>) -> Self {
        TypescriptInterface {
            name: n,
            properties: p,
            generics: g,
        }
    }
}

impl Interface for TypescriptInterface {
    fn to_string(&self) -> String {
        let mut output = format!("message {} {{\n", self.name);
        let mut count = 0;
        for kw in &self.properties {
            count += 1;
            output.push_str(format!("  {} = {};\n", kw.as_ref().to_string(), count).as_str());
        }
        output.push_str("}\n");

        output
    }
}

pub fn interface_factory(ts_interface: TsInterfaceDecl) {}
pub fn handle_interface(ts_interface: TsInterfaceDecl) -> Box<(dyn Interface + 'static)> {
    let mut properties: Vec<Box<dyn Property>> = vec![];
    for property in ts_interface.body.body {
        match property {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                key: box Expr::Ident(id),
                type_ann: Some(TsTypeAnn { box type_ann, .. }),
                ..
            }) => properties.push(ts_type_factory(id.sym.to_string(), type_ann)),
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
                // TODO: Handle this !
                {}
            }
            _ => unreachable!("unhandled TsTypeElement at interface factory"),
        }
    }
    Box::new(TypescriptInterface::new(
        ts_interface.id.sym.to_string(),
        properties,
        None,
    ))
}
