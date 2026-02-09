use std::sync::{Arc, Mutex};

use crate::db::InMemDatabase;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<InMemDatabase>>,
}

impl AppState {
    pub fn new(db: InMemDatabase) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }
}
