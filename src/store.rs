use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::Row;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{Question, QuestionId},
};

#[derive(Clone, Debug)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Store {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url).await {
                Ok(pool) => pool,
                Err(e) => panic!("Couldn't establish DB connection!"),
            };

            Store {
                connection: db_pool,
            }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}
