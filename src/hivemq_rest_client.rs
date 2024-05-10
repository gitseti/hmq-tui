use hivemq_openapi::apis::Error;
use hivemq_openapi::{apis::configuration::Configuration, models::PaginationCursor};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

pub fn get_cursor(links: Option<Option<Box<PaginationCursor>>>) -> Option<String> {
    lazy_static! {
        static ref CURSOR_REGEX: Regex = Regex::new(r"cursor=([^&]*)").unwrap();
    }

    let next = links.flatten().and_then(|cursor| cursor.next)?;

    CURSOR_REGEX
        .captures_iter(&next)
        .next()
        .and_then(|cap| cap.get(1))
        .map(|mat| mat.as_str().to_string())
}

pub fn build_rest_api_config(host: String) -> Configuration {
    let mut configuration = Configuration::default();
    configuration.base_path = host.to_string();
    configuration
}

pub fn transform_api_err<T: Serialize>(error: Error<T>) -> String {
    let message = if let Error::ResponseError(response) = error {
        match &response.entity {
            None => response.content.clone(),
            Some(entity) => serde_json::to_string_pretty(entity).expect("Can not serialize entity"),
        }
    } else {
        error.to_string()
    };

    format!("API request failed: {}", message)
}

#[cfg(test)]
pub(crate) mod tests {
    use hivemq_openapi::models::PaginationCursor;
    use httpmock::{Method::GET, Mock, MockServer};
    use serde::Serialize;

    pub fn create_responses<T>(
        url: &str,
        build_list: fn(usize, usize, Option<Option<Box<PaginationCursor>>>) -> T,
    ) -> Vec<T> {
        let mut responses: Vec<T> = Vec::with_capacity(100);
        for i in 0..10 {
            let start = i * 10;
            let end = start + 10;
            let cursor = if i != 9 {
                Some(Some(Box::new(PaginationCursor {
                    next: Some(format!("{url}?cursor=foobar{}", i + 1)),
                })))
            } else {
                None
            };
            responses.push(build_list(start, end, cursor));
        }
        responses
    }

    pub fn mock_cursor_responses<'a, T: Serialize>(
        broker: &'a MockServer,
        url: &str,
        responses: &Vec<T>,
        cursor_prefix: &str,
    ) -> Vec<Mock<'a>> {
        let mut mocks = Vec::with_capacity(responses.len());
        for (i, response) in responses.iter().enumerate().rev() {
            let mock = broker.mock(|when, then| {
                if i == 0 {
                    when.method(GET).path(url);
                } else {
                    when.method(GET)
                        .path(url)
                        .query_param("cursor", format!("{cursor_prefix}{i}"));
                }
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&response).unwrap());
            });
            mocks.push(mock);
        }

        mocks
    }
}
