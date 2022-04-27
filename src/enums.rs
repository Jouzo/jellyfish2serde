use convert_case::{Case, Casing};
use swc_ecma_ast::{TsEnumDecl, TsEnumMember, TsEnumMemberId};

pub fn handle_enum(TsEnumDecl { id, members, .. }: TsEnumDecl) {
    println!("#[derive(Debug, Serialize, Deserialize)]");
    println!("pub enum {} {{", id.sym);
    for member in members {
        if let TsEnumMember {
            id: TsEnumMemberId::Ident(id),
            ..
        } = member
        {
            println!("\t{}", id.sym.to_string().to_case(Case::Pascal))
        }
    }
    println!("}}")
}
