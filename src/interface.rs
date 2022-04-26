use std::fmt::Display;

extern crate swc_ecma_parser;

use swc_ecma_ast::{
    BindingIdent, Expr, TsFnParam, TsIndexSignature, TsInterfaceDecl, TsPropertySignature, TsType,
    TsTypeAnn, TsTypeElement, TsTypeParamDecl,
};

use crate::utils::{ConversionType, Param};

struct Interface {
    name: String,
    generics: Vec<String>,
    properties: Vec<Param>,
    conversion_type: ConversionType,
}

impl Interface {
    fn fmt_rust(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    fn fmt_proto(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = if !self.generics.is_empty() {
            format!("{}<{}>", self.name, self.generics.join(" "))
        } else {
            self.name.clone()
        };
        writeln!(f, "message {} {{", name)?;
        let mut property_count = 0;
        for property in &self.properties {
            writeln!(f, "  {} = {};", property, property_count)?;
            property_count += 1;
        }
        writeln!(f, "}}")
    }
}

impl Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.conversion_type {
            ConversionType::Rust => self.fmt_rust(f),
            ConversionType::Protobuf => self.fmt_proto(f),
        }
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

pub fn handle_interface(interface: TsInterfaceDecl, conversion_type: ConversionType) {
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
                val: conversion_type.map_ts_types(type_ann),
                optional,
                conversion_type: conversion_type.get(),
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
                            key: conversion_type.map_type()(keyword.kind),
                            val: conversion_type.map_ts_types(type_ann)
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
            properties,
            conversion_type,
        }
    )
}
