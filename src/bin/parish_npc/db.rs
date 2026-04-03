use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// Database connection for the parish-world.db
pub struct WorldDb {
    conn: Connection,
}

impl WorldDb {
    /// Opens or creates the world database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Creates necessary tables and schemas if they don't exist
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            -- Geographic hierarchy
            CREATE TABLE IF NOT EXISTS provinces (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS counties (
                id INTEGER PRIMARY KEY,
                province_id INTEGER NOT NULL REFERENCES provinces(id),
                name TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS baronies (
                id INTEGER PRIMARY KEY,
                county_id INTEGER NOT NULL REFERENCES counties(id),
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS parishes (
                id INTEGER PRIMARY KEY,
                barony_id INTEGER NOT NULL REFERENCES baronies(id),
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS townlands (
                id INTEGER PRIMARY KEY,
                parish_id INTEGER NOT NULL REFERENCES parishes(id),
                name TEXT NOT NULL,
                area_acres REAL
            );

            CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY,
                townland_id INTEGER NOT NULL REFERENCES townlands(id),
                name TEXT NOT NULL,
                description TEXT
            );

            -- Households
            CREATE TABLE IF NOT EXISTS households (
                id INTEGER PRIMARY KEY,
                townland_id INTEGER NOT NULL REFERENCES townlands(id),
                dwelling_type TEXT NOT NULL,
                land_acres REAL
            );

            -- NPCs
            CREATE TABLE IF NOT EXISTS npcs (
                id INTEGER PRIMARY KEY,
                data_tier INTEGER NOT NULL DEFAULT 0, -- 0: Sketched, 1: Elaborated, 2: Authored
                name TEXT NOT NULL,
                surname TEXT NOT NULL,
                sex TEXT NOT NULL,
                birth_year INTEGER NOT NULL,
                parish_id INTEGER NOT NULL REFERENCES parishes(id),
                townland_id INTEGER NOT NULL REFERENCES townlands(id),
                occupation TEXT,
                religion TEXT,
                social_class TEXT,
                household_id INTEGER REFERENCES households(id),

                -- Rich fields (nullable for Sketched)
                personality TEXT,
                mood TEXT,

                -- Runtime state
                current_location INTEGER REFERENCES locations(id),
                state TEXT DEFAULT 'alive'
            );

            -- Indexes for quick lookups
            CREATE INDEX IF NOT EXISTS idx_npcs_parish ON npcs(parish_id);
            CREATE INDEX IF NOT EXISTS idx_npcs_townland ON npcs(townland_id);
            CREATE INDEX IF NOT EXISTS idx_npcs_household ON npcs(household_id);
            CREATE INDEX IF NOT EXISTS idx_npcs_surname ON npcs(surname);

            -- Relationships (sparse adjacency list)
            CREATE TABLE IF NOT EXISTS relationships (
                id INTEGER PRIMARY KEY,
                from_npc_id INTEGER NOT NULL REFERENCES npcs(id),
                to_npc_id INTEGER NOT NULL REFERENCES npcs(id),
                kind TEXT NOT NULL,
                subkind TEXT,
                strength REAL NOT NULL,
                UNIQUE(from_npc_id, to_npc_id, kind)
            );

            -- Schedule Templates
            CREATE TABLE IF NOT EXISTS schedule_templates (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );

            -- Knowledge/Memories
            CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY,
                npc_id INTEGER NOT NULL REFERENCES npcs(id),
                topic TEXT NOT NULL,
                details TEXT NOT NULL,
                acquired_at_game_time TEXT
            );
            "#,
        ).context("Failed to run database migrations")?;

        Ok(())
    }

    pub fn get_conn(&self) -> &Connection {
        &self.conn
    }
}
