use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use serde::{Deserialize, Serialize};

use warp::{
    Filter, 
    http::Method,
    http::StatusCode,
};

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Store {
        Store { 
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

mod error {
    use warp::{
        filters::{
            body::BodyDeserializeError,
            cors::CorsForbidden,
        },
        reject::Reject,
        Rejection,
        Reply,
        http::StatusCode,
    };

    #[derive(Debug)]
    pub enum Error {
        ParseError(std::num::ParseIntError),
        MissingParameters,
        QuestionNotFound,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // Why *self and not just self (&self)?
            // Would it then only match on a reference?
            match *self {
                Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
                Error::MissingParameters => write!(f, "Missing parameter"),
                Error::QuestionNotFound => write!(f, "Question not Found"),
            }
        }
    }

    impl Reject for Error {}

    #[derive(Debug)]
    struct InvalidId;
    impl Reject for InvalidId {}

    // Any error that doesn't match the first two conditions will return
    // "Route not found". Not ideal.
    pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
        if let Some(error) = r.find::<Error>() {
            Ok(warp::reply::with_status(
                error.to_string(),
                StatusCode::RANGE_NOT_SATISFIABLE,
            ))
        } else if let Some(error) = r.find::<CorsForbidden>() {
            Ok(warp::reply::with_status(
                error.to_string(),
                StatusCode::FORBIDDEN,
            ))
        } else if let Some(error) = r.find::<BodyDeserializeError>() {
            Ok(warp::reply::with_status(
                error.to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            ))
        } else {
            Ok(warp::reply::with_status(
                "Route not found".to_string(),
                StatusCode::NOT_FOUND,
            ))
        }
}

}



#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct AnswerId(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);



// TODO:
// What happens if we specify an end parameter which is greater than the length of our vector? 
// And what happens if start is 20 and end is 10?
fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, error::Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(error::Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(error::Error::ParseError)?,
        });
    }
    Err(error::Error::MissingParameters)
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.read().values().cloned().collect();
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.read().values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(
    store: Store, 
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    store
        .questions
        .write()
        .insert(question.id.clone(), question);

    Ok(warp::reply::with_status(
        "Question added", 
        StatusCode::OK,
    ))
}

async fn update_question(
    id: String,
    store: Store,
    question: Question
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }

    Ok(warp::reply::with_status(
        "Question updated", 
        StatusCode::OK,
    ))
}

async fn delete_question(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().remove(&QuestionId(id)) {
        Some(_) => return Ok(warp::reply::with_status(
            "Question deleted", StatusCode::OK)),
        None => return Err(warp::reject::custom(error::Error::QuestionNotFound))
    }
}

// TODO:
// Create a random, unique ID instead of the one by hand
// Add error handling if the fields we require aren't present
// Check if a question exists
// Change route to answers: /questions/:questionId/answers 
async fn add_answer(
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

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(error::return_error);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
