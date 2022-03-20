extern crate rusqlite;
use rusqlite::Connection;
pub use rusqlite::{Result, Error};

use std::collections::HashSet;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(fname: &str) -> Storage {
        let storage = Storage {
            conn: Connection::open(fname).unwrap(),
        };
        storage.conn.execute("create table if not exists records (
            id integer primary key,
            name text not null unique
        )", []).unwrap();
        match storage.conn.execute("create table if not exists lcells (
            id integer primary key,
            record_id integer not null references records(id),
            x integer not null,
            y integer not null
        )", []) {
            Ok(_) => {},
            Err(s) => {
                println!("{}", s);
                panic!("{}", s);
            },
        }
        storage
    }

    pub fn delete(&mut self, name: &str) -> Result<()> {
        let recs = self.get_records()?;
        let mut rid: i64 = -1;
        for (id, n) in recs {
              if n == String::from(name) {
                rid = id;
                break;
            }
        }
        if rid < 0i64 {
            return Err(Error::InvalidQuery);
        }
        let tx = self.conn.transaction()?;
        tx.execute("delete from lcells where record_id=(?1)", &[&rid])?;
        tx.execute("delete from records where id=(?1)", &[&rid])?;
        tx.commit()?;
        Ok(())
    }

    pub fn save(&mut self, name: &str, cells: &HashSet<(i32, i32)>) -> Result<i64> {
        let tx = self.conn.transaction()?;
        tx.execute("insert into records (name) values (?1)", &[&name.to_string()])?;
        let last_id = tx.last_insert_rowid();

        for (x, y) in cells.iter() {
            let (xx, yy) = (*x as i64, *y as i64);
            tx.execute("insert into lcells (record_id, x, y) values (?1, ?2, ?3)", &[&last_id, &xx, &yy])?;
        }
        tx.commit()?;
        Ok(last_id)
    }

    pub fn get_records(&self) -> Result<Vec<(i64, String)>> {
        let mut recs: Vec<(i64, String)> = vec![];
        let mut sel = self.conn.prepare("SELECT id, name from records;")?;
        let rows = sel.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        for idn in rows {
            let ir = idn?;
            recs.push((ir.0, ir.1));
        } 
        Ok(recs)
    }

    pub fn load(&self, name: &str) -> Result<Vec<(i32, i32)>> {
        let recs = self.get_records()?;

        let mut rid: i64 = -1;
        for (id, n) in recs {
              if n == String::from(name) {
                rid = id;
                break;
            }
        }
        if rid < 0i64 {
            return Err(Error::InvalidQuery);
        }
        let mut sel = self.conn.prepare("SELECT x, y from lcells WHERE record_id=?1;")?;
        let rows = sel.query_map(&[&rid], |row| {
            Ok((row.get(0), row.get(1)))
        })?;

        let mut res: Vec<(i32, i32)> = vec![];
        for idn in rows {
            let ir = idn?;
            res.push((ir.0?, ir.1?))
        }
        Ok(res)
    }
}

#[cfg(test)]

#[test]
fn test_save() {
    let mut storage = Storage::new("test.db");
    let mut cells: HashSet<(i32, i32)> = HashSet::new();
    cells.insert((1, 2));
    cells.insert((3, 4));
    storage.save("test_cfg", &cells);
}
