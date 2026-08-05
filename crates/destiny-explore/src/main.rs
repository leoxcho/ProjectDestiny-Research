use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name="destiny-explore", about="Explore a Project Destiny definition database")]
struct Cli { #[arg(short, long, default_value="destiny.db", global=true)] database: PathBuf, #[command(subcommand)] command: Command }
#[derive(Subcommand)]
enum Command { Stats, Search { query: String }, Refs { hash: String }, Dump { tag: String } }

fn main() -> Result<()> { let cli = Cli::parse(); let db = Connection::open(&cli.database).with_context(|| format!("open {}", cli.database.display()))?; match cli.command { Command::Stats => stats(&db)?, Command::Search { query } => search(&db, &query)?, Command::Refs { hash } => refs(&db, &hash)?, Command::Dump { tag } => dump(&db, &tag)? }; Ok(()) }

fn stats(db: &Connection) -> Result<()> {
    println!("files\t{}", scalar(db, "select count(*) from files")?);
    println!("fields\t{}", scalar(db, "select count(*) from fields")?);
    println!("references\t{}", scalar(db, "select count(*) from references_")?);
    println!("strings\t{}", scalar(db, "select count(*) from strings")?);
    println!("\nclassifications");
    let mut stmt = db.prepare("select id,path from files")?; let mut counts = std::collections::BTreeMap::new();
    for row in stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?)))? { let (id,path)=row?; let c=classify(db,id,&path)?; *counts.entry(c).or_insert(0usize)+=1; }
    for (c,n) in counts { println!("{c}\t{n}"); }
    Ok(())
}

fn search(db: &Connection, query: &str) -> Result<()> {
    let pattern = format!("%{}%", query.to_lowercase());
    let mut stmt = db.prepare("select distinct f.id,f.path from files f left join strings s on s.file_id=f.id left join fields x on x.file_id=f.id where lower(s.value) like ?1 or lower(x.value) like ?1 or lower(f.path) like ?1 order by f.path limit 100")?;
    let rows = stmt.query_map(params![pattern], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))?;
    for row in rows { let (id,path)=row?; println!("{}\t{}\t{}", classify(db,id,&path)?, id, path); } Ok(())
}

fn refs(db: &Connection, hash: &str) -> Result<()> {
    let h = normalize_hash(hash); let mut stmt=db.prepare("select f.id,f.path,r.offset from references_ r join files f on f.id=r.file_id where lower(r.target_hash)=lower(?1) order by f.path")?;
    for row in stmt.query_map(params![h], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))? { let (id,path,offset)=row?; println!("{}\t{}\t{}\t{}", classify(db,id,&path)?, id, offset, path); } Ok(())
}

fn dump(db: &Connection, tag: &str) -> Result<()> {
    let tag = normalize_hash(tag); let row: Option<(i64,String,String,i64,String)> = db.query_row("select f.id,f.path,f.sha256,f.size,f.signature from files f join fields x on x.file_id=f.id where lower(x.value)=lower(?1) and x.kind='tag_identifier' limit 1", params![tag], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
    let Some((id,path,sha,size,sig))=row else { println!("no definition found for {tag}"); return Ok(()); };
    println!("id\t{id}\npath\t{path}\nsignature\t{sig}\nsize\t{size}\nsha256\t{sha}\nclassification\t{}", classify(db,id,&path)?);
    println!("\nfields"); let mut s=db.prepare("select kind,offset,size,value from fields where file_id=?1 order by offset,kind")?; for r in s.query_map(params![id], |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,Option<String>>(3)?)))? { let (k,o,z,v)=r?; println!("{k}\t{o}\t{z}\t{}",v.unwrap_or_default()); }
    println!("\nstrings"); let mut s=db.prepare("select offset,value from strings where file_id=?1 order by offset")?; for r in s.query_map(params![id], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))? { let (o,v)=r?; println!("{o}\t{v}"); } Ok(())
}

fn classify(db:&Connection,id:i64,path:&str)->Result<String>{
    // This is deliberately conservative: the current extraction contains no
    // authoritative Destiny schema-name table. A category is emitted only
    // when a semantic token is present in a path/string/field; type IDs and
    // relationship counts remain searchable evidence, not invented labels.
    let mut text=path.to_lowercase();
    let mut s=db.prepare("select value from strings where file_id=?1 union all select value from fields where file_id=?1")?;
    for r in s.query_map(params![id],|r|r.get::<_,Option<String>>(0))? { if let Some(v)=r? { text.push(' '); text.push_str(&v.to_lowercase()); } }
    for key in ["weapon","armor","activity","quest","perk","location","vendor","character","item"] { if text.contains(key) { return Ok(key.to_string()); } }
    Ok("unknown".into())
}
fn scalar(db:&Connection,sql:&str)->Result<i64>{Ok(db.query_row(sql,[],|r|r.get(0))?)}
fn normalize_hash(h:&str)->String{let h=h.trim().to_lowercase(); if h.starts_with("0x"){h}else{format!("0x{h}")}}

#[cfg(test)]
mod tests { use super::*; #[test] fn hash_normalization(){assert_eq!(normalize_hash("80800184"),"0x80800184"); assert_eq!(normalize_hash("0x80800184"),"0x80800184");} #[test] fn classification_uses_path(){let db=Connection::open_in_memory().unwrap(); db.execute_batch("create table strings(file_id integer,value text); create table fields(file_id integer,value text); create table files(id integer,path text);").unwrap(); db.execute("insert into files values(1,'/weapons/foo')",[]).unwrap(); assert_eq!(classify(&db,1,"/weapons/foo").unwrap(),"weapon");} }
