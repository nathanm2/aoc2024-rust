use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub fn read_ints<T>(path: String) -> Result<Vec<T>, Box<dyn Error>>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + 'static,
{
    let mut results = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        results.push(line?.parse::<T>()?);
    }
    Ok(results)
}
