use serde::Deserialize;
use serde_json::Value;

use actix_web::http::StatusCode;
use actix_web::dev::ServiceResponse;
use actix_web::test;


#[derive(Deserialize, Debug)]
pub struct GraphQLResponseReciever<T: PartialEq> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLResponseError>>,
}

impl<T: PartialEq> GraphQLResponseReciever<T> {
    pub fn get_data(&self) -> &T {
        self.data.as_ref().unwrap()
    }

    pub fn get_messages(&self) -> Vec<String> {
        match &self.errors {
            Some(s) => s
                .iter()
                .map(|gre: &GraphQLResponseError| &gre.message)
                .cloned()
                .collect(),
            None => {
                vec![]
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GraphQLResponseError {
    pub message: String,
    // locations field is not retrieved or compared in this context
}

pub struct Argument{
    pub headers: Vec<(String, String)>,
    pub payload: String,
}

pub struct Expected<V>{
    pub status: StatusCode,
    pub errmsg: Option<Vec<String>>,
    pub value: Option<V>,
}

pub async fn test_framework<'a, FI, FR, FutR, R, FE, FutE, V> (
    init_func: FI,
    repo_func: FR,
    repo_data: Option<&'a mut [Value]>,
    arg: Argument,
    exec_func: FE, 
    exp: Expected<V>,
) where 
    FI: Fn(),
    FR: Fn(Option<&'a mut [Value]>) -> FutR,
    FutR: std::future::Future<Output = R>,
    FE: Fn(R, Argument) -> FutE,
    FutE: std::future::Future<Output = ServiceResponse>,
    V: serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    init_func();

    let repo: R = repo_func(repo_data).await;
    let response = exec_func(repo, arg).await;

    // validate status is expected
    let got_status = response.status();

    assert_eq!(
        got_status,
        exp.status,
        "Got unexpected status {}, expected {}; body: {:?}",
        got_status,
        exp.status,
        test::read_body(response).await
    );

    // validate error or return, if required
    if got_status == StatusCode::OK {
        // success case
        let got: GraphQLResponseReciever<V> = test::read_body_json(response).await;

        match exp.errmsg {
            Some(errmsg) => assert_eq!(got.get_messages(), errmsg),
            None => {}
        };

        match exp.value {
            Some(v) => assert_eq!(got.get_data(), &v),
            None => {}
        };
    } else {
        // error case

        let exp_err = &exp.errmsg
            .expect("Expected an error message in case where status does is not 200 OK")[0];

        let got_bytes = test::read_body(response).await;
        let got_err = std::str::from_utf8(&got_bytes).unwrap();

        assert_eq!(got_err, exp_err);
    }
}

