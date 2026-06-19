use crate::errors::Result;
use crate::types::{ErrorEntry, TrackedObject, Transaction};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// Local SQLite registry for tracked objects, transactions, and errors.
pub struct Registry {
    conn: Connection,
}

impl Registry {
    /// Open (or create) the registry database at `path` and run migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrent read performance (dashboard + CLI)
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        let registry = Self { conn };
        registry.migrate()?;
        Ok(registry)
    }

    /// Open an in-memory database — used for tests.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let registry = Self { conn };
        registry.migrate()?;
        Ok(registry)
    }

    /// Create tables and indices if they don't exist.
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS objects (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                object_id   TEXT NOT NULL UNIQUE,
                object_type TEXT,
                alias       TEXT,
                owner       TEXT,
                package_id  TEXT,
                version     TEXT,
                digest      TEXT,
                tx_digest   TEXT,
                network     TEXT NOT NULL DEFAULT 'testnet',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS transactions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_digest   TEXT NOT NULL UNIQUE,
                command     TEXT,
                status      TEXT NOT NULL,
                gas_used    INTEGER,
                gas_owner   TEXT,
                package_id  TEXT,
                module_name TEXT,
                function    TEXT,
                raw_response TEXT,
                network     TEXT NOT NULL DEFAULT 'testnet',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS errors (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                error_code   TEXT,
                error_message TEXT NOT NULL,
                module_id    TEXT,
                explanation  TEXT,
                tx_digest    TEXT,
                network      TEXT NOT NULL DEFAULT 'testnet',
                created_at   TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_objects_alias ON objects(alias);
            CREATE INDEX IF NOT EXISTS idx_objects_package ON objects(package_id);
            CREATE INDEX IF NOT EXISTS idx_transactions_package ON transactions(package_id);
            CREATE INDEX IF NOT EXISTS idx_errors_tx ON errors(tx_digest);
            ",
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Objects
    // -----------------------------------------------------------------------

    /// Insert or update a tracked object (upsert on `object_id`).
    pub fn upsert_object(&self, obj: &TrackedObject) -> Result<()> {
        self.conn.execute(
            "INSERT INTO objects (object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(object_id) DO UPDATE SET
                object_type = COALESCE(excluded.object_type, object_type),
                alias       = COALESCE(excluded.alias, alias),
                owner       = COALESCE(excluded.owner, owner),
                package_id  = COALESCE(excluded.package_id, package_id),
                version     = COALESCE(excluded.version, version),
                digest      = COALESCE(excluded.digest, digest),
                tx_digest   = COALESCE(excluded.tx_digest, tx_digest),
                updated_at  = datetime('now')",
            params![
                obj.object_id,
                obj.object_type,
                obj.alias,
                obj.owner,
                obj.package_id,
                obj.version,
                obj.digest,
                obj.tx_digest,
                obj.network,
            ],
        )?;
        Ok(())
    }

    /// List all tracked objects for a given network.
    pub fn list_objects(&self, network: &str) -> Result<Vec<TrackedObject>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network, created_at, updated_at
             FROM objects
             WHERE network = ?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![network], |row| {
            Ok(TrackedObject {
                id: row.get(0)?,
                object_id: row.get(1)?,
                object_type: row.get(2)?,
                alias: row.get(3)?,
                owner: row.get(4)?,
                package_id: row.get(5)?,
                version: row.get(6)?,
                digest: row.get(7)?,
                tx_digest: row.get(8)?,
                network: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let mut objects = Vec::new();
        for row in rows {
            objects.push(row?);
        }
        Ok(objects)
    }

    /// Resolve an alias to a tracked object.
    pub fn get_by_alias(&self, alias: &str) -> Result<Option<TrackedObject>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network, created_at, updated_at
                 FROM objects WHERE alias = ?1",
                params![alias],
                |row| {
                    Ok(TrackedObject {
                        id: row.get(0)?,
                        object_id: row.get(1)?,
                        object_type: row.get(2)?,
                        alias: row.get(3)?,
                        owner: row.get(4)?,
                        package_id: row.get(5)?,
                        version: row.get(6)?,
                        digest: row.get(7)?,
                        tx_digest: row.get(8)?,
                        network: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Look up an object by its Object ID.
    pub fn get_by_id(&self, object_id: &str) -> Result<Option<TrackedObject>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network, created_at, updated_at
                 FROM objects WHERE object_id = ?1",
                params![object_id],
                |row| {
                    Ok(TrackedObject {
                        id: row.get(0)?,
                        object_id: row.get(1)?,
                        object_type: row.get(2)?,
                        alias: row.get(3)?,
                        owner: row.get(4)?,
                        package_id: row.get(5)?,
                        version: row.get(6)?,
                        digest: row.get(7)?,
                        tx_digest: row.get(8)?,
                        network: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Assign (or overwrite) an alias for an object.
    pub fn set_alias(&self, object_id: &str, alias: &str) -> Result<bool> {
        let rows = self.conn.execute(
            "UPDATE objects SET alias = ?1, updated_at = datetime('now') WHERE object_id = ?2",
            params![alias, object_id],
        )?;
        Ok(rows > 0)
    }

    /// Delete an object from the registry.
    pub fn delete_object(&self, object_id: &str) -> Result<bool> {
        let rows = self.conn.execute(
            "DELETE FROM objects WHERE object_id = ?1",
            params![object_id],
        )?;
        Ok(rows > 0)
    }

    /// Resolve an alias OR object ID to a TrackedObject.
    /// Tries alias first, then falls back to object ID lookup.
    pub fn resolve(&self, id_or_alias: &str) -> Result<Option<TrackedObject>> {
        if let Some(obj) = self.get_by_alias(id_or_alias)? {
            return Ok(Some(obj));
        }
        self.get_by_id(id_or_alias)
    }

    // -----------------------------------------------------------------------
    // Transactions
    // -----------------------------------------------------------------------

    /// Record a transaction.
    pub fn insert_transaction(&self, tx: &Transaction) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO transactions
             (tx_digest, command, status, gas_used, gas_owner, package_id, module_name, function, raw_response, network)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                tx.tx_digest,
                tx.command,
                tx.status,
                tx.gas_used,
                tx.gas_owner,
                tx.package_id,
                tx.module_name,
                tx.function,
                tx.raw_response,
                tx.network,
            ],
        )?;
        Ok(())
    }

    /// List transactions for a network, most recent first.
    pub fn list_transactions(&self, network: &str, limit: u32) -> Result<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_digest, command, status, gas_used, gas_owner, package_id, module_name, function, raw_response, network, created_at
             FROM transactions
             WHERE network = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![network, limit], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                tx_digest: row.get(1)?,
                command: row.get(2)?,
                status: row.get(3)?,
                gas_used: row.get(4)?,
                gas_owner: row.get(5)?,
                package_id: row.get(6)?,
                module_name: row.get(7)?,
                function: row.get(8)?,
                raw_response: row.get(9)?,
                network: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;

        let mut txs = Vec::new();
        for row in rows {
            txs.push(row?);
        }
        Ok(txs)
    }

    // -----------------------------------------------------------------------
    // Errors
    // -----------------------------------------------------------------------

    /// Log an error.
    pub fn insert_error(&self, entry: &ErrorEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO errors (error_code, error_message, module_id, explanation, tx_digest, network)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.error_code,
                entry.error_message,
                entry.module_id,
                entry.explanation,
                entry.tx_digest,
                entry.network,
            ],
        )?;
        Ok(())
    }

    /// List errors for a network, most recent first.
    pub fn list_errors(&self, network: &str, limit: u32) -> Result<Vec<ErrorEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, error_code, error_message, module_id, explanation, tx_digest, network, created_at
             FROM errors
             WHERE network = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![network, limit], |row| {
            Ok(ErrorEntry {
                id: row.get(0)?,
                error_code: row.get(1)?,
                error_message: row.get(2)?,
                module_id: row.get(3)?,
                explanation: row.get(4)?,
                tx_digest: row.get(5)?,
                network: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut errors = Vec::new();
        for row in rows {
            errors.push(row?);
        }
        Ok(errors)
    }

    /// Expose the raw connection for advanced queries (dashboard use).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Merge data from another SQLite registry database file.
    pub fn merge_from_db_file(&self, other_db_path: &Path) -> Result<()> {
        let other_path_str = other_db_path.to_str().ok_or_else(|| {
            crate::errors::SuiScopeError::Config("Invalid UTF-8 in other database path".into())
        })?;
        
        let attach_query = format!("ATTACH DATABASE '{}' AS other;", other_path_str.replace('\'', "''"));
        self.conn.execute(&attach_query, [])?;

        let merge_res = (|| -> Result<()> {
            // Merge objects: upsert on object_id
            self.conn.execute(
                "INSERT INTO objects (object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network, created_at, updated_at)
                 SELECT object_id, object_type, alias, owner, package_id, version, digest, tx_digest, network, created_at, updated_at
                 FROM other.objects
                 WHERE 1
                 ON CONFLICT(object_id) DO UPDATE SET
                    object_type = COALESCE(excluded.object_type, object_type),
                    alias       = COALESCE(excluded.alias, alias),
                    owner       = COALESCE(excluded.owner, owner),
                    package_id  = COALESCE(excluded.package_id, package_id),
                    version     = COALESCE(excluded.version, version),
                    digest      = COALESCE(excluded.digest, digest),
                    tx_digest   = COALESCE(excluded.tx_digest, tx_digest),
                    updated_at  = datetime('now')",
                [],
            )?;

            // Merge transactions: upsert on tx_digest
            self.conn.execute(
                "INSERT INTO transactions (tx_digest, command, status, gas_used, gas_owner, package_id, module_name, function, raw_response, network, created_at)
                 SELECT tx_digest, command, status, gas_used, gas_owner, package_id, module_name, function, raw_response, network, created_at
                 FROM other.transactions
                 WHERE 1
                 ON CONFLICT(tx_digest) DO UPDATE SET
                    command      = COALESCE(excluded.command, command),
                    status       = excluded.status,
                    gas_used     = COALESCE(excluded.gas_used, gas_used),
                    gas_owner    = COALESCE(excluded.gas_owner, gas_owner),
                    package_id   = COALESCE(excluded.package_id, package_id),
                    module_name  = COALESCE(excluded.module_name, module_name),
                    function     = COALESCE(excluded.function, function),
                    raw_response = COALESCE(excluded.raw_response, raw_response)",
                [],
            )?;

            // Merge errors: insert new errors
            self.conn.execute(
                "INSERT INTO errors (error_code, error_message, module_id, explanation, tx_digest, network, created_at)
                 SELECT error_code, error_message, module_id, explanation, tx_digest, network, created_at
                 FROM other.errors AS oe
                 WHERE NOT EXISTS (
                     SELECT 1 FROM errors WHERE tx_digest = oe.tx_digest AND error_message = oe.error_message
                 )",
                [],
            )?;

            Ok(())
        })();

        let detach_res = self.conn.execute("DETACH DATABASE other;", []);

        merge_res.and(detach_res.map(|_| ()).map_err(Into::into))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_object(object_id: &str) -> TrackedObject {
        TrackedObject {
            id: None,
            object_id: object_id.to_string(),
            object_type: Some("0x2::coin::Coin<0x2::sui::SUI>".to_string()),
            alias: None,
            owner: Some("AddressOwner(0xabc)".to_string()),
            package_id: Some("0xpkg".to_string()),
            version: Some("1".to_string()),
            digest: Some("abc123".to_string()),
            tx_digest: Some("tx_abc".to_string()),
            network: "testnet".to_string(),
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_insert_and_list_objects() {
        let reg = Registry::open_in_memory().unwrap();
        let obj = make_test_object("0x0001");
        reg.upsert_object(&obj).unwrap();

        let objects = reg.list_objects("testnet").unwrap();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].object_id, "0x0001");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let reg = Registry::open_in_memory().unwrap();
        let obj = make_test_object("0x0001");
        reg.upsert_object(&obj).unwrap();

        // Update with new version
        let mut updated = obj.clone();
        updated.version = Some("2".to_string());
        reg.upsert_object(&updated).unwrap();

        let objects = reg.list_objects("testnet").unwrap();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].version.as_deref(), Some("2"));
    }

    #[test]
    fn test_alias_roundtrip() {
        let reg = Registry::open_in_memory().unwrap();
        let obj = make_test_object("0x0001");
        reg.upsert_object(&obj).unwrap();

        reg.set_alias("0x0001", "my-coin").unwrap();
        let found = reg.get_by_alias("my-coin").unwrap().unwrap();
        assert_eq!(found.object_id, "0x0001");
    }

    #[test]
    fn test_resolve_alias_then_id() {
        let reg = Registry::open_in_memory().unwrap();
        let mut obj = make_test_object("0x0001");
        obj.alias = Some("my-coin".to_string());
        reg.upsert_object(&obj).unwrap();

        // Resolve by alias
        let found = reg.resolve("my-coin").unwrap().unwrap();
        assert_eq!(found.object_id, "0x0001");

        // Resolve by ID
        let found = reg.resolve("0x0001").unwrap().unwrap();
        assert_eq!(found.object_id, "0x0001");
    }

    #[test]
    fn test_delete_object() {
        let reg = Registry::open_in_memory().unwrap();
        let obj = make_test_object("0x0001");
        reg.upsert_object(&obj).unwrap();

        assert!(reg.delete_object("0x0001").unwrap());
        assert!(reg.get_by_id("0x0001").unwrap().is_none());
    }

    #[test]
    fn test_insert_and_list_transactions() {
        let reg = Registry::open_in_memory().unwrap();
        let tx = Transaction {
            id: None,
            tx_digest: "tx_001".to_string(),
            command: Some("publish".to_string()),
            status: "success".to_string(),
            gas_used: Some(5_000_000),
            gas_owner: Some("0xsender".to_string()),
            package_id: Some("0xpkg".to_string()),
            module_name: Some("counter".to_string()),
            function: None,
            raw_response: None,
            network: "testnet".to_string(),
            created_at: None,
        };
        reg.insert_transaction(&tx).unwrap();

        let txs = reg.list_transactions("testnet", 10).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].tx_digest, "tx_001");
    }

    #[test]
    fn test_insert_and_list_errors() {
        let reg = Registry::open_in_memory().unwrap();
        let entry = ErrorEntry {
            id: None,
            error_code: Some("-32602".to_string()),
            error_message: "Invalid params".to_string(),
            module_id: None,
            explanation: Some("The parameters you sent are wrong.".to_string()),
            tx_digest: None,
            network: "testnet".to_string(),
            created_at: None,
        };
        reg.insert_error(&entry).unwrap();

        let errors = reg.list_errors("testnet", 10).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code.as_deref(), Some("-32602"));
    }

    #[test]
    fn test_merge_from_db_file() {
        let temp_dir = std::env::temp_dir().join("suiscope_merge_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db1_path = temp_dir.join("db1.db");
        let db2_path = temp_dir.join("db2.db");

        let reg1 = Registry::open(&db1_path).unwrap();
        let reg2 = Registry::open(&db2_path).unwrap();

        // Populate database 1
        let obj1 = make_test_object("0x0001");
        reg1.upsert_object(&obj1).unwrap();

        // Populate database 2
        let mut obj2 = make_test_object("0x0002");
        obj2.alias = Some("alias2".to_string());
        reg2.upsert_object(&obj2).unwrap();

        // Merge DB2 into DB1
        reg1.merge_from_db_file(&db2_path).unwrap();

        // Verify DB1 now contains both objects
        let objects = reg1.list_objects("testnet").unwrap();
        assert_eq!(objects.len(), 2);

        let o1 = reg1.get_by_id("0x0001").unwrap().unwrap();
        let o2 = reg1.get_by_id("0x0002").unwrap().unwrap();
        assert_eq!(o1.object_id, "0x0001");
        assert_eq!(o2.object_id, "0x0002");
        assert_eq!(o2.alias.as_deref(), Some("alias2"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
