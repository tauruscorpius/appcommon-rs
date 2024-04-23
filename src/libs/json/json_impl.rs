extern crate serde_json;

use serde::{Serialize, Deserialize};

pub fn marshal<T>(data : &T) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    where T : Serialize
{
    match serde_json::to_string_pretty(&data) {
        Ok(t) => {
            Ok(t)
        }
        _ => {
            Err("to string failed")?
        }
    }
}

pub fn unmarshal<'de, T>(stx: &'de String, data : &mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
       where T : Deserialize<'de>
{
    let r: serde_json::Result<T> = serde_json::from_str(stx.as_str());
    return match r {
        Ok(v) => {
            *data = v;
            Ok(())
        }
        Err(e) => {
            Err(Box::try_from(e).unwrap())
        }
    }
}