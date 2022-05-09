pub fn parse_key_value_pair(v: &str) -> (String, String) {
    let mut it = v.split('=');
    let key = it.next().expect("already validated").to_owned();
    let value = it.next().expect("already validated").to_owned();

    (key, value)
}

pub fn is_key_value_pair(v: &str) -> Result<(), String> {
    let kv: Vec<_> = v.split('=').collect();

    match (kv.get(0), kv.get(1)) {
        (Some(key), Some(value)) if !key.is_empty() && !value.is_empty() => Ok(()),
        _ => Err(String::from("tag has to be a <key>=<value> pair")),
    }
}
