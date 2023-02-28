use rusqlite::params;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

use crate::helpers::user::User;

pub fn init_db(pool: &Pool<SqliteConnectionManager>) {
    let conn: PooledConnection<SqliteConnectionManager> = pool.get().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            admin INTEGER CHECK(admin IN (0, 1))
        )",
        [],
    ).unwrap();
}

pub fn get_user(pool: &Pool<SqliteConnectionManager>, username: &str) -> Option<User> {
    let conn: PooledConnection<SqliteConnectionManager> = pool.get().unwrap();
    conn.query_row(
        "SELECT username, password, admin FROM users WHERE username = ?1",
        params![username],
        |row| Ok(User {
            username: row.get(0)?,
            password: row.get(1)?,
            admin: row.get(2)?,
        }),
    ).ok()
}

pub fn insert_user(pool: &Pool<SqliteConnectionManager>, user: User) -> Result<(), Box<dyn std::error::Error>> {
    let conn: PooledConnection<SqliteConnectionManager> = pool.get().unwrap();
    let mut stmt = conn
        .prepare("INSERT INTO users (username, password, admin) VALUES (?, ?, ?)")
        .unwrap();
    let result = stmt.execute(params![user.username, user.password, user.admin]);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err("failed to insert user".into()),
    }
}