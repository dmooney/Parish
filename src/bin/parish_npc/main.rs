mod commands;
mod db;
mod generate;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "parish-npc", about = "CLI for managing scalable NPC data for Parish", version)]
struct Cli {
    /// Path to the world database
    #[arg(long, global = true, default_value = "parish-world.db")]
    db: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build the world DB for given counties
    GenerateWorld {
        /// Comma-separated list of counties
        #[arg(long, value_delimiter = ',')]
        counties: Vec<String>,
    },
    /// Seed one parish
    GenerateParish {
        /// Parish name
        parish: String,
        /// Target population
        #[arg(long)]
        pop: u32,
    },
    /// List NPCs based on criteria
    List {
        /// Parish name
        #[arg(long)]
        parish: String,
        /// Occupation to filter by
        #[arg(long)]
        occupation: Option<String>,
    },
    /// Show details for an NPC
    Show {
        /// NPC ID
        id: u32,
    },
    /// Search for NPCs
    Search {
        /// Query string (e.g. name or surname)
        query: String,
    },
    /// Edit an NPC's properties
    Edit {
        /// NPC ID
        id: u32,
        /// New mood for the NPC
        #[arg(long)]
        mood: Option<String>,
    },
    /// Promote a Sketched NPC to Elaborated
    Promote {
        /// NPC ID
        id: u32,
    },
    /// Batch LLM elaboration for NPCs in a parish
    Elaborate {
        /// Parish name
        #[arg(long)]
        parish: String,
        /// Number of NPCs to batch elaborate
        #[arg(long)]
        batch: u32,
    },
    /// Validate consistency of the NPC data
    Validate {
        /// Parish name
        #[arg(long)]
        parish: Option<String>,
        /// Validate the entire world
        #[arg(long)]
        all: bool,
    },
    /// Show population counts, tier distributions, etc.
    Stats,
    /// Export NPC data
    Export {
        /// Parish name
        #[arg(long)]
        parish: String,
    },
    /// Import NPC data from stdin
    Import,
    /// Show family tree for an NPC
    FamilyTree {
        /// NPC ID
        id: u32,
    },
    /// Show relationships for an NPC
    Relationships {
        /// NPC ID
        id: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = db::WorldDb::open(&cli.db)?;
    let conn = db.get_conn();

    match &cli.command {
        Commands::GenerateWorld { counties } => {
            generate::generate_world(conn, counties)?;
        }
        Commands::GenerateParish { parish, pop } => {
            generate::generate_parish(conn, parish, *pop)?;
        }
        Commands::List { parish, occupation } => {
            commands::list_npcs(conn, parish, occupation.as_ref())?;
        }
        Commands::Show { id } => {
            commands::show_npc(conn, *id)?;
        }
        Commands::Search { query } => {
            commands::search_npc(conn, query)?;
        }
        Commands::Edit { id, mood } => {
            commands::edit_npc(conn, *id, mood.as_ref())?;
        }
        Commands::Promote { id } => {
            commands::promote_npc(conn, *id)?;
        }
        Commands::Elaborate { parish, batch } => {
            commands::elaborate_parish(conn, parish, *batch)?;
        }
        Commands::Validate { parish, all } => {
            commands::validate_data(conn, parish.as_ref(), *all)?;
        }
        Commands::Stats => {
            commands::show_stats(conn)?;
        }
        Commands::Export { parish } => {
            commands::export_data(conn, parish)?;
        }
        Commands::Import => {
            commands::import_data(conn)?;
        }
        Commands::FamilyTree { id } => {
            commands::show_family_tree(conn, *id)?;
        }
        Commands::Relationships { id } => {
            commands::show_relationships(conn, *id)?;
        }
    }

    Ok(())
}
