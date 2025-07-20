#![allow(unused)]

use crate::parser::WithCodeEmitting;

// mod bytecode;
mod parser;

fn main() {
    let blob = {
        let mut blob = parser::RsBlob::from_file("precompiled.rsblob");
        parser::TableManager::init(&mut blob);

        blob
    };

    let mut parsed_functions = Vec::new();
    for class in &blob.classes {
        for function in &class.functions {
            parsed_functions.push(function.parse_bytecode());
        }
    }

    let mut code = String::new();
    for body in parsed_functions {
        body.emit_code(&mut code);
    }

    println!("{code}");
}
