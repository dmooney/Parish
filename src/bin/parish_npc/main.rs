use clap::{Parser, Subcommand};

/// Tool for managing and generating Parish NPCs
#[derive(Parser, Debug)]
#[command(name = "parish-npc", about = "CLI for managing scalable NPC data for Parish", version)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenerateWorld { counties } => {
            println!("Generating world for counties: {:?}", counties);
        }
        Commands::GenerateParish { parish, pop } => {
            println!("Generating parish {} with population {}", parish, pop);
        }
        Commands::List { parish, occupation } => {
            println!("Listing NPCs in parish: {}", parish);
            if let Some(occ) = occupation {
                println!("Filtering by occupation: {}", occ);
            }
        }
        Commands::Show { id } => {
            println!("Showing NPC with ID: {}", id);
        }
        Commands::Search { query } => {
            println!("Searching for: {}", query);
        }
        Commands::Edit { id, mood } => {
            println!("Editing NPC {}", id);
            if let Some(m) = mood {
                println!("Setting mood to: {}", m);
            }
        }
        Commands::Promote { id } => {
            println!("Promoting NPC {}", id);
        }
        Commands::Elaborate { parish, batch } => {
            println!("Elaborating {} NPCs in parish {}", batch, parish);
        }
        Commands::Validate { parish, all } => {
            if *all {
                println!("Validating entire world consistency");
            } else if let Some(p) = parish {
                println!("Validating parish: {}", p);
            } else {
                println!("Must specify --parish or --all");
            }
        }
        Commands::Stats => {
            println!("Displaying stats");
        }
        Commands::Export { parish } => {
            println!("Exporting data for parish: {}", parish);
        }
        Commands::Import => {
            println!("Importing data");
        }
        Commands::FamilyTree { id } => {
            println!("Showing family tree for NPC {}", id);
        }
        Commands::Relationships { id } => {
            println!("Showing relationships for NPC {}", id);
        }
    }
}
