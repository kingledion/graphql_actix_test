//! Library for testing graphql endpoints with the actix-web framework
//! 
//! This library is designed to process graphql as the result of a test. The test framework 
//! wraps an assertion testing capability and takes as arguments an initial setup function, 
//! definitions of a repository,repository setup data, arguments to the graphql schema, an
//! execution function, and expected results of the execution schema. 
//! 
//! In addition to the test framework, there are helper structures. A reciever structure 
//! unpacks the expected data and errors from a graphql response. Argument and Expected 
//! structures pass the arguments to a graphql schema and test the expeccted output values 
//! (including http status code, data and errors).
#![warn(missing_docs)]

use serde::Deserialize;
use serde_json::Value;

use actix_web::http::StatusCode;
use actix_web::dev::ServiceResponse;
use actix_web::test;

/// A struct for deserializing a GraphQL response according to GraphQL specification
#[derive(Deserialize, Debug)]
pub struct GraphQLResponseReciever<T: PartialEq> {
    /// The data specified by this struct's type paramter. May be None. 
    pub data: Option<T>,
    /// A vector of error struct. May be None. 
    pub errors: Option<Vec<GraphQLResponseError>>,
}

impl<T: PartialEq> GraphQLResponseReciever<T> {
    /// A convenience function for unwrapping and returning the data member. Will panic if the
    /// data is none; should be used for testing when a value is expected and a panic indicates
    /// a failed test. 
    pub fn get_data(&self) -> &T {
        self.data.as_ref().unwrap()
    }

    /// A convenience function for returning the error messages. Will return a vector of the
    /// 'message' fields from all errors, with order maintained. If the optional errors field 
    /// is None, then an empty vector is returned. 
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

/// A struct for deserializing an GraphQl error message according to GraphQL specification. Only 
/// the 'message' field is implemented; 'locations' and 'paths' are ignored.
#[derive(Deserialize, Debug)]
pub struct GraphQLResponseError {
    /// A string error message 
    pub message: String,
    // locations field is not retrieved or compared in this context
    // paths field is not retrieved or compared in this context
}

/// A struct for passing the arguments to a GraphQL schema. The arguments consist of HTTP headers
/// and a payload. 
pub struct Argument{
    /// A vector of header tuples, which consist of a pair of strings. 
    pub headers: Vec<(String, String)>,
    /// A string graphql payload. 
    pub payload: String,
}

/// A struct for defining the expected output of a GraphQL schema. Expected results consist of 
/// an http status code, am optional vector of error messages, and some optional data. 
pub struct Expected<V>{
    /// An http status code
    pub status: StatusCode,
    /// An optional vector of String error messages. This should correspond to the 'message' fields 
    /// of the array in the 'error' field, as defined in a GraphQL schema response map. 
    pub errmsg: Option<Vec<String>>,
    /// An optional data of the struct's type pa
    pub data: Option<V>,
}

/// Executes tests against a defined environment using the actix_web framework.
/// 
/// Requires the following type parameters:
/// - `FI` : An initializing function, which takes no arguments and returns no parameters. This can
/// be used to execute code that is expected to run only one time across all parallel tests. 
/// - `FR` : A function to initialize the repository. This function must take as an argument 
/// an optional JSON deserialziable data structure to be set as data in the repo. Returns `FutR`.
/// - `FutR` : A future that resolves to a repository of type `R`.
/// - `R` :  A repository; there are no restrictions on this type but it will be passed as argument
/// to `FE`. 
/// - `FE` : An executing function that will run the test schema. Takes an [Argument] as argument
/// and returns `FutE`.
/// - `FutE` : A future that resolves to an [actix_web::dev::ServiceResponse](https://docs.rs/actix-web/latest/actix_web/dev/struct.ServiceResponse.html)
/// - `V` : The data type returned by the schema being tested by this framework. 
/// 
/// Takes the following function arguments:
/// - `init_func` : An initializing function of type `FI`.
/// - `repo_func` : A fuction to initialize the repository of type `FR`.
/// - `repo_data` : Optional data used to initialize the repository. Must be a JSON deserializable 
/// data structure. 
/// - `arg` : [Argument] that is passed to the executing function
/// - `exec_func` : An executing function of type `FE`.
/// - `exp` : [Expected] return of the function, with any data of type `V`. 
/// 
/// This function will execute the test with the defined initialization function, initialized 
/// repository and arguments. Compares the resulting GraphQL response to the expected values using
/// a series of asserts, which prints results from any test failures.
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

        match exp.data {
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

