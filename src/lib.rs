#![recursion_limit = "128"]

/// serializes a type as a different repr type using the given conversion functions
#[macro_export]
macro_rules! serde_conv {
	($m:ident, $t:ty, /*$ser:expr,*/ $de:expr) => {
		pub mod $m {
			use serde::{/*Serialize, Serializer,*/ Deserialize, Deserializer};
			use super::*;
/*
			pub fn serialize<S: Serializer>(x: &$t, serializer: S) -> Result<S::Ok, S::Error> {
				let y = $ser(*x);
				y.serialize(serializer)
			}
*/
			pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<$t, D::Error> {
				let y = Deserialize::deserialize(deserializer)?;
				Ok($de(y))
			}
		}
	}
}

serde_conv!(serde_timestamp, DOMTimeStamp, |x: f64| x as _);
serde_conv!(serde_pos_err_code, PositionErrorCode, |x: u16| match x {
	1 => PositionErrorCode::PermissionDenied,
	2 => PositionErrorCode::PositionUnavailable,
	3 => PositionErrorCode::Timeout,
	_ => unreachable!()
});
js_deserializable!(Position);
js_deserializable!(Coordinates);
js_serializable!(PositionOptions);
js_deserializable!(PositionError);

use yew::{prelude::*, services::Task};
use stdweb::{*, unstable::TryInto};
use serde_derive::*;
use smart_default::*;

// https://w3c.github.io/geolocation-api/#idl-index

pub type DOMTimeStamp = u64;

#[derive(Debug, Deserialize)]
pub struct Position {
	pub coords: Coordinates,
	#[serde(with = "serde_timestamp")] // TODO: ensure it won't truncate
	pub timestamp: DOMTimeStamp, // TODO: if u64 is used directly, it causes: ConversionError { kind: Custom("invalid type: floating point 1535164942454, expected u64") }
}

#[derive(Debug, Deserialize)]
pub struct Coordinates {
	pub latitude: f64,
	pub longitude: f64,
	pub altitude: Option<f64>,
	pub accuracy: f64,
	#[serde(rename = "altitudeAccuracy")]
	pub altitude_accuracy: Option<f64>,
	pub heading: Option<f64>,
	pub speed: Option<f64>,
}

#[derive(Debug, Serialize, SmartDefault)]
pub struct PositionOptions {
	#[serde(rename = "enableHighAccuracy")]
	pub enable_high_accuracy: bool,
	#[default = "0xFFFFFFFF"]
	#[serde(rename = "timeout")]
	pub timeout_ms: u32,
	#[serde(rename = "maximumAge")]
	pub maximum_age: u32,
}

#[derive(Debug, Deserialize)]
pub struct PositionError {
	#[serde(with = "serde_pos_err_code")]
	pub code: PositionErrorCode,
	pub message: String,
}

#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum PositionErrorCode {
	PermissionDenied = 1,
	PositionUnavailable = 2,
	Timeout = 3,
}

#[derive(Default)]
pub struct GeolocationService {}

impl GeolocationService {
	pub fn new() -> Self { Self::default() }
	pub fn get_current_position(&self, success_cb: Callback<Position>, error_cb: Option<Callback<PositionError>>, options: Option<PositionOptions>) {
		let success_cb = move |arg: Value| success_cb.emit(arg.try_into().unwrap());
		let error_cb = move |arg: Value| if let Some(ref error_cb) = error_cb {
			error_cb.emit(arg.try_into().unwrap());
		};
		js! { @(no_return)
			var success_cb = @{success_cb};
			var error_cb = @{error_cb};
			var options = @{options.unwrap_or_default()};
			var success_action = function(arg) {
				success_cb(cloneAsObject(arg));
				success_cb.drop();
				error_cb.drop();
			};
			var error_action = function(arg) {
				error_cb(cloneAsObject(arg));
				success_cb.drop();
				error_cb.drop();
			};
			navigator.geolocation.getCurrentPosition(success_action, error_action, options);
		}
	}
	pub fn watch_position(&mut self, success_cb: Callback<Position>, error_cb: Option<Callback<PositionError>>, options: Option<PositionOptions>) -> WatchPositionTask {
		let success_cb = move |arg: Value| success_cb.emit(arg.try_into().unwrap());
		let error_cb = move |arg: Value| if let Some(ref error_cb) = error_cb {
			error_cb.emit(arg.try_into().unwrap());
		};
		let handle = js! {
			var success_cb = @{success_cb};
			var error_cb = @{error_cb};
			var options = @{options.unwrap_or_default()};
			var success_action = function(arg) {
				success_cb(cloneAsObject(arg));
			};
			var error_action = function(arg) {
				error_cb(cloneAsObject(arg));
				// TODO: find out if watchPosition will keep trying. if not, drop both callbacks (and then don't drop in cancel())
				/*success_cb.drop(); // assuming watchPosition won't keep trying
				error_cb.drop();*/
			};
			return {
				watch_id: navigator.geolocation.watchPosition(success_action, error_action, options),
				success_cb: success_cb,
				error_cb: error_cb,
			};
		};
		WatchPositionTask(Some(handle))
	}
}

pub struct WatchPositionTask(Option<Value>);

impl Task for WatchPositionTask {
	fn is_active(&self) -> bool {
		self.0.is_some()
	}
	fn cancel(&mut self) {
		let handle = self.0.take().expect("tried to cancel WatchPositionTask twice");
		js! { @(no_return)
			var handle = @{handle};
			navigator.geolocation.clearWatch(handle.watch_id);
			handle.success_cb.drop();
			handle.error_cb.drop();
		}
	}
}

impl Drop for WatchPositionTask {
	fn drop(&mut self) {
		if self.is_active() {
			self.cancel();
		}
	}
}
