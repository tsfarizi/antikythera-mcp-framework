//! MongoDB storage backend.
//!
//! Sessions are stored as documents with binary data. Supports
//! automatic schema creation with JSON Schema validation.

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use futures::stream::StreamExt;
use mongodb::bson::{Binary, Document, doc, spec::BinarySubtype};
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};

use crate::config::MongodbConfig;
use crate::error::StorageError;

use super::StorageBackend;

/// MongoDB storage backend for session persistence.
pub struct MongoBackend {
    client: Client,
    database: String,
    collection: String,
    backup_dir: PathBuf,
}

impl MongoBackend {
    /// Create a new MongoDB backend from configuration.
    ///
    /// Connects to MongoDB using the URI from `config`. If `auto_create_schema`
    /// is enabled, creates the collection with a validation schema.
    pub async fn new(config: &MongodbConfig) -> Result<Self, StorageError> {
        let client_options = ClientOptions::parse(&config.uri)
            .await
            .map_err(|e| StorageError::Connection(format!("failed to parse MongoDB URI: {e}")))?;

        let client = Client::with_options(client_options).map_err(|e| {
            StorageError::Connection(format!("failed to create MongoDB client: {e}"))
        })?;

        let database = config.database.clone();
        let collection_name = config.collection.clone();
        let backup_dir = PathBuf::from("./data/backups");

        if config.auto_create_schema {
            let db = client.database(&database);
            let coll = db.collection::<Document>(&collection_name);

            let index = IndexModel::builder()
                .keys(doc! { "created_at": 1 })
                .options(IndexOptions::builder().build())
                .build();
            coll.create_index(index)
                .await
                .map_err(|e| StorageError::Schema(format!("failed to create index: {e}")))?;

            let validator = doc! {
                "$jsonSchema": doc! {
                    "bsonType": "object",
                    "required": vec!["_id", "data", "created_at", "updated_at"],
                    "properties": doc! {
                        "_id": doc! { "bsonType": "string" },
                        "data": doc! { "bsonType": "binData" },
                        "created_at": doc! { "bsonType": "string" },
                        "updated_at": doc! { "bsonType": "string" }
                    }
                }
            };

            db.run_command(doc! {
                "collMod": &collection_name,
                "validator": validator
            })
            .await
            .ok();
        }

        Ok(Self {
            client,
            database,
            collection: collection_name,
            backup_dir,
        })
    }

    /// Get a handle to the sessions collection.
    fn coll(&self) -> Collection<Document> {
        self.client
            .database(&self.database)
            .collection(&self.collection)
    }

    /// Get the backup file path for a session.
    fn backup_path(&self, session_id: &str) -> PathBuf {
        self.backup_dir.join(format!("{session_id}.json"))
    }
}

#[async_trait]
impl StorageBackend for MongoBackend {
    async fn save(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        let binary = Binary {
            subtype: BinarySubtype::Generic,
            bytes: data.to_vec(),
        };
        let doc = doc! {
            "_id": session_id,
            "data": binary,
            "created_at": &now,
            "updated_at": &now,
        };

        self.coll()
            .insert_one(doc)
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB insert failed: {e}")))?;

        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let filter = doc! { "_id": session_id };

        let result = self
            .coll()
            .find_one(filter)
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB find failed: {e}")))?;

        match result {
            Some(doc) => {
                let data = doc
                    .get_binary_generic("data")
                    .map_err(|e| StorageError::Backend(format!("invalid data field: {e}")))?;
                Ok(Some(data.clone()))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), StorageError> {
        let filter = doc! { "_id": session_id };

        self.coll()
            .delete_one(filter)
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB delete failed: {e}")))?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>, StorageError> {
        let mut cursor = self
            .coll()
            .find(doc! {})
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB find failed: {e}")))?;

        let mut ids = Vec::new();

        while let Some(result) = cursor
            .next()
            .await
            .transpose()
            .map_err(|e| StorageError::Backend(format!("MongoDB cursor error: {e}")))?
        {
            if let Ok(id) = result.get_str("_id") {
                ids.push(id.to_string());
            }
        }

        Ok(ids)
    }

    async fn exists(&self, session_id: &str) -> Result<bool, StorageError> {
        let filter = doc! { "_id": session_id };

        let count = self
            .coll()
            .count_documents(filter)
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB count failed: {e}")))?;

        Ok(count > 0)
    }

    async fn backup(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.backup_path(session_id);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError::Backup(format!("failed to create backup dir: {e}")))?;
        }

        std::fs::write(&path, data)
            .map_err(|e| StorageError::Backup(format!("failed to write backup file: {e}")))?;

        Ok(())
    }

    async fn sync_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.backup_path(session_id);

        let data = std::fs::read(&path)
            .map_err(|e| StorageError::Backup(format!("failed to read backup file: {e}")))?;

        let now = Utc::now().to_rfc3339();
        let binary = Binary {
            subtype: BinarySubtype::Generic,
            bytes: data,
        };

        let filter = doc! { "_id": session_id };
        let update = doc! {
            "$set": doc! {
                "data": binary,
                "updated_at": &now
            },
            "$setOnInsert": doc! {
                "created_at": &now
            }
        };

        self.coll()
            .update_one(filter, update)
            .upsert(true)
            .await
            .map_err(|e| StorageError::Backend(format!("MongoDB upsert failed: {e}")))?;

        Ok(())
    }

    async fn verify_sync(&self, session_id: &str) -> Result<bool, StorageError> {
        self.exists(session_id).await
    }

    async fn delete_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.backup_path(session_id);

        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| StorageError::Backup(format!("failed to delete backup file: {e}")))?;
        }

        Ok(())
    }
}
