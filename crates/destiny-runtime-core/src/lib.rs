//! Read-only definition access. This crate does not reinterpret unknown binary layouts.
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub kind: String,
    pub offset: i64,
    pub size: i64,
    pub value: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub offset: i64,
    pub target_hash: Option<String>,
    pub target_offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub id: i64,
    pub path: String,
    pub signature: String,
    pub confidence: i64,
    pub sha256: String,
    pub size: i64,
    pub fields: Vec<Field>,
    pub references: Vec<Reference>,
    pub strings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDefinition {
    pub metadata: Definition,
    pub bytes: Vec<u8>,
}

pub struct Runtime {
    db: Mutex<Connection>,
    cache: Mutex<HashMap<String, Definition>>,
    raw_root: Option<PathBuf>,
}
impl Runtime {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            db: Mutex::new(Connection::open(path.as_ref())?),
            cache: Mutex::new(HashMap::new()),
            raw_root: None,
        })
    }
    pub fn with_raw_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.raw_root = Some(root.into());
        self
    }
    pub fn get_definition(&self, hash: &str) -> Result<Option<Definition>> {
        let key = normalize(hash);
        if let Some(v) = self.cache.lock().unwrap().get(&key).cloned() {
            return Ok(Some(v));
        }
        let db = self.db.lock().unwrap();
        let row:Option<(i64,String,String,i64,String,i64)>=db.query_row("select id,path,signature,confidence,sha256,size from files where lower(sha256)=lower(?1) or id in (select file_id from fields where kind='tag_identifier' and lower(value)=lower(?1)) limit 1",params![key],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?;
        let Some((id, path, sig, conf, sha, size)) = row else {
            return Ok(None);
        };
        let fields = load_fields(&db, id)?;
        let references = load_refs(&db, id)?;
        let strings = load_strings(&db, id)?;
        let v = Definition {
            id,
            path,
            signature: sig,
            confidence: conf,
            sha256: sha,
            size,
            fields,
            references,
            strings,
        };
        self.cache.lock().unwrap().insert(key, v.clone());
        Ok(Some(v))
    }
    pub fn get_references(&self, hash: &str) -> Result<Vec<Reference>> {
        let db = self.db.lock().unwrap();
        let mut s=db.prepare("select offset,target_hash,target_offset from references_ where lower(target_hash)=lower(?1)")?;
        let rows = s
            .query_map(params![normalize(hash)], |r| {
                Ok(Reference {
                    offset: r.get(0)?,
                    target_hash: r.get(1)?,
                    target_offset: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    pub fn get_definition_raw(&self, hash: &str) -> Result<Option<RawDefinition>> {
        let Some(def) = self.get_definition(hash)? else {
            return Ok(None);
        };
        let p = PathBuf::from(&def.path);
        let p = self.raw_root.as_ref().map(|r| r.join(&p)).unwrap_or(p);
        let bytes =
            std::fs::read(&p).with_context(|| format!("read preserved payload {}", p.display()))?;
        Ok(Some(RawDefinition {
            metadata: def,
            bytes,
        }))
    }
    pub fn stats(&self) -> Result<(i64, i64, i64, i64)> {
        let d = self.db.lock().unwrap();
        Ok((
            scalar(&d, "select count(*) from files")?,
            scalar(&d, "select count(*) from fields")?,
            scalar(&d, "select count(*) from references_")?,
            scalar(&d, "select count(*) from strings")?,
        ))
    }
}
fn load_fields(d: &Connection, id: i64) -> Result<Vec<Field>> {
    let mut s =
        d.prepare("select kind,offset,size,value from fields where file_id=?1 order by offset")?;
    let rows = s
        .query_map(params![id], |r| {
            Ok(Field {
                kind: r.get(0)?,
                offset: r.get(1)?,
                size: r.get(2)?,
                value: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
fn load_refs(d: &Connection, id: i64) -> Result<Vec<Reference>> {
    let mut s = d.prepare(
        "select offset,target_hash,target_offset from references_ where file_id=?1 order by offset",
    )?;
    let rows = s
        .query_map(params![id], |r| {
            Ok(Reference {
                offset: r.get(0)?,
                target_hash: r.get(1)?,
                target_offset: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
fn load_strings(d: &Connection, id: i64) -> Result<Vec<String>> {
    let mut s = d.prepare("select value from strings where file_id=?1 order by offset")?;
    let rows = s
        .query_map(params![id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
fn scalar(d: &Connection, q: &str) -> Result<i64> {
    Ok(d.query_row(q, [], |r| r.get(0))?)
}
fn normalize(v: &str) -> String {
    let v = v.trim().to_lowercase();
    if v.starts_with("0x") {
        v
    } else {
        format!("0x{v}")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalize_hash() {
        assert_eq!(normalize("80800184"), "0x80800184");
    }
}
