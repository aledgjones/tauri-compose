// BEGIN GENERATED MIGRATIONS
#[path = "next.rs"]
mod next;
#[path = "1.25.0.rs"]
mod v1_25_0;
#[path = "1.25.2.rs"]
mod v1_25_2;

// BEGIN SUPPORT MODULES
// Add non-version helper modules here. The release tool preserves this section.
// END SUPPORT MODULES

use lazy_static::lazy_static;
use serde_json::Value;

pub type MigrationFn = fn(Value) -> Result<Value, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationKind {
    Shipped(&'static str),
    PendingNext,
}

#[derive(Clone, Copy)]
pub struct MigrationDescriptor {
    kind: MigrationKind,
    migrate: MigrationFn,
}

impl MigrationDescriptor {
    const fn shipped(version: &'static str, migrate: MigrationFn) -> Self {
        Self {
            kind: MigrationKind::Shipped(version),
            migrate,
        }
    }

    const fn pending_next(migrate: MigrationFn) -> Self {
        Self {
            kind: MigrationKind::PendingNext,
            migrate,
        }
    }

    pub const fn kind(&self) -> MigrationKind {
        self.kind
    }

    pub const fn migrate(&self) -> MigrationFn {
        self.migrate
    }
}

lazy_static! {
    pub static ref MIGRATIONS: Vec<MigrationDescriptor> = vec![
        MigrationDescriptor::shipped("1.25.0", v1_25_0::migrate),
        MigrationDescriptor::shipped("1.25.2", v1_25_2::migrate),
        MigrationDescriptor::pending_next(next::migrate),
    ];
}

// END GENERATED MIGRATIONS
