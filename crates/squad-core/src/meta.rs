use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, SquadError};

#[derive(Debug, Clone)]
pub struct FieldMeta {
    pub name: String,
    pub rangelow: i64,
}

#[derive(Debug, Clone)]
pub struct TableMeta {
    pub name: String,
    pub fields: HashMap<String, FieldMeta>,
}

pub type MetaMap = HashMap<String, TableMeta>;

fn parse_attrs(tag: &str) -> HashMap<&str, &str> {
    let bytes = tag.as_bytes();
    let mut attrs = HashMap::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if key_start == i || i >= bytes.len() || bytes[i] != b'=' {
            break;
        }
        let key = &tag[key_start..i];
        i += 1;
        if i >= bytes.len() || bytes[i] != b'"' {
            break;
        }
        i += 1;
        let val_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let val = &tag[val_start..i];
        i += 1;
        attrs.insert(key, val);
    }
    attrs
}

pub fn load_from_xml(path: &Path) -> Result<MetaMap> {
    let content = std::fs::read_to_string(path).map_err(SquadError::Io)?;
    parse_meta_xml(&content)
}

pub fn parse_meta_xml(content: &str) -> Result<MetaMap> {
    let mut tables = HashMap::new();
    let mut cursor = 0usize;

    while let Some(rel_start) = content[cursor..].find("<table ") {
        let tag_start = cursor + rel_start + "<table ".len();
        let Some(rel_tag_end) = content[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + rel_tag_end;
        let tag_str = &content[tag_start..tag_end];

        let Some(rel_body_end) = content[tag_end..].find("</table>") else {
            break;
        };
        let body_start = tag_end + 1;
        let body_end = tag_end + rel_body_end;
        let body = &content[body_start..body_end];
        cursor = body_end + "</table>".len();

        let table_attrs = parse_attrs(tag_str);
        let Some(&shortname) = table_attrs.get("shortname") else {
            continue;
        };
        let name = table_attrs
            .get("name")
            .copied()
            .unwrap_or_default()
            .to_string();

        let mut fields = HashMap::new();
        let mut fcursor = 0usize;
        while let Some(rel_fstart) = body[fcursor..].find("<field ") {
            let ftag_start = fcursor + rel_fstart + "<field ".len();
            let Some(rel_fend) = body[ftag_start..].find("/>") else {
                break;
            };
            let ftag_end = ftag_start + rel_fend;
            let field_tag = &body[ftag_start..ftag_end];
            fcursor = ftag_end + 2;

            let field_attrs = parse_attrs(field_tag);
            let Some(&field_shortname) = field_attrs.get("shortname") else {
                continue;
            };
            let field_name = field_attrs
                .get("name")
                .copied()
                .unwrap_or_default()
                .to_string();
            let rangelow = field_attrs
                .get("rangelow")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);
            fields.insert(
                field_shortname.to_string(),
                FieldMeta {
                    name: field_name,
                    rangelow,
                },
            );
        }

        tables.insert(shortname.to_string(), TableMeta { name, fields });
    }

    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_table() {
        let xml = r#"
            <table name="players" shortname="CZUM">
              <fields>
                <field name="playerid" shortname="ykFq" rangelow="0" rangehigh="500000" />
              </fields>
            </table>
        "#;
        let meta = parse_meta_xml(xml).unwrap();
        let table = meta.get("CZUM").unwrap();
        assert_eq!(table.name, "players");
        let field = table.fields.get("ykFq").unwrap();
        assert_eq!(field.name, "playerid");
        assert_eq!(field.rangelow, 0);
    }

    #[test]
    fn parses_multiple_tables_and_fields() {
        let xml = r#"
            <table name="teams" shortname="lyxL">
              <fields>
                <field name="teamid" shortname="AAAA" rangelow="0" />
                <field name="teamname" shortname="BBBB" rangelow="0" />
              </fields>
            </table>
            <table name="players" shortname="CZUM">
              <fields>
                <field name="playerid" shortname="ykFq" rangelow="0" />
              </fields>
            </table>
        "#;
        let meta = parse_meta_xml(xml).unwrap();
        assert_eq!(meta.len(), 2);
        assert_eq!(meta.get("lyxL").unwrap().fields.len(), 2);
        assert_eq!(meta.get("CZUM").unwrap().fields.len(), 1);
    }
}
