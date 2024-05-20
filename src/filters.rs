pub fn trunc_id<T: std::fmt::Display>(id: T) -> ::askama::Result<String> {
    let mut id = id.to_string();
    id.truncate(8);
    Ok(id)
}
