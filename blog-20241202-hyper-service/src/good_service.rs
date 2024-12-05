use crate::error::MyError;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Method, Request, Response};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

/// An example of a good tower-esque service that can be tested
#[derive(Clone)]
pub struct GoodTowerService {}

impl<BODY> hyper::service::Service<Request<BODY>> for GoodTowerService
where
    BODY: hyper::body::Body<Data=Bytes> + Send + 'static,
    BODY::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<String>;
    type Error = MyError;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<BODY>) -> Self::Future {
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            match parts.method {
                Method::GET => {
                    Ok(Response::new("test".to_string()))
                }
                Method::POST => {
                    match body.collect().await {
                        Ok(the_body) => {
                            Ok(Response::new(String::from_utf8(the_body.to_bytes().to_vec()).unwrap()))
                        }
                        Err(e) => {
                            Err(MyError { payload: "unexpected body error".to_string() })
                        }
                    }
                }
                _ => {
                    return Err(MyError { payload: "Method not allowed".to_string() })
                }
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::good_service::GoodTowerService;
    use hyper::service::Service;

    #[tokio::test]
    async fn test_endpoint() {
        let service = GoodTowerService {};

        let body = http_body_util::Full::from("simple request");
        let req = hyper::Request::builder()
            .method("POST")
            .uri("http://any-url:12345")
            .body(body)
            .unwrap();
        let resp = service.call(req).await.unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.body(), "simple request");
    }
}