use rusqlite::{Connection, Result};

fn get_connection() -> Result<rusqlite::Connection, rusqlite::Error> {
  return Connection::open("database.db");
}

pub fn init() -> Result<()> {
  let sql = "
    CREATE TABLE IF NOT EXISTS guild_data (
      guild_id BIGINT NOT NULL,
      key VARCHAR(64) NOT NULL,
      value VARCHAR(64) NOT NULL,
      PRIMARY KEY (guild_id, key)
    );
  ";
  let conn = get_connection()?;
  conn.execute(sql, ())?;
  Ok(())
}

struct Row {
  guild_id: u64,
  key: String,
  value: String,
}

pub fn get_setting(guild_id: &u64, key: &str) -> Result<Option<String>> {
  let sql = "SELECT * FROM guild_data WHERE guild_id=? AND key=?";
  let conn = get_connection()?;
  let mut stmt = conn.prepare(sql)?;
  let iter = stmt.query_map((guild_id, key), |row| {
    Ok(Row {
      guild_id: row.get(0)?,
      key: row.get(1)?,
      value: row.get(2)?
    })
  })?;
  for row in iter {
    return Ok(Some(row?.value))
  }
  Ok(None)
}

pub fn set_setting(guild_id: &u64, key: &str, value: &str) -> Result<()> {
  let sql = "REPLACE INTO guild_data (guild_id, key, value) VALUES(?,?,?)";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &key, &value))?;
  Ok(())
}

pub fn delete_setting(guild_id: &u64, key: &str) -> Result<()> {
  let sql = "DELETE FROM guild_data WHERE guild_id = ? AND key = ?;";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &key))?;
  Ok(())
}