use warp::hyper::StatusCode;

use crate::{
    store::Store,
    types::answer::NewAnswer,
};

// TODO:
// Create a random, unique ID instead of the one by hand
// Add error handling if the fields we require aren't present
// Check if a question exists
// Change route to answers: /questions/:questionId/answers
pub async fn add_answer(
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_answer(new_answer).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
