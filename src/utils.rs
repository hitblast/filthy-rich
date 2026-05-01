use crate::errors::InnerParsingError;

pub fn get_current_timestamp() -> Result<u64, InnerParsingError> {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| InnerParsingError::from(e))?
        .as_secs();

    Ok(s)
}

pub fn filter_none_string(text: impl Into<String>) -> Option<String> {
    let text: String = text.into();

    if !text.is_empty() { Some(text) } else { None }
}
