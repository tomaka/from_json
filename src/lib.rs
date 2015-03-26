/*!

# from_json

Crate that allows you to easily turn JSON into objects. It is easier to use
and cleaner than the standard `Decodable` trait.

See the documentation of `from_json_macros` for a real-life example.

*/

#![deny(missing_docs)]
#![deny(warnings)]

extern crate rustc_serialize as serialize;

use serialize::json;
use std::collections::HashMap;

/// Error that can be triggered while building an object from Json.
#[derive(Debug)]
pub enum FromJsonError {
    /// The decoder expected an element of a type and got another type.
    ///
    /// The first parameter is the expected type, the second parameter is the Json object.
    ExpectError(&'static str, json::Json),

    /// A field could not be found while attempting to read a structure.
    ///
    /// The first parameter is the field name.
    FieldNotFound(&'static str, json::Json),
}

/// Trait that attempts to read an object from a JSON object.
pub trait FromJson {
    /// Builds the object from JSON.
    fn from_json(&json::Json) -> Result<Self, FromJsonError>;
}

macro_rules! number_impl(
    ($t:ty) => (
        impl FromJson for $t {
            fn from_json(input: &json::Json) -> Result<$t, FromJsonError> {
                //use std::num::Bounded;

                //let my_min: $t = Bounded::min_value();
                //let my_max: $t = Bounded::max_value();

                match input {
                    // TODO: find out why this doesn't work
                    /*&json::Json::I64(value) if value >= my_min as i64 && value <= my_max as i64 => Ok(value as $t),
                    &json::Json::U64(value) if value <= my_max as u64 => Ok(value as $t),
                    &json::Json::F64(value) if value >= my_min as f64 && value <= my_max as f64 => Ok(value as $t),*/

                    &json::Json::I64(value) => Ok(value as $t),
                    &json::Json::U64(value) => Ok(value as $t),
                    &json::Json::F64(value) => Ok(value as $t),
                    _ => Err(FromJsonError::ExpectError("integer", input.clone()))
                }
            }
        }
    )
);

number_impl!(isize);
number_impl!(usize);
number_impl!(u8);
number_impl!(i8);
number_impl!(u16);
number_impl!(i16);
number_impl!(u32);
number_impl!(i32);
number_impl!(u64);
number_impl!(i64);
number_impl!(f32);
number_impl!(f64);

impl FromJson for bool {
    fn from_json(input: &json::Json) -> Result<bool, FromJsonError> {
        match input {
            &json::Json::Boolean(value) => Ok(value),
            _ => Err(FromJsonError::ExpectError("boolean", input.clone()))
        }
    }
}

impl FromJson for String {
    fn from_json(input: &json::Json) -> Result<String, FromJsonError> {
        match input {
            &json::Json::String(ref value) => Ok(value.clone()),
            _ => Err(FromJsonError::ExpectError("string", input.clone()))
        }
    }
}

impl<T: FromJson> FromJson for Option<T> {
    fn from_json(input: &json::Json) -> Result<Option<T>, FromJsonError> {
        match input {
            &json::Json::Null => return Ok(None),
            _ => ()
        };

        FromJson::from_json(input).map(|v| Some(v))
    }
}

impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(input: &json::Json) -> Result<Vec<T>, FromJsonError> {
        if let Some(ref list) = input.as_array() {
            let mut result = Vec::with_capacity(list.len());

            for element in list.iter() {
                match FromJson::from_json(element) {
                    Ok(value) => result.push(value),
                    Err(e) => return Err(e)
                }
            }

            Ok(result)

        } else {
            Err(FromJsonError::ExpectError("list", input.clone()))
        }
    }
}

impl<T: FromJson> FromJson for HashMap<String, T> {
    fn from_json(input: &json::Json) -> Result<HashMap<String, T>, FromJsonError> {
        if let Some(ref object) = input.as_object() {
            let mut result = HashMap::new();
            
            for (key, element) in object.iter() {
                match FromJson::from_json(element) {
                    Ok(value) => result.insert(key.clone(), value),
                    Err(e) => return Err(e)
                };
            }

            Ok(result)

        } else {
            Err(FromJsonError::ExpectError("object", input.clone()))
        }
    }
}
