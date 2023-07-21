use crate::{
	board::Layer,
	common::Point,
	internal::{rename, tuple, tuple_or_default},
	mm
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename = "segment")]
pub struct Segment {
	#[serde(with = "rename::start")]
	pub start: Point,

	#[serde(with = "rename::end")]
	pub end: Point,

	#[serde(with = "tuple")]
	pub width: mm,

	pub layer: Layer,

	#[serde(with = "tuple")]
	pub net: u8,

	#[serde(with = "tuple_or_default", skip_serializing_if = "crate::skip_uuid")]
	pub tstamp: Uuid
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{sexpr_test_case, Unit};

	sexpr_test_case! {
		name: segment,
		input: r#"(segment (start 1 0) (end 2 0) (width 0.12) (layer "F.Cu") (net 1) (tstamp "00000000-0000-0000-0000-000000000000"))"#,
		value: Segment {
			start: Point::new(1.0.mm(), 0.0.mm()),
			end: Point::new(2.0.mm(), 0.0.mm()),
			width: 0.12.mm(),
			layer: Layer(String::from("F.Cu")),
			net: 1,
			tstamp: Uuid::nil()
		}
	}
}
