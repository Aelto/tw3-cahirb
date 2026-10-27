mod assemble;
pub mod ast;
mod codegen;
mod lexer;
mod parser;
mod rtti;
mod symbols;
mod tables;

use std::path::{Path, PathBuf};

use crate::parser::RsBlob;

pub struct CompileInput<'a> {
    pub vanilla_roots: &'a [PathBuf],
    pub rtti_dump: &'a Path,
    pub script_roots: &'a [PathBuf],
}

pub fn compile(input: &CompileInput) -> Result<RsBlob, String> {
    let missing: Vec<String> = input
        .script_roots
        .iter()
        .chain(input.vanilla_roots)
        .filter(|root| !root.is_dir())
        .map(|root| root.display().to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!("directories not found: {}", missing.join(", ")));
    }

    let collect = |roots: &[PathBuf]| -> Vec<PathBuf> {
        roots.iter().flat_map(|r| collect_scripts(r)).collect()
    };

    let mod_files = parse_files(&collect(input.script_roots), false)?;
    let vanilla_files = parse_files(&collect(input.vanilla_roots), true)?;
    let rtti = rtti::load(input.rtti_dump)?;

    let mut symbols = symbols::Symbols::default();
    symbols.add_enums(&mod_files, true);
    symbols.add_enums(&vanilla_files, true);
    symbols.add_rtti_enums(&rtti);

    symbols.add_type_names(&mod_files);
    symbols.add_type_names(&vanilla_files);
    symbols.add_rtti_classes(&rtti);

    symbols.add_members(&mod_files)?;
    symbols.add_members(&vanilla_files)?;
    symbols.add_rtti_properties(&rtti);

    assemble::assemble(&symbols, &mod_files)
}

pub struct SourceFile {
    pub path: PathBuf,
    pub items: Vec<ast::Item>,
}

pub fn collect_scripts(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_into(root, &mut files);
    files
}

fn collect_into(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort_by_key(|path| sort_key(path));

    let (dirs, scripts): (Vec<_>, Vec<_>) = entries.into_iter().partition(|p| p.is_dir());

    files.extend(
        scripts
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("ws"))),
    );

    for dir in dirs {
        collect_into(&dir, files);
    }
}

fn sort_key(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_uppercase())
        .unwrap_or_default()
}

pub fn parse_files(paths: &[PathBuf], skip_bodies: bool) -> Result<Vec<SourceFile>, String> {
    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let source = decode_source(&bytes);
            let items = parser::parse_source(&source, skip_bodies)
                .map_err(|e| format!("{}: {e}", path.display()))?;

            Ok(SourceFile {
                path: path.clone(),
                items,
            })
        })
        .collect()
}

fn decode_source(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }

    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8_lossy(bytes).into_owned()
}
