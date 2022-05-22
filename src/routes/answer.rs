use std::collections::HashMap;

use warp::hyper::StatusCode;

use crate::{
    store::Store,
    types::{
        answer::{Answer, AnswerId},
        question::QuestionId,
    },

};


// TODO:
// Create a random, unique ID instead of the one by hand
// Add error handling if the fields we require aren't present
// Check if a question exists
// Change route to answers: /questions/:questionId/answers 
pub async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    };

    store.answers.write().insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status(
        "Answer added",
        StatusCode::OK))
}