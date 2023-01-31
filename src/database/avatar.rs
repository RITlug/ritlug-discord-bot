use rusqlite::{Connection, Result, Error};


fn get_connection() -> Result<Connection> {
    return Connection::open("database.db");
}

pub fn init() -> Result<()> {
    let sql = "
        CREATE TABLE IF NOT EXISTS irc_avatars (
            nick TEXT NOT NULL PRIMARY KEY,
            url TEXT NOT NULL
        );
    ";
    let conn = get_connection()?;
    conn.execute(sql, ())?;
    Ok(())
}

pub fn update_avatar(nick: &str, url: &str) -> Result<()> {
    let sql = "REPLACE INTO irc_avatars (nick, url) VALUES (?, ?)";
    let conn = get_connection()?;
    conn.execute(sql, (nick, url))?;
    Ok(())
}

pub fn get_avatar(name: &str) -> Result<Option<String>> {
    let sql = "SELECT * FROM irc_avatars WHERE nick=?";
    let conn = get_connection()?;
    let mut stmt = conn.prepare(sql)?;
    match stmt.query_row((name,), |row| row.get(1)) {
        Ok(url) => Ok(Some(url)),
        Err(Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e)
    }
}
