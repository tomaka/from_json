/*!

# from_json

Crate that allows you to easily turn JSON into objects. It is easier to use
and cleaner than the standard `Decodable` trait.

See the documentation of `from_json_macros` for a real-life example.

*/
#![deny(missing_docs)]

extern crate rustc_serialize as serialize;

pub use serialize::json::Json;
use std::collections::HashMap;

/// Error that can be triggered while building an object from Json.
#[derive(Debug)]
pub enum FromJsonError {
    /// The decoder expected an element of a type and got another type.
    ///
    /// The first parameter is the expected type, the second parameter is the Json object.
    ExpectError(&'static str, Json),

    /// A field could not be found while attempting to read a structure.
    ///
    /// The first parameter is the field name.
    FieldNotFound(&'static str, Json),
}

/// Trait that attempts to read an object from a JSON object.
pub trait FromJson {
    /// Builds the object from JSON.
    fn from_json(&Json) -> Result<Self, FromJsonError>;
}

#[macro_export]
macro_rules! derive_from_json {
    ($struct_name:ident, $($rest:tt)*) => (
        impl $crate::FromJson for $struct_name {
            fn from_json(input: &$crate::Json) -> Result<Self, $crate::FromJsonError> {
                if input.is_object() {
                    Ok(derive_from_json!(__all input $struct_name, $($rest)+))
                } else {
                    Err($crate::FromJsonError::ExpectError(stringify!($struct_name), input.clone()))
                }
            }
        }
    );

    (__all $input:ident $struct_name:ident, $($rest:tt)+) => (
        derive_from_json!(__parse_member $input [$struct_name,] $($rest)+,)
    );

    (__parse_member $i:ident [$s:ident, $($f:ident:$v:expr),*] $field:ident, $($rest:tt)*) => (
        derive_from_json!(__parse_member $i [$s, $($f:$v),*] $field as stringify!($field), $($rest)*)
    );

    (__parse_member $input:ident [$s:ident, $($f:ident:$v:expr),*] $field:ident as $name:expr, $($rest:tt)*) => (
        derive_from_json!(__parse_member $input [$s $(,$f:$v)*, $field: {
            match $input.find($name) {
                Some(elem) => match $crate::FromJson::from_json(elem) {
                    Ok(value) => value,
                    Err(e) => return Err(e)
                },
                None => match $crate::FromJson::from_json(&$crate::Json::Null) {
                    Ok(value) => value,
                    Err($crate::FromJsonError::ExpectError(_, _)) => return Err(
                        $crate::FromJsonError::FieldNotFound($name, $input.clone())),
                    Err(e) => return Err(e)
                }
            }
        }] $($rest)*)
    );

    (__parse_member $i:ident [$s:ident,]) => (
        $s
    );

    (__parse_member $i:ident [$s:ident, $($f:ident:$v:expr),+]) => (
        $s {
            $(
                $f: $v
            ),+
        }
    );
}

macro_rules! number_impl(
    ($t:ty) => (
        impl FromJson for $t {
            fn from_json(input: &Json) -> Result<$t, FromJsonError> {
                //use std::num::Bounded;

                //let my_min: $t = Bounded::min_value();
                //let my_max: $t = Bounded::max_value();

                match input {
                    // TODO: find out why this doesn't work
                    /*&Json::I64(value) if value >= my_min as i64 && value <= my_max as i64 => Ok(value as $t),
                    &Json::U64(value) if value <= my_max as u64 => Ok(value as $t),
                    &Json::F64(value) if value >= my_min as f64 && value <= my_max as f64 => Ok(value as $t),*/

                    &Json::I64(value) => Ok(value as $t),
                    &Json::U64(value) => Ok(value as $t),
                    &Json::F64(value) => Ok(value as $t),
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
    fn from_json(input: &Json) -> Result<bool, FromJsonError> {
        match input {
            &Json::Boolean(value) => Ok(value),
            _ => Err(FromJsonError::ExpectError("boolean", input.clone()))
        }
    }
}

impl FromJson for String {
    fn from_json(input: &Json) -> Result<String, FromJsonError> {
        match input {
            &Json::String(ref value) => Ok(value.clone()),
            _ => Err(FromJsonError::ExpectError("string", input.clone()))
        }
    }
}

impl<T: FromJson> FromJson for Option<T> {
    fn from_json(input: &Json) -> Result<Option<T>, FromJsonError> {
        match input {
            &Json::Null => return Ok(None),
            _ => ()
        };

        FromJson::from_json(input).map(|v| Some(v))
    }
}

impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(input: &Json) -> Result<Vec<T>, FromJsonError> {
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
    fn from_json(input: &Json) -> Result<HashMap<String, T>, FromJsonError> {
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
