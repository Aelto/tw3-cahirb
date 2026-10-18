#![allow(unused)]
#![feature(string_from_utf8_lossy_owned)]

use orx_parallel::{IntoParIter, ParIter};

use crate::parser::{WithInstructionEmitting, WithTableResolving, ast::FunctionDefinition};

mod compiler;
mod decompiler;
mod parser;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if let [command, path] = args.as_slice()
        && command == "roundtrip"
    {
        roundtrip(path);
        return;
    }

    if let [command, output, vanilla, rtti_dump, roots @ ..] = args.as_slice()
        && command == "compile"
    {
        compile(output, vanilla, rtti_dump, roots);
        return;
    }

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

    // decompile_bytecode(blob);
    parse_and_emit_instructions(blob);
}

fn compile(output: &str, vanilla: &str, rtti_dump: &str, roots: &[String]) {
    use parser::WithWriting;
    use std::path::{Path, PathBuf};

    let started = std::time::Instant::now();
    let input = compiler::CompileInput {
        vanilla_roots: &[PathBuf::from(vanilla)],
        rtti_dump: Path::new(rtti_dump),
        script_roots: &roots.iter().map(PathBuf::from).collect::<Vec<_>>(),
    };

    match compiler::compile(&input) {
        Ok(blob) => {
            std::fs::write(output, blob.to_bytes()).expect("failed to write .rsblob");
            println!("☑️ wrote {output} in {:?}", started.elapsed());
        }
        Err(error) => {
            println!("❌ {error}");
            std::process::exit(1);
        }
    }
}

fn roundtrip(path: &str) {
    use parser::WithWriting;

    use parser::WithParsing;
    use parser::ast::{ByteCodeRef, ParsedFunctionBytecode};

    let original = std::fs::read(path).expect("failed to read .rsblob bytes");

    let mut blob = parser::RsBlob::from_bytes(&original);
    for function in blob.functions_mut() {
        let (_, decoded) = ParsedFunctionBytecode::parse(function.bytecode.as_ref())
            .expect("function_bytecode_parsing_failure");

        function.bytecode = ByteCodeRef::new(decoded.to_bytes());
    }

    let written = blob.to_bytes();

    let first_difference = original
        .iter()
        .zip(&written)
        .position(|(a, b)| a != b)
        .or_else(|| (original.len() != written.len()).then(|| original.len().min(written.len())));

    let Some(offset) = first_difference else {
        println!("✔️ byte identical ({} bytes)", original.len());
        return;
    };

    let window = |bytes: &[u8]| -> Vec<u8> {
        let start = offset.saturating_sub(16);
        let end = (offset + 16).min(bytes.len());
        bytes[start.min(end)..end].to_vec()
    };

    println!(
        "❌ first difference at {offset:#x} (original {} bytes, written {} bytes)",
        original.len(),
        written.len()
    );
    println!("original: {:02x?}", window(&original));
    println!("written:  {:02x?}", window(&written));
}

fn decompile_bytecode(blob: parser::RsBlob) {
    use decompiler::ast::FunctionDeclaration;
    use orx_parallel::ParallelizableCollection;

    fn decompile_function_definitions(
        defs: impl ParIter<Item = FunctionDefinition>,
    ) -> Vec<FunctionDeclaration> {
        defs.filter_map(|function: FunctionDefinition| {
            let parsed = function.parse_bytecode();
            let instructions_iter = decompiler::InstructionsIter::new(&parsed.instructions);
            let definition = function.parse_definition();
            let result = FunctionDeclaration::decompile(instructions_iter, definition);

            match result {
                Ok(v) => {
                    if v.0.is_finished() {
                        // println!("✔️ Successfully decompiled {}", function.name);

                        Some(v.1)
                    } else {
                        println!("❌ Failed to decompile {}", function.name);

                        println!(
                            "  stopped at {:?}",
                            v.0.peek().map(|i| i.into_emitted_instruction())
                        );

                        dbg!(&v.1);

                        None
                    }
                }
                Err(e) => {
                    println!("❌ Failed to decompile {}", function.name);
                    println!("  error = {e}");
                    None
                }
            }
        })
        .collect()
    }

    let class_functions =
        decompile_function_definitions(blob.classes.into_par().flat_map(|c| c.functions));
    let global_functions = decompile_function_definitions(blob.global_functions.into_par());
    let ext_wrap_functions = decompile_function_definitions(blob.ext_wrap_functions.into_par());
    let ext_add_functions = decompile_function_definitions(blob.ext_add_functions.into_par());
    let ext_replace_class_functions =
        decompile_function_definitions(blob.ext_replace_class_functions.into_par());
    let ext_replace_global_functions =
        decompile_function_definitions(blob.ext_replace_global_functions.into_par());

    let all_functions = class_functions
        .iter()
        .chain(global_functions.iter())
        .chain(ext_wrap_functions.iter())
        .chain(ext_add_functions.iter())
        .chain(ext_replace_class_functions.iter())
        .chain(ext_replace_global_functions.iter());

    let mut count = 0;
    let mut output = String::new();
    for result in all_functions {
        use crate::decompiler::WithCodeEmitting;

        let mut emitter = crate::decompiler::CodeEmitter::new();
        emitter.linebreak();
        result.emit_code(&mut emitter);
        emitter.linebreak();

        output.push_str(emitter.to_string());

        count += 1;
    }

    std::fs::write("code.ws", &output).unwrap();

    println!("Decompiled {count} functions");
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

    for func in &blob.ext_add_functions {
        println!("parsing {}", func.name);
        parsed_functions.push(func.parse_bytecode());
    }

    for func in &blob.ext_wrap_functions {
        println!("parsing {}", func.name);
        parsed_functions.push(func.parse_bytecode());
    }

    let mut code = String::new();
    for body in parsed_functions {
        use std::fmt::Write;

        writeln!(code, "{}", body.definition_description);
        body.emit_instruction(&mut code);
        code.push_str("\n\n");
    }

    println!("{code}");
    std::fs::write("output.py", code).unwrap();
}
