use rusqlite::Connection;
use anyhow::Result;
use rand::Rng;

pub fn generate_world(conn: &Connection, counties: &[String]) -> Result<()> {
    println!("Generating geographic hierarchy for counties: {:?}", counties);

    // Dummy logic to insert province and counties
    conn.execute(
        "INSERT OR IGNORE INTO provinces (id, name) VALUES (1, 'Connacht')",
        [],
    )?;

    for (i, county) in counties.iter().enumerate() {
        let county_id = i + 1;
        conn.execute(
            "INSERT OR IGNORE INTO counties (id, province_id, name) VALUES (?1, 1, ?2)",
            rusqlite::params![county_id, county],
        )?;

        // Give each county a dummy barony and parish so we can use them later
        conn.execute(
            "INSERT OR IGNORE INTO baronies (id, county_id, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![county_id, county_id, format!("Barony of {}", county)],
        )?;

        // This makes sure we have at least some dummy parishes
        conn.execute(
            "INSERT OR IGNORE INTO parishes (id, barony_id, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![county_id, county_id, format!("Default Parish of {}", county)],
        )?;
    }

    Ok(())
}

pub fn generate_parish(conn: &Connection, parish_name: &str, target_pop: u32) -> Result<()> {
    println!("Seeding parish '{}' with population {}", parish_name, target_pop);

    // Ensure parish exists
    let parish_id: i64 = conn.query_row(
        "SELECT id FROM parishes WHERE name = ?1",
        rusqlite::params![parish_name],
        |row| row.get(0),
    ).unwrap_or_else(|_| {
        conn.execute(
            "INSERT INTO parishes (barony_id, name) VALUES (1, ?1)",
            rusqlite::params![parish_name],
        ).unwrap();
        conn.last_insert_rowid()
    });

    // Create a default townland
    let townland_name = format!("Townland of {}", parish_name);
    let townland_id: i64 = conn.query_row(
        "SELECT id FROM townlands WHERE name = ?1",
        rusqlite::params![townland_name],
        |row| row.get(0),
    ).unwrap_or_else(|_| {
        conn.execute(
            "INSERT INTO townlands (parish_id, name, area_acres) VALUES (?1, ?2, 100.0)",
            rusqlite::params![parish_id, townland_name],
        ).unwrap();
        conn.last_insert_rowid()
    });

    let mut rng = rand::thread_rng();

    // Generate households and NPCs
    let avg_household_size = 5;
    let num_households = target_pop / avg_household_size;

    for _ in 0..num_households {
        conn.execute(
            "INSERT INTO households (townland_id, dwelling_type, land_acres) VALUES (?1, 'cottage', ?2)",
            rusqlite::params![townland_id, rng.gen_range(5.0..20.0)],
        )?;
        let household_id = conn.last_insert_rowid();

        // Generate head of household
        conn.execute(
            "INSERT INTO npcs (data_tier, name, surname, sex, birth_year, parish_id, townland_id, occupation, religion, social_class, household_id)
             VALUES (0, 'Pádraig', 'Darcy', 'M', ?1, ?2, ?3, 'Farmer', 'Catholic', 'Tenant', ?4)",
            rusqlite::params![
                1820 - rng.gen_range(28..60),
                parish_id,
                townland_id,
                household_id
            ],
        )?;

        let head_id = conn.last_insert_rowid();

        // Generate spouse
        conn.execute(
            "INSERT INTO npcs (data_tier, name, surname, sex, birth_year, parish_id, townland_id, occupation, religion, social_class, household_id)
             VALUES (0, 'Mary', 'Darcy', 'F', ?1, ?2, ?3, 'Farmer', 'Catholic', 'Tenant', ?4)",
            rusqlite::params![
                1820 - rng.gen_range(25..55),
                parish_id,
                townland_id,
                household_id
            ],
        )?;
        let spouse_id = conn.last_insert_rowid();

        // Relationship
        conn.execute(
            "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'spouse', 0.8)",
            rusqlite::params![head_id, spouse_id],
        )?;
        conn.execute(
            "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'spouse', 0.8)",
            rusqlite::params![spouse_id, head_id],
        )?;

        // Children
        let num_children = rng.gen_range(0..8);
        for _ in 0..num_children {
            let birth_year = 1820 - rng.gen_range(0..20);
            let (name, sex) = if rng.gen_bool(0.5) { ("Seán", "M") } else { ("Bridget", "F") };

            conn.execute(
                "INSERT INTO npcs (data_tier, name, surname, sex, birth_year, parish_id, townland_id, occupation, religion, social_class, household_id)
                 VALUES (0, ?1, 'Darcy', ?2, ?3, ?4, ?5, 'None', 'Catholic', 'Tenant', ?6)",
                rusqlite::params![
                    name, sex, birth_year, parish_id, townland_id, household_id
                ],
            )?;
            let child_id = conn.last_insert_rowid();

            conn.execute(
                "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'parent', 0.9)",
                rusqlite::params![head_id, child_id],
            )?;
            conn.execute(
                "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'parent', 0.9)",
                rusqlite::params![spouse_id, child_id],
            )?;
            conn.execute(
                "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'child', 0.9)",
                rusqlite::params![child_id, head_id],
            )?;
            conn.execute(
                "INSERT INTO relationships (from_npc_id, to_npc_id, kind, strength) VALUES (?1, ?2, 'child', 0.9)",
                rusqlite::params![child_id, spouse_id],
            )?;
        }
    }

    Ok(())
}
