use std::{
    fmt::Display,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use parking_lot::{Mutex, MutexGuard};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Serialize};
use thread_local::ThreadLocal;

/// A key value storage based on SQLite.
pub struct KVStorage {
    inner: Arc<Inner>,
}

struct Inner {
    path: PathBuf,
    write: Mutex<Connection>,
    read: ThreadLocal<Connection>,
}

impl KVStorage {
    /// Create a new persistent key-value storage on disk.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let write = Connection::open_with_flags(&path, flags)?;
        let read = ThreadLocal::new();

        init(&write)?;

        let inner = Inner {
            path,
            write: Mutex::new(write),
            read,
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get a connection with read and write permissions.
    pub fn write(&self) -> WriteGuard<'_> {
        let guard = self.inner.write.lock();
        WriteGuard { guard }
    }

    /// Get a connection with read-only permission.
    pub fn read(&self) -> Result<ReadGuard<'_>> {
        let conn = self.inner.read.get_or_try(|| {
            let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            Connection::open_with_flags(&self.inner.path, flags)
        })?;

        Ok(ReadGuard { conn })
    }
}

pub struct WriteGuard<'a> {
    guard: MutexGuard<'a, Connection>,
}

pub struct WriteTransaction<'a> {
    tx: Transaction<'a>,
}

pub struct ReadGuard<'a> {
    conn: &'a Connection,
}

impl<'a> WriteGuard<'a> {
    /// Store the given pair of key-value.
    pub fn push<K, V>(&self, key: K, value: &V) -> Result<()>
    where
        K: Display,
        V: Serialize + ?Sized,
    {
        push(&*self.guard, key, value)
    }

    /// Retrieve a value from the storage.
    pub fn pull<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: Display,
        V: DeserializeOwned,
    {
        pull(&*self.guard, key)
    }

    /// Start a new transaction.
    pub fn transaction(&mut self) -> Result<WriteTransaction<'_>> {
        let tx = self.guard.transaction()?;
        Ok(WriteTransaction { tx })
    }
}

impl<'a> WriteTransaction<'a> {
    /// Store the given pair of key-value.
    pub fn push<K, V>(&self, key: K, value: &V) -> Result<()>
    where
        K: Display,
        V: Serialize + ?Sized,
    {
        push(&self.tx, key, value)
    }

    /// Retrieve a value from the storage.
    pub fn pull<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: Display,
        V: DeserializeOwned,
    {
        pull(&self.tx, key)
    }

    /// Commit the transaction.
    pub fn commit(self) -> Result<()> {
        self.tx.commit().map_err(Into::into)
    }

    /// Rollback the transaction.
    pub fn rollback(self) -> Result<()> {
        self.tx.rollback().map_err(Into::into)
    }
}

impl<'a> ReadGuard<'a> {
    /// Retrieve a value from the storage.
    pub fn pull<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: Display,
        V: DeserializeOwned,
    {
        pull(self.conn, key)
    }
}

fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value BLOB NOT NULL)",
        (),
    )?;
    Ok(())
}

fn push<K, V>(q: &Connection, key: K, value: &V) -> Result<()>
where
    K: Display,
    V: Serialize + ?Sized,
{
    let key = key.to_string();
    let value = serde_json::to_vec(value)?;
    q.execute(
        "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
        (key, value),
    )?;
    Ok(())
}

fn pull<K, V>(q: &Connection, key: K) -> Result<Option<V>>
where
    K: Display,
    V: DeserializeOwned,
{
    let key = key.to_string();
    q.query_row("SELECT value FROM kv WHERE key = ?1", (key,), |row| {
        row.get::<_, Vec<u8>>("value")
    })
    .optional()?
    .map(|value| serde_json::from_slice(&value))
    .transpose()
    .map_err(Into::into)
}
