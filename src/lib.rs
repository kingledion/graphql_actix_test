use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GraphQLResponseReciever<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLResponseError>>,
}

impl<T> GraphQLResponseReciever<T> {
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
pub struct GoodsReciever<T> {
    pub goods: T,
}

#[derive(Deserialize, Debug)]
pub struct GraphQLResponseError {
    pub message: String,
    // locations field is not retrieved or compared in this context
}
