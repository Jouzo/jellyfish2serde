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
    name: Option<String>,
    value: TypescriptKeywordType,
}

// basic types
impl TypescriptProperty {
    pub fn keyword_type(value: TsKeywordTypeKind) -> TypescriptKeywordType {
        match value {
            TsKeywordTypeKind::TsNumberKeyword => TypescriptKeywordType::NUMBER,
            TsKeywordTypeKind::TsBooleanKeyword => TypescriptKeywordType::BOOLEAN,
            TsKeywordTypeKind::TsStringKeyword => TypescriptKeywordType::STRING,
            _ => unreachable!("unhandled TypescriptProperty"),
        }
    }
    pub fn new(name: Option<String>, value: TsKeywordTypeKind) -> Self {
        let value = TypescriptProperty::keyword_type(value);
        TypescriptProperty { name, value }
    }
}

impl Property for TypescriptProperty {
    fn to_string(&self) -> String {
        let mut keyword = match &self.value {
            TypescriptKeywordType::NUMBER => String::from("int32"),
            TypescriptKeywordType::BOOLEAN => String::from("bool"),
            TypescriptKeywordType::STRING => String::from("string"),
        };
        if let Some(name) = &self.name {
            format!("{} {}", keyword, name)
        } else {
            format!("{}", keyword)
        }
    }
}

// array type
struct TypescriptArrayProperty {
    property: TypescriptProperty,
}

impl TypescriptArrayProperty {
    pub fn new(name: Option<String>, value: TsKeywordTypeKind) -> Self {
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

pub fn ts_type_factory(name: Option<String>, ts_type: TsType) -> Box<(dyn Property + 'static)> {
    match ts_type {
        TsType::TsKeywordType(keyword) => Box::new(TypescriptProperty::new(name, keyword.kind)),
        TsType::TsArrayType(TsArrayType {
            elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
            ..
        }) => Box::new(TypescriptArrayProperty::new(name, kind)),
        _ => unreachable!("unhandled ts_keyword_type for ts_type_factory"),
    }
}
// interfaces
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

// Map Interface
struct TypescriptMapInterface {
    name: String,
    keyword_name: String,
    keyword_type: TypescriptProperty,
    value_type: Box<dyn Property>,
}

impl TypescriptMapInterface {
    pub fn new(
        name: String,
        keyword_name: String,
        keyword_kind: TsKeywordTypeKind,
        type_ann: TsType,
    ) -> Self {
        let keyword_type = TypescriptProperty::new(None, keyword_kind);
        let value_type = ts_type_factory(None, type_ann);
        TypescriptMapInterface {
            name,
            keyword_name,
            keyword_type,
            value_type,
        }
    }
}

impl Interface for TypescriptMapInterface {
    fn to_string(&self) -> String {
        let mut output = format!("message {} {{\n", self.name);
        output.push_str(
            format!(
                "  map<{}, {}> {} = 0;\n",
                self.keyword_type.to_string(),
                self.value_type.as_ref().to_string(),
                self.keyword_name
            )
            .as_str(),
        );
        output.push_str("}\n");
        output
    }
}
enum TsInterfacePropertyTypes {
    TsProperty {
        interface_id: String,
        type_ann: TsType,
    },
    TsIndexProperty {
        interface_id: String,
        keyword_id: String,
        keyword_kind: TsKeywordTypeKind,
        type_ann: TsType,
    },
}
impl TsInterfacePropertyTypes {
    fn extract_interface_params(
        element: TsTypeElement,
        ts_interface: TsInterfaceDecl,
    ) -> TsInterfacePropertyTypes {
        match element {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                key: box Expr::Ident(id),
                type_ann: Some(TsTypeAnn { box type_ann, .. }),
                ..
            }) => {
                return TsInterfacePropertyTypes::TsProperty {
                    interface_id: id.sym.to_string(),
                    type_ann,
                }
            }
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
                    id,
                })) = params.get(0)
                {
                    return TsInterfacePropertyTypes::TsIndexProperty {
                        interface_id: ts_interface.id.sym.to_string(),
                        keyword_id: id.sym.to_string(),
                        keyword_kind: keyword.kind,
                        type_ann,
                    };
                } else {
                    unreachable!("unhandled section for TsIndexSignature")
                }
            }
            _ => unreachable!("unhandled TsTypeElement at interface factory"),
        }
    }
}
pub fn handle_interface(ts_interface: TsInterfaceDecl) -> Box<(dyn Interface + 'static)> {
    let mut properties: Vec<Box<dyn Property>> = vec![];
    for property in ts_interface.body.body {
        match TsInterfacePropertyTypes::extract_interface_params(property, ts_interface) {
            TsInterfacePropertyTypes::TsProperty {
                interface_id,
                type_ann,
            } => properties.push(ts_type_factory(Some(interface_id), type_ann)),
            TsInterfacePropertyTypes::TsIndexProperty {
                interface_id,
                keyword_id,
                keyword_kind,
                type_ann,
            } => {
                return Box::new(TypescriptMapInterface::new(
                    interface_id,
                    keyword_id,
                    keyword_kind,
                    type_ann,
                ));
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
