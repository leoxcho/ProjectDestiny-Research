use anyhow::{Context, Result};
use clap::Parser;
use destiny_parser::parse_file;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "destiny-index",
    about = "Index extracted Destiny Tiger definitions"
)]
struct Args {
    input: PathBuf,
    #[arg(short, long, default_value = "destiny.db")]
    output: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();
    let files: Vec<_> = WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::with_template("{bar:40} {pos}/{len} {msg}")?);
    let mut db = Database::open(&args.output)?;
    for entry in files {
        pb.set_message(entry.path().display().to_string());
        match parse_file(entry.path()) {
            Ok(parsed) => db.insert(entry.path(), &parsed)?,
            Err(e) => warn!(path=%entry.path().display(), error=%e, "skipping unreadable file"),
        }
        pb.inc(1);
    }
    pb.finish_with_message("complete");
    info!(output=%args.output.display(), "index created");
    Ok(())
}

struct Database {
    conn: Connection,
}
impl Database {
    fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("open {}", path.display()))?;
        conn.execute_batch(include_str!("../../../migrations/001_init.sql"))?;
        Ok(Self { conn })
    }
    fn insert(&mut self, path: &Path, p: &destiny_parser::ParsedFile) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO files(path,signature,confidence,sha256,size) VALUES(?1,?2,?3,?4,?5)",
            params![
                path.to_string_lossy(),
                format!("{:?}", p.signature),
                p.confidence,
                p.hash,
                p.size
            ],
        )?;
        let id = tx.last_insert_rowid();
        for f in &p.fields {
            tx.execute(
                "INSERT INTO fields(file_id,kind,offset,size,value) VALUES(?1,?2,?3,?4,?5)",
                params![id, f.kind, f.offset, f.size, f.value],
            )?;
        }
        for (offset, text) in &p.strings {
            tx.execute(
                "INSERT INTO strings(file_id,offset,value) VALUES(?1,?2,?3)",
                params![id, offset, text],
            )?;
        }
        for (offset, hash) in &p.references {
            tx.execute(
                "INSERT INTO references_(file_id,offset,target_hash) VALUES(?1,?2,?3)",
                params![id, offset, hash],
            )?;
        }
        for (offset, tag) in &p.tag_identifiers {
            tx.execute("INSERT INTO fields(file_id,kind,offset,size,value) VALUES(?1,'tag_identifier',?2,4,?3)", params![id, offset, tag])?;
        }
        if let Some(type_info) = &p.type_information {
            tx.execute("INSERT INTO fields(file_id,kind,offset,size,value) VALUES(?1,'type_information',4,4,?2)", params![id, type_info])?;
        }
        for warning in &p.warnings {
            tx.execute(
                "INSERT INTO warnings(file_id,message) VALUES(?1,?2)",
                params![id, warning],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_creates_schema() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        assert!(db.conn.prepare("SELECT 1 FROM files").is_ok());
    }
}
