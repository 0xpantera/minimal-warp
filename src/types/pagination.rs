use handle_errors::Error;
use std::collections::HashMap;

/// Pagination struct which is getting extracted
/// from query params.
#[derive(Default, Debug)]
pub struct Pagination {
    /// The index of the last item which has to be returned
    pub limit: Option<u32>,
    /// The index of the first item which has to be returned
    pub offset: u32,
}

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `/questions?start=1&end=10`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = types::pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.limit, 1);
/// assert_eq!(p.offset, 10);
/// ```
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // TODO:
    // What happens if we specify an end parameter which is greater than the length of our vector?
    // And what happens if start is 20 and end is 10?
    if params.contains_key("limit") && params.contains_key("offset") {
        return Ok(Pagination {
            // Takes the "limit" parameter in the query and tries to convert it to a number
            limit: Some(params
                .get("limit")
                .unwrap()
                .parse::<u32>()
                .map_err(Error::ParseError)?),
            // Takes the "offset" parameter in the query and tries to convert it to a number
            offset: params
                .get("offset")
                .unwrap()
                .parse::<u32>()
                .map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
}
