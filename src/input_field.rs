use std::any::Any;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use druid::{Event, EventCtx, InternalEvent, TimerToken, Widget};
use druid::text::{Formatter, Selection, Validation, ValidationError};
use druid::widget::Controller;
use crate::GrabData;

pub struct PositiveNumberFormatter;

impl Formatter<String> for PositiveNumberFormatter {
    fn format(&self, value: &String) -> String {
        value.clone()
    }

    fn format_for_editing(&self, value: &String) -> String {
        value.clone()
    }

    fn validate_partial_input(&self, text: &str, _: &Selection) -> Validation {
        if text.is_empty() {
            Validation::success()
        } else if let Ok(parsed) = text.parse::<usize>() {
            Validation::success()
        } else {
            Validation::failure(CustomError {
                message: "Invalid input".to_string(),
            })
        }
    }

    fn value(&self, input: &str) -> Result<String, ValidationError> {
        match input.parse::<usize>() {
            Ok(parsed) => Ok(parsed.to_string()),
            Err(parse) => {
                if parse.to_string() == "cannot parse integer from empty string" {
                    Ok("".to_string())
                } else {
                    Err(ValidationError::new(CustomError {
                        message: "Invalid input".to_string(),
                    }))
                }
            },
        }
    }

}

struct CustomError {
    message: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Error for CustomError {}