use rusqlite::{Connection, Result};

fn get_connection() -> Result<rusqlite::Connection, rusqlite::Error> {
  return Connection::open("database.db");
}

pub fn init() -> Result<()> {
  let sql = "
    CREATE TABLE IF NOT EXISTS role_data (
      guild_id BIGINT NOT NULL,
      page INT NOT NULL,
      data TEXT NOT NULL,
      PRIMARY KEY (guild_id,page)
    );
  ";
  let conn = get_connection()?;
  conn.execute(sql, ())?;
  Ok(())
}

struct Row {
  guild_id: u64,
  page: u64,
  data: String
}

pub fn get_page(guild_id: &u64, page: &u64) -> Result<Option<String>> {
  let sql = "SELECT * FROM role_data WHERE guild_id=? AND page=?";
  let conn = get_connection()?;
  let mut stmt = conn.prepare(sql)?;
  let iter = stmt.query_map([guild_id, page], |row| {
    Ok(Row {
      guild_id: row.get(0)?,
      page: row.get(1)?,
      data: row.get(2)?
    })
  })?;
  for row in iter {
    return Ok(Some(row?.data))
  }
  Ok(None)
}

pub fn set_page(guild_id: &u64, page: &u64, data: &str) -> Result<()> {
  let sql = "REPLACE INTO role_data (guild_id, page, data) VALUES(?,?,?)";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &page, &data))?;
  Ok(())
}

pub fn delete_page(guild_id: &u64, page: &u64) -> Result<()> {
  let sql = "DELETE FROM role_data WHERE guild_id = ? AND page = ?;";
  let conn = get_connection()?;
  conn.execute(sql, (&guild_id, &page))?;
  let sql2 = "UPDATE role_data SET page = page - 1 WHERE guild_id = ? AND page > ?;";
  conn.execute(sql2, (&guild_id, &page))?;
  Ok(())
}

pub fn get_page_amount(guild_id: &u64) -> Result<Option<u64>> {
  let sql = "SELECT * FROM role_data WHERE guild_id = ? ORDER BY page DESC LIMIT 1;";
  let conn = get_connection()?;
  let mut stmt = conn.prepare(sql)?;
  let iter = stmt.query_map([guild_id], |row| {
    Ok(Row {
      guild_id: row.get(0)?,
      page: row.get(1)?,
      data: row.get(2)?
    })
  })?;
  for row in iter {
    return Ok(Some(row?.page))
  }
  Ok(Some(0))
}