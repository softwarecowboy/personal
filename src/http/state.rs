use std::sync::{Arc, Mutex};

use crate::db::InMemDatabase;
use crate::views::ViewCounterStore;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<InMemDatabase>>,
    pub views: Arc<Mutex<ViewCounterStore>>,
}

impl AppState {
    pub fn new(db: InMemDatabase, views: ViewCounterStore) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            views: Arc::new(Mutex::new(views)),
        }
    }
}
