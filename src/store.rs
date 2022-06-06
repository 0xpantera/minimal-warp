use std::collections::HashMap;
use sqlx::postgres::{PgPoolOptions, PgPool, PgRow};
use sqlx::Row;

use handle_errors::Error;

use crate::types::{
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

            Store { connection: db_pool, }
    }

    pub async fn get_questions(
        &self, 
        limit: Option<u32>, 
        offset: u32
    ) -> Result<Vec<Question>, sqlx::Error> {
        match sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags")
            })
            .fetch_all(&self.connection)
            .await {
                Ok(questions) => Ok(questions),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(Error::DatabaseQueryError)
                }
            }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}
