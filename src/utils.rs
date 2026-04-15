use anyhow::Result;

pub fn get_current_timestamp() -> Result<u64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs())
}

pub fn filter_none_string(text: impl Into<String>) -> Option<String> {
    let text: String = text.into();

    if !text.is_empty() { Some(text) } else { None }
}
