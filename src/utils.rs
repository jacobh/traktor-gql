use std;

pub fn parse_option_str<T>(x: String) -> Option<T>
    where T: std::str::FromStr
{
    match x.parse::<T>() {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}
