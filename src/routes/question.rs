use handle_errors::Error;
use std::collections::HashMap;
use warp::hyper::StatusCode;
use tracing::{instrument, info, Level};

use crate::store::Store;

use crate::types::{
    pagination::{Pagination, extract_pagination},
    question::{Question, QuestionId},
};

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    tracing::event!(target: "minimal_warp", tracing::Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        tracing::event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }
        let res: Vec<Question> = match store.get_questions(pagination.limit, pagination.offset).await {
            Ok(res) => res,
            Err(e) => return Err(warp::reject::custom(Error::DatabaseQueryError)),
        };

        Ok(warp::reply::json(&res))
}

pub async fn add_question(
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    store
        .questions
        .write()
        .insert(question.id.clone(), question);

    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

pub async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }

    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

pub async fn delete_question(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    }
}
