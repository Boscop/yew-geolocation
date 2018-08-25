#![recursion_limit = "128"]

use std::convert::From;
use yew::{prelude::*, services::Task};
use stdweb::{*, unstable::TryInto};
use serde_derive::*;
use smart_default::*;

// https://w3c.github.io/geolocation-api/#idl-index

pub type DOMString = String;

pub type DOMTimeStamp = u64;

#[derive(Deserialize, Debug)]
pub struct InternalPosition {
	pub coords: Coordinates,
	pub timestamp: f64, // TODO: should be u64 but: ConversionError { kind: Custom("invalid type: floating point 1535164942454, expected u64") }
}

#[derive(Debug)]
pub struct Position {
	pub coords: Coordinates,
	pub timestamp: DOMTimeStamp,
}

#[derive(Deserialize, Debug)]
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

#[derive(Serialize, Debug, SmartDefault)]
pub struct PositionOptions {
	#[serde(rename = "enableHighAccuracy")]
	pub enable_high_accuracy: bool,
	#[default = "0xFFFFFFFF"]
	#[serde(rename = "timeout")]
	pub timeout_ms: u32,
	#[serde(rename = "maximumAge")]
	pub maximum_age: u32,
}

#[derive(Deserialize, Debug)]
pub struct InternalPositionError {
	pub code: u16,
	pub message: DOMString,
}

#[derive(Debug)]
pub struct PositionError {
	pub code: PositionErrorCode,
	pub message: DOMString,
}

#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum PositionErrorCode {
	PermissionDenied = 1,
	PositionUnavailable = 2,
	Timeout = 3,
}

impl From<InternalPositionError> for PositionError {
	fn from(x: InternalPositionError) -> Self {
		use self::PositionErrorCode::*;
		Self {
			code: match x.code {
				1 => PermissionDenied,
				2 => PositionUnavailable,
				3 => Timeout,
				_ => unreachable!()
			},
			message: x.message,
		}
	}
}

impl From<InternalPosition> for Position {
	fn from(x: InternalPosition) -> Self {
		Self {
			coords: x.coords,
			timestamp: x.timestamp as _, // TODO: ensure it won't truncate
		}
	}
}

js_deserializable!(InternalPosition);
js_deserializable!(Coordinates);
js_serializable!(PositionOptions);
js_deserializable!(InternalPositionError);

#[derive(Default)]
pub struct GeolocationService {}

impl GeolocationService {
	pub fn new() -> Self { Self::default() }
	pub fn get_current_position(&self, success_cb: Callback<Position>, error_cb: Option<Callback<PositionError>>, options: Option<PositionOptions>) {
		let success_cb = move |arg: Value| {
			let r: InternalPosition = arg.try_into().unwrap();
			success_cb.emit(r.into());
		};
		let error_cb = move |arg: Value| if let Some(ref error_cb) = error_cb {
			let r: InternalPositionError = arg.try_into().unwrap();
			error_cb.emit(r.into());
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
		let success_cb = move |arg: Value| {
			let r: InternalPosition = arg.try_into().unwrap();
			success_cb.emit(r.into());
		};
		let error_cb = move |arg: Value| if let Some(ref error_cb) = error_cb {
			let r: InternalPositionError = arg.try_into().unwrap();
			error_cb.emit(r.into());
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
