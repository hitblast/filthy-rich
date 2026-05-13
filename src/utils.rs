use crate::errors::InnerParsingError;

pub fn get_current_timestamp() -> Result<u64, InnerParsingError> {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    Ok(s)
}
