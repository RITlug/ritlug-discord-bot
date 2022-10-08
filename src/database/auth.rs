use rusqlite::{Connection, Result};

fn get_connection() -> Result<rusqlite::Connection, rusqlite::Error> {
  return Connection::open("database.db");
}

pub fn init() -> Result<()> {
  let sql = "
    CREATE TABLE IF NOT EXISTS auth_data (
      guild_id BIGINT NOT NULL,
      user_id BIGINT NOT NULL,
      email VARCHAR(320) NOT NULL,
      auth_date BITING NOT NULL,
      PRIMARY KEY (guild_id,user_id)
    );
  ";
  let conn = get_connection()?;
  conn.execute(sql, ())?;
  Ok(())
}

pub struct Row {
  pub guild_id: u64,
  pub user_id: u64,
  pub email: String,
  pub auth_date: u64
}

pub fn get_user(guild_id: &u64, user_id: &u64) -> Result<Option<Row>> {
  let sql = "SELECT * FROM auth_data WHERE guild_id=? AND user_id=?";
  let conn = get_connection()?;
  let mut stmt = conn.prepare(sql)?;
  let iter = stmt.query_map([guild_id, user_id], |row| {
    Ok(Row {
      guild_id: row.get(0)?,
      user_id: row.get(1)?,
      email: row.get(2)?,
      auth_date: row.get(3)?
    })
  })?;
  for row in iter {
    return Ok(Some(row?))
  }
  Ok(None)
}

pub fn set_user(guild_id: &u64, user_id: &u64, email: &str, auth_date: &u64) -> Result<()> {
  let sql = "REPLACE INTO auth_data (guild_id, user_id, email, auth_date) VALUES(?,?,?,?)";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &user_id, &email, &auth_date))?;
  Ok(())
}

pub fn delete_user(guild_id: &u64, user_id: &u64) -> Result<()> {
  let sql = "DELETE FROM auth_data WHERE guild_id = ? AND user_id = ?;";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &user_id))?;
  Ok(())
}