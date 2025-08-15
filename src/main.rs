#![allow(unused)]
#![feature(string_from_utf8_lossy_owned)]

use crate::parser::{WithCodeEmitting, WithTableResolving};

mod decompiler;
mod parser;

fn main() {
    let blob = {
        let mut blob = parser::RsBlob::from_file("precompiled.rsblob");
        parser::TableManager::init(&mut blob);

        blob
    };

    // let functions_to_decompile = vec![
    //     blob.classes.first().and_then(|c| {
    //         c.functions
    //             .iter()
    //             .find(|f| f.name.try_resolve().unwrap_or(&String::new()) == "handleCurrentChoices")
    //     }),
    //     blob.classes.last().and_then(|c| c.functions.last()),
    // ];

    // for function in functions_to_decompile {
    //     if let Some(function) = function {
    //         println!("Decompiling {}", function.name);

    //         let parsed = function.parse_bytecode();

    //         let iter = decompiler::InstructionsIter::new(&parsed.instructions);

    //         let (_, result) = iter.ok(decompiler::ast::FunctionDeclaration::decompile(
    //             iter.clone(),
    //         ));

    //         dbg!(result);
    //     }
    // }

    decompile_bytecode(blob);
    // parse_and_emit_instructions(blob);
}

fn decompile_bytecode(blob: parser::RsBlob) {
    'classes: for class in &blob.classes {
        for function in &class.functions {
            let parsed = function.parse_bytecode();

            let instructions_iter = decompiler::InstructionsIter::new(&parsed.instructions);
            let result = decompiler::ast::FunctionDeclaration::decompile(instructions_iter);

            match result {
                Ok(v) => {
                    if v.0.is_finished() {
                        println!("✔️ Successfully decompiled {}", function.name);
                        // dbg!(v.1);
                    } else {
                        println!("❌ Failed to decompile {}", function.name);

                        println!(
                            "  stopped at {:?}",
                            v.0.peek().map(|i| i.into_emitted_code())
                        );

                        // dbg!(&v.1);
                        break 'classes;
                    }
                }
                Err(e) => {
                    println!("❌ Failed to decompile {}", function.name);
                    println!("  error = {e}");
                    break 'classes;
                }
            }
        }
    }
}

fn parse_and_emit_instructions(blob: parser::RsBlob) {
    let mut parsed_functions = Vec::new();

    for class in &blob.classes {
        for function in &class.functions {
            println!("parsing {}", function.name);
            parsed_functions.push(function.parse_bytecode());
        }
    }

    for func in &blob.ext_replace_global_functions {
        println!("parsing {}", func.name);
        parsed_functions.push(func.parse_bytecode());
    }

    for func in &blob.ext_replace_class_functions {
        println!("parsing {}", func.name);
        parsed_functions.push(func.parse_bytecode());
    }

    let mut code = String::new();
    for body in parsed_functions {
        use std::fmt::Write;

        writeln!(code, "{}", body.definition_description);
        body.emit_code(&mut code);
        code.push_str("\n\n");
    }

    println!("{code}");
    std::fs::write("output.py", code).unwrap();
}
