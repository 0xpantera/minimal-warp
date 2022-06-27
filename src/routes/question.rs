use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::hyper::StatusCode;
use tracing::{instrument, Level};

use handle_errors;
use crate::store::Store;

use crate::types::{
    pagination::{Pagination, extract_pagination},
    question::{Question, NewQuestion},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse(String);

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,
    #[serde(rename = "replacedLen")]
    replaced_len: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

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
            Err(e) => return Err(warp::reject::custom(e)),
        };

        Ok(warp::reply::json(&res))
}

pub async fn add_question(
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character={censor_character}")
        .header("apikey", "sj7Ik9TUYAUlhs6oMuGzK4ErlMbc8Ske")
        .body(new_question.content)
        .send()
        .await
        .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;

    if !res.status().is_success() {
        let status = res.status().as_u16();
        let message = res.json::<APIResponse>().await.unwrap();

        let err = handle_errors::APILayerError {
            status,
            message: message.0,
        };

        if status < 500 {
            return Err(warp::reject::custom(handle_errors::Error::ClientError(err)));
        } else {
            return Err(warp::reject::custom(handle_errors::Error::ServerError(err)));
        }
    }

    let res = res.json::<BadWordsResponse>()
        .await
        .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;

    let content = res.censored_content;

    let question = NewQuestion {
        title: new_question.title,
        content,
        tags: new_question.tags,
    };

    match store.add_question(question).await {
        Ok(_) => Ok(warp::reply::with_status("Question added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn update_question(
    id: i32,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = match store.update_question(question, id).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    Ok(warp::reply::json(&res))
}

pub async fn delete_question(
    id: i32,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Err(e) = store.delete_question(id).await {
        return Err(warp::reject::custom(e));
    }

    Ok(warp::reply::with_status(format!("Question {} deleted", id), StatusCode::OK))
}
