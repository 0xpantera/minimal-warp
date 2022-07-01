use warp::hyper::StatusCode;

use crate::{
    store::Store,
    types::answer::NewAnswer, profanity::check_profanity,
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
    let content = match check_profanity(new_answer.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let answer = NewAnswer {
        content,
        question_id: new_answer.question_id,
    };

    match store.add_answer(answer).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
