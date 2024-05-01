use crate::token::TokenLocation;

pub struct SourceError {
    pub message: String,
    pub location: TokenLocation,
}

pub type SourceResult<T> = Result<T, Box<SourceError>>;

pub fn source_error<T>(message: String, location: TokenLocation) -> SourceResult<T> {
    Err(Box::new(SourceError { message, location }))
}
