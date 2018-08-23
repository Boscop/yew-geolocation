#![recursion_limit = "128"]

#[macro_use] extern crate smart_default;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate stdweb_derive;

use yew::prelude::*;
use stdweb::*;
use stdweb::unstable::TryInto;

// https://w3c.github.io/geolocation-api/#idl-index

pub type DOMString = String;

pub type DOMTimeStamp = u64;

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
	pub coords: Coordinates,
	pub timestamp: DOMTimeStamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Coordinates {
	pub latitude: f64,
	pub longitude: f64,
	pub altitude: Option<f64>,
	pub accuracy: f64,
	pub altitudeAccuracy: Option<f64>,
	pub heading: Option<f64>,
	pub speed: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, SmartDefault)]
pub struct PositionOptions {
	pub enableHighAccuracy: bool,
	#[default = "0xFFFFFFFF"]
	pub timeout: u32,
	pub maximumAge: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PositionError {
	pub code: PositionErrorCode,
	pub message: DOMString,
}

#[repr(u16)]
#[derive(Serialize, Deserialize, Debug)]
pub enum PositionErrorCode {
	PERMISSION_DENIED = 1,
	POSITION_UNAVAILABLE = 2,
	TIMEOUT = 3,
}

js_serializable!(Position); js_deserializable!(Position);
js_serializable!(Coordinates); js_deserializable!(Coordinates);
js_serializable!(PositionOptions); js_deserializable!(PositionOptions);
js_serializable!(PositionError); js_deserializable!(PositionError);
js_serializable!(PositionErrorCode); js_deserializable!(PositionErrorCode);

#[derive(Clone, Debug, ReferenceType)]
#[reference(instance_of = "Object")]
pub struct Position2(Reference);

impl Position2 {
	pub fn get_timestamp( &self ) -> DOMTimeStamp {
		js! (
			return @{&self.0}.timestamp;
		).try_into().unwrap()
	}
}

pub struct GeolocationService(Value);

impl GeolocationService {
	pub fn new() -> Self {
		let geolocation = js! {
			return navigator.geolocation;
		};
		GeolocationService(geolocation)
	}
	pub fn get_current_position(&self, success_cb: Callback<Position>, error_cb: Option<Callback<PositionError>>, options: Option<PositionOptions>) {
		let success_cb = move |arg: Value| success_cb.emit(arg.try_into().unwrap());
		let error_cb = move |arg: Value| if let Some(ref error_cb) = error_cb {
			error_cb.emit(arg.try_into().unwrap());
		};
		let geolocation = &self.0;
		js! { @(no_return)
			var geolocation = @{geolocation};
			var success_cb = @{success_cb};
			var error_cb = @{error_cb};
			var options = @{options.unwrap_or_default()};
			var success_action = function(arg) {
				console.log(arg);
				success_cb(arg);
				success_cb.drop();
				error_cb.drop();
			};
			var error_action = function(arg) {
				console.error(arg);
				error_cb(arg);
				success_cb.drop();
				error_cb.drop();
			};
			geolocation.getCurrentPosition(success_action, error_action, options);
		}
	}
}
