use std::collections::HashMap;

use crate::error;

#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

// TODO:
// What happens if we specify an end parameter which is greater than the length of our vector? 
// And what happens if start is 20 and end is 10?
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, error::Error> {
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