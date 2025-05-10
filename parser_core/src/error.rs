//! Error handling for the parser.

use std::fmt;
use thiserror::Error;

/// Result type for parsing operations.
pub type Result<T> = std::result::Result<T, ParseError>;

/// Error type for parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Syntax error at {location}: {message}")]
    Syntax {
        location: SourceLocation,
        message: String,
    },
    
    #[error("Type error at {location}: {message}")]
    Type {
        location: SourceLocation,
        message: String,
    },
    
    #[error("Scope error at {location}: {message}")]
    Scope {
        location: SourceLocation,
        message: String,
    },
    
    #[error("ASG build error: {0}")]
    AsgBuild(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("{0}")]
    Other(String),
}

/// Represents a location in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub filename: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.filename {
            Some(filename) => write!(f, "{}:{}:{}", filename, self.line, self.column),
            None => write!(f, "line {}, column {}", self.line, self.column),
        }
    }
}

/// Converts a lalrpop error into our custom ParseError.
impl<'input> From<lalrpop_util::ParseError<usize, (usize, &'input str), &'static str>> for ParseError {
    fn from(err: lalrpop_util::ParseError<usize, (usize, &'input str), &'static str>) -> Self {
        match err {
            lalrpop_util::ParseError::InvalidToken { location } => {
                // Convert location to line:column (simplified)
                ParseError::Syntax {
                    location: SourceLocation {
                        filename: None,
                        line: 1, // Would need to compute actual line
                        column: location,
                    },
                    message: "Invalid token".to_string(),
                }
            }
            lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => {
                ParseError::Syntax {
                    location: SourceLocation {
                        filename: None,
                        line: 1, // Would need to compute actual line
                        column: location,
                    },
                    message: format!("Unexpected end of file, expected: {}", expected.join(", ")),
                }
            }
            lalrpop_util::ParseError::UnrecognizedToken { token: (start, token, end), expected } => {
                ParseError::Syntax {
                    location: SourceLocation {
                        filename: None,
                        line: 1, // Would need to compute actual line
                        column: start,
                    },
                    message: format!("Unexpected token '{}', expected: {}", token, expected.join(", ")),
                }
            }
            lalrpop_util::ParseError::ExtraToken { token: (start, token, _) } => {
                ParseError::Syntax {
                    location: SourceLocation {
                        filename: None,
                        line: 1, // Would need to compute actual line
                        column: start,
                    },
                    message: format!("Extra token: '{}'", token),
                }
            }
            lalrpop_util::ParseError::User { error } => {
                ParseError::Other(error.to_string())
            }
        }
    }
}