#![allow(unused)]

use crate::parser::WithCodeEmitting;

mod decompiler;
mod parser;

fn main() {
    let blob = {
        let mut blob = parser::RsBlob::from_file("precompiled.rsblob");
        parser::TableManager::init(&mut blob);

        blob
    };

    if let Some(class) = blob.classes.last() {
        if let Some(function) = class.functions.last() {
            println!("Decompiling {}", function.name);

            let parsed = function.parse_bytecode();

            let mut iter = decompiler::iterator::instructions_iter(&parsed.instructions);
            decompiler::ast::FunctionDeclaration::decompile(&mut iter);
        }
    }

    // let mut parsed_functions = Vec::new();
    // for class in &blob.classes {
    //     for function in &class.functions {
    //         parsed_functions.push(function.parse_bytecode());
    //     }
    // }

    // let mut code = String::new();
    // for body in parsed_functions {
    //     body.emit_code(&mut code);
    //     code.push_str("\n\n");
    // }

    // println!("{code}");
    // // std::fs::write("output.py", code).unwrap();
}
