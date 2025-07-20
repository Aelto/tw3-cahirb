use crate::parser::prelude::*;

static TABLES: std::sync::OnceLock<TableManager> = std::sync::OnceLock::new();

pub struct TableManager {
    string_table: Vec<String>,
    import_type_table: Vec<ImportType>,
    import_property_table: Vec<ImportProperty>, // unused atm,
    import_function_table: Vec<ImportFunction>,
}

impl TableManager {
    pub fn init(blob: &mut RsBlob) {
        TABLES.get_or_init(|| Self {
            string_table: std::mem::take(&mut blob.string_table),
            import_type_table: std::mem::take(&mut blob.import_type_table),
            import_property_table: std::mem::take(&mut blob.import_property_table),
            import_function_table: std::mem::take(&mut blob.import_function_table),
        });

        if let Some(tables) = TABLES.get() {
            dbg!(tables.import_function_table.len());
            dbg!(&tables.import_type_table);
            dbg!(&tables.import_function_table);
        }
    }

    pub fn try_resolve_string<'a>(index: usize) -> Option<&'a String> {
        TABLES.get().and_then(|t| t.string_table.get(index))
    }

    pub fn try_resolve_import_type<'a>(index: usize) -> Option<&'a ImportType> {
        TABLES.get().and_then(|t| t.import_type_table.get(index))
    }

    pub fn try_resolve_import_property<'a>(index: usize) -> Option<&'a ImportProperty> {
        TABLES
            .get()
            .and_then(|t| t.import_property_table.get(index))
    }

    pub fn try_resolve_import_function<'a>(index: usize) -> Option<&'a ImportFunction> {
        TABLES
            .get()
            .and_then(|t| t.import_function_table.get(index))
    }
}

pub trait WithTableResolving<T> {
    fn try_resolve<'a>(&self) -> Option<&T>;
}
