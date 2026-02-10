//! Shared test utilities for Foxus.
//!
//! This module provides common setup functions used across test modules.

#![cfg(test)]

use crate::db::{migrations, Database};
use tempfile::{tempdir, TempDir};

/// Create a temporary test database with migrations applied.
///
/// Returns a tuple of (Database, TempDir). The TempDir must be kept alive
/// for the duration of the test to prevent the database file from being deleted.
pub fn setup_test_db() -> (Database, TempDir) {
    let dir = tempdir().expect("Failed to create temp directory for test DB");
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("Failed to open test database");
    migrations::run(db.connection()).expect("Failed to run migrations on test DB");
    (db, dir)
}
