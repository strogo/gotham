//! Helpers for HTTP response generation

use hyper::{Method, Response, StatusCode};
use hyper::header::{ContentLength, ContentType};
use mime::Mime;

use state::{request_id, FromState, State};
use http::header::{XContentTypeOptions, XFrameOptions, XRequestId, XXssProtection};

type Body = (Vec<u8>, Mime);

/// Creates a `Response` object and populates it with a set of default headers that help to improve
/// security and conformance to best practice.
///
/// `create_response` utilises `extend_response`, which delegates to `set_headers` for setting
/// security headers. See `set_headers` for information about the headers which are populated.
///
/// # Examples
///
/// ```rust
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate mime;
/// #
/// # use hyper::{Response, StatusCode};
/// # use hyper::header::{ContentLength, ContentType};
/// # use gotham::state::State;
/// # use gotham::http::response::create_response;
/// # use gotham::http::header::XRequestId;
/// # use gotham::test::TestServer;
/// #
/// static BODY: &'static [u8] = b"Hello, world!";
///
/// fn handler(state: State) -> (State, Response) {
///     let response = create_response(
///         &state,
///         StatusCode::Ok,
///         Some((BODY.to_vec(), mime::TEXT_PLAIN)),
///     );
///
///     (state, response)
/// }
/// #
/// # fn main() {
/// #     let test_server = TestServer::new(|| Ok(handler)).unwrap();
/// #     let response = test_server
/// #         .client()
/// #         .get("http://example.com/")
/// #         .perform()
/// #         .unwrap();
/// #
/// #     assert_eq!(response.status(), StatusCode::Ok);
/// #     assert!(response.headers().get::<XRequestId>().is_some());
/// #
/// #     assert_eq!(
/// #         *response.headers().get::<ContentType>().unwrap(),
/// #         ContentType(mime::TEXT_PLAIN)
/// #     );
/// #
/// #     assert_eq!(
/// #         *response.headers().get::<ContentLength>().unwrap(),
/// #         ContentLength(BODY.len() as u64)
/// #     );
/// # }
/// ```
pub fn create_response(state: &State, status: StatusCode, body: Option<Body>) -> Response {
    let mut res = Response::new();
    extend_response(state, &mut res, status, body);
    res
}

/// Extends a `Response` object with an optional body and set of default headers that help to
/// improve security and conformance to best practice.
///
/// `extend_response` delegates to `set_headers` for setting security headers. See `set_headers`
/// for information about the headers which are populated.
///
/// # Examples
///
/// ```rust
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate mime;
/// #
/// # use hyper::{Response, StatusCode};
/// # use hyper::header::{ContentLength, ContentType};
/// # use gotham::state::State;
/// # use gotham::http::response::extend_response;
/// # use gotham::http::header::XRequestId;
/// # use gotham::test::TestServer;
/// #
/// static BODY: &'static [u8] = b"Hello, world!";
///
/// fn handler(state: State) -> (State, Response) {
///     let mut response = Response::new();
///
///     extend_response(
///         &state,
///         &mut response,
///         StatusCode::Ok,
///         Some((BODY.to_vec(), mime::TEXT_PLAIN)),
///     );
///
///     (state, response)
/// }
/// #
/// # fn main() {
/// #     let test_server = TestServer::new(|| Ok(handler)).unwrap();
/// #     let response = test_server
/// #         .client()
/// #         .get("http://example.com/")
/// #         .perform()
/// #         .unwrap();
/// #
/// #     assert_eq!(response.status(), StatusCode::Ok);
/// #     assert!(response.headers().get::<XRequestId>().is_some());
/// #
/// #     assert_eq!(
/// #         *response.headers().get::<ContentType>().unwrap(),
/// #         ContentType(mime::TEXT_PLAIN)
/// #     );
/// #
/// #     assert_eq!(
/// #         *response.headers().get::<ContentLength>().unwrap(),
/// #         ContentLength(BODY.len() as u64)
/// #     );
/// # }
/// ```
pub fn extend_response(state: &State, res: &mut Response, status: StatusCode, body: Option<Body>) {
    if usize::max_value() > u64::max_value() as usize {
        error!(
            "[{}] unable to handle content_length of response, outside u64 bounds",
            request_id(state)
        );
        panic!(
            "[{}] unable to handle content_length of response, outside u64 bounds",
            request_id(state)
        );
    }

    match body {
        Some((body, mime)) => {
            set_headers(state, res, Some(mime), Some(body.len() as u64));
            res.set_status(status);

            match *Method::borrow_from(state) {
                Method::Head => (),
                _ => res.set_body(body),
            };
        }
        None => {
            set_headers(state, res, None, None);
            res.set_status(status);
        }
    };
}

/// Sets a number of default headers in a `Response` that ensure security and conformance to
/// best practice.
///
/// # Examples
///
/// When `Content-Type` and `Content-Length` are not provided, only the security headers are set on
/// the response.
///
/// ```rust
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate mime;
/// #
/// # use hyper::{Response, StatusCode};
/// # use gotham::state::State;
/// # use gotham::http::response::set_headers;
/// # use gotham::http::header::*;
/// # use gotham::test::TestServer;
/// #
/// fn handler(state: State) -> (State, Response) {
///     let mut response = Response::new().with_status(StatusCode::Accepted);
///
///     set_headers(
///         &state,
///         &mut response,
///         None,
///         None,
///     );
///
///     (state, response)
/// }
///
/// # fn main() {
/// // Demonstrate the returned headers by making a request to the handler.
/// let test_server = TestServer::new(|| Ok(handler)).unwrap();
/// let response = test_server
///     .client()
///     .get("http://example.com/")
///     .perform()
///     .unwrap();
///
/// assert_eq!(response.status(), StatusCode::Accepted);
///
/// // e.g.:
/// // X-Request-Id: 848c651a-fdd8-4859-b671-3f221895675e
/// assert!(response.headers().get::<XRequestId>().is_some());
///
/// // X-Frame-Options: DENY
/// assert_eq!(
///     *response.headers().get::<XFrameOptions>().unwrap(),
///     XFrameOptions::Deny,
/// );
///
/// // X-XSS-Protection: 1; mode=block
/// assert_eq!(
///     *response.headers().get::<XXssProtection>().unwrap(),
///     XXssProtection::EnableBlock,
/// );
///
/// // X-Content-Type-Options: nosniff
/// assert_eq!(
///     *response.headers().get::<XContentTypeOptions>().unwrap(),
///     XContentTypeOptions::NoSniff,
/// );
/// # }
/// ```
///
/// When the `Content-Type` and `Content-Length` are included, the headers are set in addition to
/// the security headers.
///
/// ```rust
/// # extern crate gotham;
/// # extern crate hyper;
/// # extern crate mime;
/// #
/// # use hyper::{Response, StatusCode};
/// # use hyper::header::{ContentLength, ContentType};
/// # use gotham::state::State;
/// # use gotham::http::response::set_headers;
/// # use gotham::http::header::*;
/// # use gotham::test::TestServer;
/// #
/// static BODY: &'static [u8] = b"Hello, world!";
///
/// fn handler(state: State) -> (State, Response) {
///     let mut response = Response::new().with_status(StatusCode::Ok).with_body(BODY.to_vec());
///
///     set_headers(
///         &state,
///         &mut response,
///         Some(mime::TEXT_PLAIN),
///         Some(BODY.len() as u64),
///     );
///
///     (state, response)
/// }
///
/// # fn main() {
/// // Demonstrate the returned headers by making a request to the handler.
/// let test_server = TestServer::new(|| Ok(handler)).unwrap();
/// let response = test_server
///     .client()
///     .get("http://example.com/")
///     .perform()
///     .unwrap();
///
/// assert_eq!(response.status(), StatusCode::Ok);
///
/// assert_eq!(
///     *response.headers().get::<ContentType>().unwrap(),
///     ContentType(mime::TEXT_PLAIN)
/// );
///
/// assert_eq!(
///     *response.headers().get::<ContentLength>().unwrap(),
///     ContentLength(BODY.len() as u64)
/// );
/// #
/// # // e.g.:
/// # // X-Request-Id: 848c651a-fdd8-4859-b671-3f221895675e
/// # assert!(response.headers().get::<XRequestId>().is_some());
/// #
/// # // X-Frame-Options: DENY
/// # assert_eq!(
/// #     *response.headers().get::<XFrameOptions>().unwrap(),
/// #     XFrameOptions::Deny,
/// # );
/// #
/// # // X-XSS-Protection: 1; mode=block
/// # assert_eq!(
/// #     *response.headers().get::<XXssProtection>().unwrap(),
/// #     XXssProtection::EnableBlock,
/// # );
/// #
/// # // X-Content-Type-Options: nosniff
/// # assert_eq!(
/// #     *response.headers().get::<XContentTypeOptions>().unwrap(),
/// #     XContentTypeOptions::NoSniff,
/// # );
/// # }
/// ```
pub fn set_headers(state: &State, res: &mut Response, mime: Option<Mime>, length: Option<u64>) {
    let headers = res.headers_mut();

    match length {
        Some(length) => headers.set(ContentLength(length)),
        None => headers.set(ContentLength(0)),
    }

    match mime {
        Some(mime) => headers.set(ContentType(mime)),
        None => (),
    };

    headers.set(XRequestId(request_id(state).into()));
    headers.set(XFrameOptions::Deny);
    headers.set(XXssProtection::EnableBlock);
    headers.set(XContentTypeOptions::NoSniff);
}
