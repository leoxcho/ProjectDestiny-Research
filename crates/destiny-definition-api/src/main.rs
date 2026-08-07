use anyhow::Result;
use clap::Parser;
use destiny_definition_api::serve;
use destiny_runtime_core::Runtime;
use std::{path::PathBuf, sync::Arc};
#[derive(Parser)]
struct A {
    #[arg(short, long, default_value = "destiny.db")]
    database: PathBuf,
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    listen: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    let a = A::parse();
    serve(Arc::new(Runtime::open(a.database)?), &a.listen).await
}
