use std::path::Path;

pub struct RttiClass {
    pub name: String,
    pub base: Option<String>,
    pub properties: Vec<(String, String)>,
}

pub struct RttiEnum {
    pub name: String,
    pub options: Vec<(String, i32)>,
}

#[derive(Default)]
pub struct Rtti {
    pub classes: Vec<RttiClass>,
    pub enums: Vec<RttiEnum>,
}

const ROOT_CLASSES: [&str; 2] = ["IScriptable", "ISerializable"];

impl RttiClass {
    pub fn is_struct(&self) -> bool {
        self.base.is_none() && !ROOT_CLASSES.contains(&self.name.as_str())
    }
}

pub fn load(path: &Path) -> Result<Rtti, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rtti = Rtti::default();
    let mut function_depth = 0;
    let mut skip_options = false;

    for line in text.lines() {
        let line = line.trim();

        if line.starts_with("<function") || line.starts_with("<globalFunction") {
            if !line.ends_with("/>") {
                function_depth += 1;
            }
            continue;
        }
        if line.starts_with("</function") || line.starts_with("</globalFunction") {
            function_depth -= 1;
            continue;
        }
        if function_depth > 0 {
            continue;
        }

        if line.starts_with("<class ") {
            rtti.classes.push(RttiClass {
                name: attribute(line, "Name").ok_or("class without Name")?,
                base: attribute(line, "BaseClass"),
                properties: Vec::new(),
            });
        } else if line.starts_with("<property ") {
            let class = rtti.classes.last_mut().ok_or("property outside class")?;
            class.properties.push((
                attribute(line, "Name").ok_or("property without Name")?,
                attribute(line, "Type").ok_or("property without Type")?,
            ));
        } else if line.starts_with("<enum ") {
            skip_options = attribute(line, "IsScripted").as_deref() == Some("true");
            if skip_options {
                continue;
            }
            rtti.enums.push(RttiEnum {
                name: attribute(line, "Name").ok_or("enum without Name")?,
                options: Vec::new(),
            });
        } else if line.starts_with("<option ") && !skip_options {
            let option = rtti.enums.last_mut().ok_or("option outside enum")?;
            let value = attribute(line, "Value")
                .and_then(|v| v.parse().ok())
                .ok_or("option without numeric Value")?;
            option
                .options
                .push((attribute(line, "Name").ok_or("option without Name")?, value));
        }
    }

    Ok(rtti)
}

fn attribute(line: &str, key: &str) -> Option<String> {
    let marker = format!(" {key}=\"");
    let start = line.find(&marker)? + marker.len();
    let end = start + line[start..].find('"')?;

    Some(
        line[start..end]
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&"),
    )
}
