use convert_case::{Case, Casing};

use swc_ecma_ast::{
    ArrayLit, AssignPat, AwaitExpr, BindingIdent, CallExpr, Class, ClassMember, ClassMethod, Expr,
    ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, Pat, PropName, ReturnStmt, Stmt, Str,
    TsArrayType, TsEntityName, TsType, TsTypeAnn, TsTypeParamInstantiation, TsTypeRef,
};

use crate::utils::{map_type_rust, ConversionType, Param};

fn get_cmd_args(args: Vec<ExprOrSpread>) -> (Option<String>, Vec<String>) {
    let mut it = args.into_iter();

    let cmd = if let Some(ExprOrSpread {
        expr: box Expr::Lit(Lit::Str(Str { value, .. })),
        ..
    }) = it.next()
    {
        Some(value.to_string())
    } else {
        None
    };

    let mut args = Vec::new();
    if let Some(ExprOrSpread {
        expr: box Expr::Array(ArrayLit { elems, .. }),
        ..
    }) = &it.next()
    {
        let mut it = elems.iter();
        while let Some(Some(ExprOrSpread { box expr, .. })) = it.next() {
            // println!("expr : {:#?}", expr);
            match expr {
                Expr::Member(MemberExpr {
                    obj: box Expr::Ident(Ident { sym, .. }),
                    prop: MemberProp::Ident(prop),
                    ..
                }) => {
                    args.push(format!(
                        "{}.{}.into()",
                        sym.to_string().to_case(Case::Snake),
                        prop.sym.to_string().to_case(Case::Snake)
                    ));
                }
                Expr::Ident(Ident { sym, .. }) => {
                    args.push(format!("{}.into()", sym.to_string().to_case(Case::Snake),));
                }
                _ => (),
            }
        }
    }

    (cmd, args)
}

pub fn handle_class(class: Class, conversion_type: ConversionType) {
    for member in class.body {
        let mut fn_name: Option<String> = None;
        let mut fn_params: Vec<Param> = Vec::new();
        let mut fn_return_type: Option<String> = None;

        let mut fn_cmd: Option<String> = None;
        let mut fn_args: Vec<String> = Vec::new();

        // Get method name and parameters
        if let ClassMember::Method(ClassMethod {
            key: PropName::Ident(ident),
            function,
            ..
        }) = member
        {
            fn_name = Some(ident.sym.to_string().to_case(Case::Snake));
            for param in function.params {
                // println!("param.pat : {:#?}", param.pat);
                match param.pat {
                    Pat::Ident(BindingIdent {
                        id,
                        type_ann: Some(TsTypeAnn { box type_ann, .. }),
                        ..
                    }) => match type_ann {
                        TsType::TsTypeRef(TsTypeRef {
                            type_name: TsEntityName::Ident(ident),
                            ..
                        }) => {
                            fn_params.push(Param {
                                key: id.sym.to_string(),
                                val: ident.sym.to_string(),
                                optional: id.optional,
                                conversion_type: conversion_type.get(),
                            });
                        }
                        TsType::TsKeywordType(keyword) => {
                            fn_params.push(Param {
                                key: id.sym.to_string(),
                                val: map_type_rust(keyword.kind),
                                optional: id.optional,
                                conversion_type: conversion_type.get(),
                            });
                        }
                        _ => (),
                    },
                    Pat::Assign(AssignPat {
                        left:
                            box Pat::Ident(BindingIdent {
                                id,
                                type_ann: Some(TsTypeAnn { type_ann, .. }),
                                ..
                            }),
                        right: _,
                        ..
                    }) => {
                        match *type_ann {
                            TsType::TsArrayType(TsArrayType {
                                elem_type:
                                    box TsType::TsTypeRef(TsTypeRef {
                                        type_name: TsEntityName::Ident(ident),
                                        ..
                                    }),
                                ..
                            }) => {
                                fn_params.push(Param {
                                    key: id.sym.to_string(),
                                    val: ident.sym.to_string(),
                                    optional: true,
                                    conversion_type: conversion_type.get(),
                                });
                                // fn_has_utxo = true;
                            }
                            _ => {}
                        }
                    }
                    _ => (),
                }
            }

            // Get method arguments
            if let Some(body) = function.body {
                for statement in body.stmts {
                    match statement {
                        Stmt::Return(ReturnStmt {
                            arg:
                                Some(box Expr::Await(
                                    AwaitExpr {
                                        arg: box Expr::Call(CallExpr { args, .. }),
                                        ..
                                    },
                                    ..,
                                )),
                            ..
                        }) => {
                            (fn_cmd, fn_args) = get_cmd_args(args);
                        }
                        _ => (),
                    }
                    // Some(box TsType::TsArrayType(TsArrayType {
                    //     elem_type:
                    //         box TsType::TsTypeRef(TsTypeRef {
                    //             type_name: TsEntityName::Ident(_ident),
                    //             ..
                    //         }),
                    //     ..
                    // })) => {
                }
            }

            // Get method return type
            if let Some(TsTypeAnn { box type_ann, .. }) = function.return_type {
                if let TsType::TsTypeRef(TsTypeRef {
                    type_name: TsEntityName::Ident(_ident),
                    type_params: Some(TsTypeParamInstantiation { params, .. }),
                    ..
                }) = type_ann
                {
                    match params.get(0) {
                        Some(box TsType::TsKeywordType(keyword)) => {
                            fn_return_type = Some(map_type_rust(keyword.kind));
                        }
                        Some(box TsType::TsArrayType(TsArrayType {
                            elem_type:
                                box TsType::TsTypeRef(TsTypeRef {
                                    type_name: TsEntityName::Ident(_ident),
                                    ..
                                }),
                            ..
                        })) => {
                            fn_return_type = Some(format!("Vec<{}>", _ident.sym));
                        }
                        Some(box TsType::TsTypeRef(TsTypeRef {
                            type_name: TsEntityName::Ident(_ident),
                            ..
                        })) => {
                            fn_return_type = Some(_ident.sym.to_string());
                        }
                        _ => fn_return_type = Some("NEED TO HANDLE ENUM".to_string()), // TODO Need to handle enum return
                    }
                }
            }

            let mut format_method = format!(
                "pub async fn {}(&self",
                fn_name.as_ref().expect("Missing fn name")
            );
            for param in &fn_params {
                format_method.push_str(&format!(", {}", param));
            }
            format_method.push(')');
            format_method.push_str(&format!(
                " -> Result<{}> {{",
                fn_return_type.as_ref().expect("Missing return type")
            ));

            println!("{}", format_method);

            for param in fn_params {
                if param.optional {
                    println!("\tlet utxos = utxos.unwrap_or_default());");
                }
            }

            let mut format_body = format!(
                "\tself.call(\"{}\", &[",
                fn_cmd.expect("Missing function command")
            );

            let mut it = fn_args.into_iter();
            if let Some(arg) = it.next() {
                format_body.push_str(&arg.to_string());
            }
            for arg in it {
                format_body.push_str(&format!(", {}", arg));
            }

            format_body.push_str("]).await");
            println!("{}", format_body);
            println!("}}\n");
        }
    }
}
