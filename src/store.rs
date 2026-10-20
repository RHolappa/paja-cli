// SPDX-License-Identifier: Apache-2.0
use crate::{
    error::{Error, Result},
    model::{Package, digest},
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{self, File, OpenOptions},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub candidate_revision: u32,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub kind: String,
    pub candidate_revision: u32,
    pub state: String,
}
pub struct Store {
    pub db: Connection,
    _lock: File,
}

fn private_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(path)?;
    }
    #[cfg(not(unix))]
    fs::create_dir(path)?;
    Ok(())
}
fn no_symlink(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(Error::invalid("Coordinator storage cannot be a symlink"));
    }
    Ok(())
}
impl Store {
    pub fn create(root: &Path, id: &str) -> Result<Self> {
        private_directory(&root.join(".paja"))?;
        let lock = File::create_new(root.join(".paja/coordinator.lock"))?;
        lock.lock()?;
        let db = Connection::open(root.join(".paja/state.sqlite3"))?;
        db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
            PRAGMA user_version=1;
            CREATE TABLE project (singleton INTEGER PRIMARY KEY CHECK(singleton=1), id TEXT NOT NULL, revision INTEGER NOT NULL, state TEXT NOT NULL);
            CREATE TABLE versions (revision INTEGER PRIMARY KEY, digest TEXT NOT NULL, package TEXT NOT NULL);
            CREATE TABLE runs (sequence INTEGER PRIMARY KEY, id TEXT UNIQUE NOT NULL, kind TEXT NOT NULL, revision INTEGER NOT NULL REFERENCES versions(revision), state TEXT NOT NULL, data TEXT NOT NULL, digest TEXT NOT NULL);
            CREATE TABLE receipts (operation TEXT NOT NULL, key TEXT NOT NULL, fingerprint TEXT NOT NULL, response TEXT NOT NULL, PRIMARY KEY(operation,key));
            CREATE TRIGGER immutable_versions_update BEFORE UPDATE ON versions BEGIN SELECT RAISE(ABORT, 'immutable version'); END;
            CREATE TRIGGER immutable_versions_delete BEFORE DELETE ON versions BEGIN SELECT RAISE(ABORT, 'immutable version'); END;
            CREATE TRIGGER immutable_runs_update BEFORE UPDATE ON runs BEGIN SELECT RAISE(ABORT, 'immutable run'); END;
            CREATE TRIGGER immutable_runs_delete BEFORE DELETE ON runs BEGIN SELECT RAISE(ABORT, 'immutable run'); END;
            CREATE TRIGGER immutable_receipts_update BEFORE UPDATE ON receipts BEGIN SELECT RAISE(ABORT, 'immutable receipt'); END;
            CREATE TRIGGER immutable_receipts_delete BEFORE DELETE ON receipts BEGIN SELECT RAISE(ABORT, 'immutable receipt'); END;
        ")?;
        db.execute("INSERT INTO project VALUES (1, ?1, 0, 'draft')", [id])?;
        Ok(Self { db, _lock: lock })
    }
    pub fn open(root: &Path) -> Result<Self> {
        if !root.join(".paja/state.sqlite3").exists() {
            return Err(Error::missing(
                "No local project. Start with paja init <name> --goal <text>",
            ));
        }
        for name in [".paja", ".paja/coordinator.lock", ".paja/state.sqlite3"] {
            no_symlink(&root.join(name))?;
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(".paja/coordinator.lock"))?;
        lock.try_lock().map_err(|_| {
            Error::new(
                "coordinator_busy",
                "Another local operation owns the coordinator; retry with the same request ID",
                6,
            )
        })?;
        let db = Connection::open_with_flags(
            root.join(".paja/state.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        let version: u32 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version != 1 {
            return Err(Error::invalid("Unsupported coordinator database schema"));
        }
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
        Ok(Self { db, _lock: lock })
    }
}

pub fn new_project_id(name: &str) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::invalid("System clock precedes Unix epoch"))?
        .as_nanos();
    Ok(format!(
        "prj_{}",
        &digest(&(name, now, std::process::id()))?[..24]
    ))
}
pub fn project(db: &Connection) -> Result<Project> {
    Ok(db.query_row(
        "SELECT id, revision, state FROM project WHERE singleton=1",
        [],
        |r| {
            Ok(Project {
                id: r.get(0)?,
                candidate_revision: r.get(1)?,
                state: r.get(2)?,
            })
        },
    )?)
}
pub fn set_current(db: &Connection, revision: u32, state: &str) -> Result<()> {
    db.execute(
        "UPDATE project SET revision=?1, state=?2 WHERE singleton=1",
        params![revision, state],
    )?;
    Ok(())
}
pub fn next_revision(db: &Connection) -> Result<u32> {
    Ok(db.query_row(
        "SELECT COALESCE(MAX(revision),0)+1 FROM versions",
        [],
        |r| r.get(0),
    )?)
}
pub fn save_package(db: &Connection, revision: u32, package: &Package) -> Result<String> {
    let hash = digest(package)?;
    db.execute(
        "INSERT INTO versions VALUES (?1, ?2, ?3)",
        params![revision, hash, serde_json::to_string(package)?],
    )?;
    Ok(hash)
}
pub fn package(db: &Connection, revision: u32) -> Result<(Package, String)> {
    let value: Option<(String, String)> = db
        .query_row(
            "SELECT package, digest FROM versions WHERE revision=?1",
            [revision],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (json, hash) =
        value.ok_or_else(|| Error::missing("No such built revision; build the project first"))?;
    let package: Package = serde_json::from_str(&json)?;
    if package.schema_version != "paja.package.v1" || digest(&package)? != hash {
        return Err(Error::integrity("Stored package digest or schema mismatch"));
    }
    package.agent.validate()?;
    package.default_suite.validate()?;
    Ok((package, hash))
}
pub fn next_run_id(db: &Connection) -> Result<String> {
    let id: i64 = db.query_row("SELECT COALESCE(MAX(sequence),0)+1 FROM runs", [], |r| {
        r.get(0)
    })?;
    Ok(format!("run_{id:06}"))
}
pub fn save_run(db: &Connection, run: &Run, data: &Value) -> Result<()> {
    let hash = digest(&(run, data))?;
    db.execute(
        "INSERT INTO runs(id,kind,revision,state,data,digest) VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            run.id,
            run.kind,
            run.candidate_revision,
            run.state,
            serde_json::to_string(data)?,
            hash
        ],
    )?;
    Ok(())
}
pub fn run(db: &Connection, id: &str) -> Result<(Run, Value)> {
    let record: Option<(Run, String, String)> = db
        .query_row(
            "SELECT kind,revision,state,data,digest FROM runs WHERE id=?1",
            [id],
            |r| {
                Ok((
                    Run {
                        id: id.into(),
                        kind: r.get(0)?,
                        candidate_revision: r.get(1)?,
                        state: r.get(2)?,
                    },
                    r.get(3)?,
                    r.get(4)?,
                ))
            },
        )
        .optional()?;
    let (run, json, hash) = record.ok_or_else(|| Error::invalid("Unknown run ID"))?;
    let data: Value = serde_json::from_str(&json)?;
    if digest(&(&run, &data))? != hash {
        return Err(Error::integrity("Stored run digest mismatch"));
    }
    Ok((run, data))
}
pub fn latest_test(db: &Connection, revision: u32) -> Result<Option<String>> {
    Ok(db.query_row("SELECT id FROM runs WHERE revision=?1 AND kind IN ('test','improve') ORDER BY sequence DESC LIMIT 1", [revision], |r| r.get(0)).optional()?)
}
pub fn receipt(
    db: &Connection,
    operation: &str,
    key: Option<&str>,
    fingerprint: &str,
) -> Result<Option<Value>> {
    let Some(key) = key else { return Ok(None) };
    let row: Option<(String, String)> = db
        .query_row(
            "SELECT fingerprint,response FROM receipts WHERE operation=?1 AND key=?2",
            params![operation, key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match row {
        Some((saved, response)) if saved == fingerprint => {
            Ok(Some(serde_json::from_str(&response)?))
        }
        Some(_) => Err(Error::new(
            "idempotency_conflict",
            "Request ID was already used with different inputs",
            4,
        )),
        None => Ok(None),
    }
}
pub fn save_receipt(
    db: &Connection,
    operation: &str,
    key: Option<&str>,
    fingerprint: &str,
    response: &Value,
) -> Result<()> {
    if let Some(key) = key {
        db.execute(
            "INSERT INTO receipts VALUES (?1,?2,?3,?4)",
            params![
                operation,
                key,
                fingerprint,
                serde_json::to_string(response)?
            ],
        )?;
    }
    Ok(())
}
