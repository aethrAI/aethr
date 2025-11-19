use crate::Result;
use crate::db::moat::{CommunityMoat, MoatEntry};
use crate::utils::config;
use crate::ui::Status;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use colored::*;

#[derive(Debug, Deserialize)]
struct SeedEntry {
    command: String,
    context_tags: Option<Vec<String>>,
    success_score: Option<f32>,
    provenance: Option<String>,
}

pub fn run() -> Result<()> {
    Status::pending("Loading community moat database...");
    
    // data/seed_moat.json (relative to project root) or in current working dir
    let path = std::path::Path::new("data/seed_moat.json");
    if !path.exists() {
        Status::error("Seed file not found at data/seed_moat.json");
        return Ok(());
    }
    
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let seeds: Vec<SeedEntry> = serde_json::from_reader(reader)?;

    let db_path = config::get_db_path();
    let moat = CommunityMoat::new(&db_path)?;
    
    Status::pending(format!("Seeding {} entries into community moat...", seeds.len()));
    
    let mut inserted = 0usize;
    for (idx, s) in seeds.iter().enumerate() {
        let tags_json = s.context_tags.as_ref().map(|v| serde_json::to_string(&v).unwrap());
        let entry = MoatEntry {
            id: None,
            command: s.command.clone(),
            context_tags: tags_json,
            success_score: s.success_score,
            provenance: s.provenance.clone(),
            created_at: None,
        };
        moat.insert(entry)?;
        inserted += 1;
        
        if idx % 10 == 0 && idx > 0 {
            print!("\r");
            Status::pending(format!("Seeding {} entries into community moat... ({}/{})", 
                seeds.len(), idx, seeds.len()));
        }
    }

    Status::success(format!("Seeded {} community moat entries", inserted));
    println!("{}", format!("📁 Location: {}", db_path.display()).bright_black());
    Ok(())
}