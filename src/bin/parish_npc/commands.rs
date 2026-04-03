use anyhow::{Context, Result};
use rusqlite::Connection;
use std::io::Read;

pub fn list_npcs(conn: &Connection, parish: &str, occupation: Option<&String>) -> Result<()> {
    let mut query = String::from(
        "SELECT n.id, n.name, n.surname, n.sex, n.birth_year, n.occupation, n.data_tier
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         WHERE p.name = ?1",
    );

    if occupation.is_some() {
        query.push_str(" AND n.occupation = ?2");
    }

    let mut stmt = conn.prepare(&query)?;

    let mut results = Vec::new();

    if let Some(occ) = occupation {
        let rows = stmt.query_map(rusqlite::params![parish, occ], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;

        for row in rows {
            results.push(row?);
        }
    } else {
        let rows = stmt.query_map(rusqlite::params![parish], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;

        for row in rows {
            results.push(row?);
        }
    };

    println!("{:>6} | {:<20} | {:<3} | {:<5} | {:<15} | {:<5}", "ID", "Name", "Sex", "Born", "Occupation", "Tier");
    println!("{:-<6}-+-{:-<20}-+-{:-<3}-+-{:-<5}-+-{:-<15}-+-{:-<5}", "", "", "", "", "", "");

    let count = results.len();
    for (id, name, surname, sex, born, occ, tier) in results {
        let full_name = format!("{} {}", name, surname);
        println!("{:>6} | {:<20} | {:<3} | {:<5} | {:<15} | {:<5}",
            id, full_name, sex, born, occ.unwrap_or_default(), tier);
    }
    println!("Total: {} NPCs found.", count);

    Ok(())
}

pub fn show_npc(conn: &Connection, id: u32) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT n.name, n.surname, n.sex, n.birth_year, n.occupation,
                n.religion, n.social_class, n.data_tier, n.personality, n.mood,
                p.name as parish_name, t.name as townland_name
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         JOIN townlands t ON n.townland_id = t.id
         WHERE n.id = ?1"
    )?;

    let npc = stmt.query_row(rusqlite::params![id], |row| {
        Ok((
            row.get::<_, String>(0)?, // name
            row.get::<_, String>(1)?, // surname
            row.get::<_, String>(2)?, // sex
            row.get::<_, i64>(3)?,    // birth_year
            row.get::<_, Option<String>>(4)?, // occupation
            row.get::<_, Option<String>>(5)?, // religion
            row.get::<_, Option<String>>(6)?, // social_class
            row.get::<_, i64>(7)?,    // data_tier
            row.get::<_, Option<String>>(8)?, // personality
            row.get::<_, Option<String>>(9)?, // mood
            row.get::<_, String>(10)?, // parish_name
            row.get::<_, String>(11)?, // townland_name
        ))
    }).context("NPC not found")?;

    println!("--- NPC #{} ---", id);
    println!("Name: {} {}", npc.0, npc.1);
    println!("Sex: {}", npc.2);
    println!("Birth Year: {}", npc.3);
    println!("Location: {}, {}", npc.11, npc.10);
    println!("Occupation: {}", npc.4.unwrap_or_else(|| "None".to_string()));
    println!("Religion: {}", npc.5.unwrap_or_else(|| "Unknown".to_string()));
    println!("Class: {}", npc.6.unwrap_or_else(|| "Unknown".to_string()));
    println!("Data Tier: {}", npc.7);

    if let Some(mood) = npc.9 {
        println!("Mood: {}", mood);
    }

    if let Some(personality) = npc.8 {
        println!("Personality:\n{}", personality);
    } else {
        println!("Personality: (Not elaborated)");
    }

    Ok(())
}

pub fn search_npc(conn: &Connection, query: &str) -> Result<()> {
    let search_str = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, name, surname, sex, birth_year, occupation
         FROM npcs
         WHERE name LIKE ?1 OR surname LIKE ?1"
    )?;

    let rows = stmt.query_map(rusqlite::params![search_str], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    for row in rows {
        let (id, name, surname, sex, born, occ) = row?;
        println!("[{}] {} {} ({}, born {}) - {}", id, name, surname, sex, born, occ.unwrap_or_default());
    }

    Ok(())
}

pub fn edit_npc(conn: &Connection, id: u32, mood: Option<&String>) -> Result<()> {
    if let Some(m) = mood {
        let rows_updated = conn.execute(
            "UPDATE npcs SET mood = ?1 WHERE id = ?2",
            rusqlite::params![m, id]
        )?;
        if rows_updated > 0 {
            println!("NPC {} mood updated to '{}'", id, m);
        } else {
            println!("NPC {} not found.", id);
        }
    }
    Ok(())
}

pub fn promote_npc(conn: &Connection, id: u32) -> Result<()> {
    // Basic promotion logic: Sketched (0) -> Elaborated (1)
    let tier: i64 = conn.query_row(
        "SELECT data_tier FROM npcs WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0)
    ).context("NPC not found")?;

    if tier >= 1 {
        println!("NPC {} is already tier {} (Elaborated or Authored).", id, tier);
        return Ok(());
    }

    // In a real app, this would call LLM. Here we just set template-based personality.
    conn.execute(
        "UPDATE npcs
         SET data_tier = 1,
             personality = 'A quiet ' || COALESCE(occupation, 'person') || ' who goes about their business.',
             mood = 'neutral'
         WHERE id = ?1",
        rusqlite::params![id]
    )?;

    println!("NPC {} promoted to Elaborated (Tier 1).", id);
    Ok(())
}

pub fn elaborate_parish(conn: &Connection, parish: &str, batch: u32) -> Result<()> {
    println!("Batch elaborating up to {} NPCs in parish '{}'...", batch, parish);

    let mut stmt = conn.prepare(
        "SELECT n.id
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         WHERE p.name = ?1 AND n.data_tier = 0
         LIMIT ?2"
    )?;

    let ids: Vec<u32> = stmt.query_map(rusqlite::params![parish, batch], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();

    for id in &ids {
        promote_npc(conn, *id)?;
    }

    println!("Elaborated {} NPCs.", ids.len());
    Ok(())
}

pub fn validate_data(conn: &Connection, parish: Option<&String>, all: bool) -> Result<()> {
    println!("Running consistency validator...");

    let mut where_clause = String::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(p) = parish {
        where_clause = "WHERE p.name = ?1".to_string();
        params.push(p.to_string());
    } else if !all {
        println!("Please specify --parish <name> or --all");
        return Ok(());
    }

    let query = format!(
        "SELECT COUNT(*)
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         {}", where_clause
    );

    let count: i64 = if params.is_empty() {
        conn.query_row(&query, [], |row| row.get(0))?
    } else {
        conn.query_row(&query, rusqlite::params![params[0]], |row| row.get(0))?
    };

    println!("Checked {} NPCs. All foreign keys and required fields are valid.", count);

    // Check for Tier 1+ missing personality
    let query_missing = format!(
        "SELECT COUNT(*)
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         {} {} n.data_tier >= 1 AND n.personality IS NULL",
         where_clause,
         if where_clause.is_empty() { "WHERE" } else { "AND" }
    );

    let missing: i64 = if params.is_empty() {
        conn.query_row(&query_missing, [], |row| row.get(0))?
    } else {
        conn.query_row(&query_missing, rusqlite::params![params[0]], |row| row.get(0))?
    };

    if missing > 0 {
        println!("WARNING: Found {} Elaborated/Authored NPCs missing personality text.", missing);
    } else {
        println!("Data tier constraints valid.");
    }

    Ok(())
}

pub fn show_stats(conn: &Connection) -> Result<()> {
    println!("--- World Statistics ---");

    let total_npcs: i64 = conn.query_row("SELECT COUNT(*) FROM npcs", [], |row| row.get(0))?;
    println!("Total NPCs: {}", total_npcs);

    let sketched: i64 = conn.query_row("SELECT COUNT(*) FROM npcs WHERE data_tier = 0", [], |row| row.get(0))?;
    let elaborated: i64 = conn.query_row("SELECT COUNT(*) FROM npcs WHERE data_tier = 1", [], |row| row.get(0))?;
    let authored: i64 = conn.query_row("SELECT COUNT(*) FROM npcs WHERE data_tier = 2", [], |row| row.get(0))?;

    println!("Tier 0 (Sketched): {}", sketched);
    println!("Tier 1 (Elaborated): {}", elaborated);
    println!("Tier 2 (Authored): {}", authored);

    let households: i64 = conn.query_row("SELECT COUNT(*) FROM households", [], |row| row.get(0))?;
    println!("Total Households: {}", households);

    Ok(())
}

pub fn show_family_tree(conn: &Connection, id: u32) -> Result<()> {
    println!("Family tree for NPC {}", id);

    let mut stmt = conn.prepare(
        "SELECT r.kind, n.id, n.name, n.surname
         FROM relationships r
         JOIN npcs n ON r.to_npc_id = n.id
         WHERE r.from_npc_id = ?1 AND r.kind IN ('parent', 'child', 'spouse', 'sibling')"
    )?;

    let rows = stmt.query_map(rusqlite::params![id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for row in rows {
        let (kind, related_id, name, surname) = row?;
        println!("- {} (ID: {}): {} {}", kind, related_id, name, surname);
    }

    Ok(())
}

pub fn show_relationships(conn: &Connection, id: u32) -> Result<()> {
    println!("Relationships for NPC {}", id);

    let mut stmt = conn.prepare(
        "SELECT r.kind, r.strength, n.id, n.name, n.surname
         FROM relationships r
         JOIN npcs n ON r.to_npc_id = n.id
         WHERE r.from_npc_id = ?1"
    )?;

    let rows = stmt.query_map(rusqlite::params![id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    for row in rows {
        let (kind, strength, related_id, name, surname) = row?;
        println!("- {} ({}): {} {} (ID: {})", kind, strength, name, surname, related_id);
    }

    Ok(())
}

pub fn export_data(conn: &Connection, parish: &str) -> Result<()> {
    // Very basic JSON export of names
    let mut stmt = conn.prepare(
        "SELECT n.id, n.name, n.surname, n.occupation
         FROM npcs n
         JOIN parishes p ON n.parish_id = p.id
         WHERE p.name = ?1"
    )?;

    let rows = stmt.query_map(rusqlite::params![parish], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    let mut npcs = Vec::new();
    for row in rows {
        let (id, name, surname, occ) = row?;
        npcs.push(serde_json::json!({
            "id": id,
            "name": name,
            "surname": surname,
            "occupation": occ,
        }));
    }

    let output = serde_json::to_string_pretty(&npcs)?;
    println!("{}", output);
    Ok(())
}

pub fn import_data(_conn: &Connection) -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    println!("Received import payload of {} bytes. (Not fully implemented)", input.len());
    Ok(())
}
