use std;

pub fn parse_option_str<T>(x: String) -> Option<T>
    where T: std::str::FromStr
{
    match x.parse::<T>() {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn get_env_var(key: &str) -> Option<String> {
    for (k, v) in std::env::vars() {
        if k == key {
            return Some(v);
        }
    }
    None
}
