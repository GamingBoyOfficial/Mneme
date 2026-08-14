use clap::{Parser, Subcommand};
use mneme_core::{MemoryStore, SqliteBackend, HashEmbedder, Embedder, MemoryType, AccessScope};
use std::sync::Arc;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "mneme")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Export { archive_path: String, #[arg(long, default_value = "mneme.db")] db: String },
    Import { archive_path: String, #[arg(long, default_value = "mneme.db")] db: String },
    Diff { left: String, right: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Export { archive_path, db } => {
            let backend = SqliteBackend::new(&db).await?;
            let embedder = Arc::new(HashEmbedder::new(384));
            let store = MemoryStore::new("cli", Arc::new(backend), embedder).await;
            store.export(&archive_path).await?;
            println!("Exported to {}", archive_path);
        }
        Commands::Import { archive_path, db } => {
            let backend = SqliteBackend::new(&db).await?;
            let embedder = Arc::new(HashEmbedder::new(384));
            let store = MemoryStore::new("cli", Arc::new(backend), embedder).await;
            let count = store.import_from(&archive_path).await?;
            println!("Imported {} memories", count);
        }
        Commands::Diff { left, right } => {
            let left_archive = mneme_core::portability::import(&left)?;
            let right_archive = mneme_core::portability::import(&right)?;
            let left_count = left_archive.records.len();
            let right_count = right_archive.records.len();
            println!("Left: {} memories, Right: {} memories", left_count, right_count);

            let left_ids: std::collections::HashSet<_> = left_archive.records.iter().map(|r| r.id.clone()).collect();
            let right_ids: std::collections::HashSet<_> = right_archive.records.iter().map(|r| r.id.clone()).collect();
            let only_left = left_ids.difference(&right_ids).count();
            let only_right = right_ids.difference(&left_ids).count();
            println!("Only in left: {}, Only in right: {}", only_left, only_right);
        }
    }
    Ok(())
}