use std::fmt::Display;

extern crate swc_ecma_parser;

use swc_ecma_ast::{
    BindingIdent, Expr, TsArrayType, TsEntityName, TsFnParam, TsIndexSignature, TsInterfaceDecl,
    TsKeywordType, TsPropertySignature, TsType, TsTypeAnn, TsTypeElement, TsTypeParamDecl,
    TsTypeRef,
};

use crate::utils::{map_type, Param};

struct Interface {
    name: String,
    generics: Vec<String>,
    properties: Vec<Param>,
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(Debug, Serialize, Deserialize)]")?;
        writeln!(f, "#[serde(rename_all = \"camelCase\")]")?;

        let name = if !self.generics.is_empty() {
            format!("{}<{}>", self.name, self.generics.join(", "))
        } else {
            self.name.clone()
        };
        writeln!(f, "pub struct {} {{", name)?;

        for property in &self.properties {
            writeln!(f, "  {}", property)?;
        }
        writeln!(f, "}}")
    }
}

struct MapInterface {
    name: String,
    generics: Vec<String>,
    key: String,
    val: String,
}

impl Display for MapInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(Debug, Serialize, Deserialize)]")?;

        let name = if !self.generics.is_empty() {
            format!("{}<{}>", self.name, self.generics.join(", "))
        } else {
            self.name.clone()
        };
        writeln!(
            f,
            "pub struct {}(HashMap<{}, {}>);",
            name, self.key, self.val
        )
    }
}

fn type_ann_to_string(type_ann: TsType) -> String {
    match type_ann {
        TsType::TsTypeRef(TsTypeRef {
            type_name: TsEntityName::Ident(ident),
            ..
        }) =>
        // Replace BigNumber by rust_decimal
        {
            if ident.sym.to_string() == "BigNumber" {
                String::from("Decimal")
            } else {
                ident.sym.to_string()
            }
        }
        TsType::TsKeywordType(keyword) => map_type(keyword.kind),
        TsType::TsArrayType(TsArrayType {
            elem_type: box TsType::TsKeywordType(TsKeywordType { kind, .. }),
            ..
        }) => format!("Vec<{}>", map_type(kind)),
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

pub fn handle_interface(interface: TsInterfaceDecl) {
    let mut generics: Vec<String> = Vec::new();
    if let Some(TsTypeParamDecl { params, .. }) = &interface.type_params {
        for param in params {
            generics.push(param.name.sym.to_string());
        }
    }

    let mut properties: Vec<Param> = Vec::new();
    for property in interface.body.body {
        match property {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                key: box Expr::Ident(id),
                type_ann: Some(TsTypeAnn { box type_ann, .. }),
                optional,
                ..
            }) => properties.push(Param {
                key: id.sym.to_string(),
                val: type_ann_to_string(type_ann),
                optional,
            }),
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
                    return println!(
                        "{}",
                        MapInterface {
                            generics,
                            name: interface.id.sym.to_string(),
                            key: map_type(keyword.kind),
                            val: type_ann_to_string(type_ann)
                        }
                    );
                }
            }
            _ => (),
        }
    }

    println!(
        "{}",
        Interface {
            generics,
            name: interface.id.sym.to_string(),
            properties
        }
    )
}
