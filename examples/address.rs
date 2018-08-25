use yew::{*, prelude::*, services::{ConsoleService, fetch::{FetchService, FetchTask, Request, Response}}, format::{Json, Nothing}};
use yew_geolocation::*;
use failure::Error;
use derive_new::*;
use serde_derive::*;

fn main() {
	yew::initialize();
	App::<Model>::new().mount_to_body();
	yew::run_loop();
}

#[derive(Debug)]
enum Msg {
	CurPosition(Position),
	HandleReverseGeocodingResult(Result<ReverseGeocodingResult, Error>),
}

#[derive(Debug, Serialize, Deserialize)]
struct ReverseGeocodingResult {
	results: Vec<AddressResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddressResult {
	formatted_address: String,
}

#[derive(new)]
struct Model {
	link: ComponentLink<Model>,
	#[new(default)]
	geolocation_service: GeolocationService,
	#[new(value = "ConsoleService::new()")]
	console: ConsoleService,
	#[new(value = "FetchService::new()")]
	fetch_service: FetchService,
	#[new(default)]
	ft: Option<FetchTask>,
	#[new(default)]
	address: String,
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_props: (), link: ComponentLink<Self>) -> Self {
		let r = Self::new(link);
		let callback = r.link.send_back(Msg::CurPosition);
		let options = PositionOptions { enable_high_accuracy: true, timeout_ms: 10000, ..Default::default() };
		r.geolocation_service.get_current_position(callback, None, Some(options));
		r
	}
	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		self.console.log(&format!("msg: {:?}", msg));
		use self::Msg::*;
		match msg {
			CurPosition(p) => {
				let callback = self.link.send_back(|r: Response<Json<Result<ReverseGeocodingResult, Error>>>| {
					let (_, Json(data)) = r.into_parts();
					Msg::HandleReverseGeocodingResult(data)
				});
				let p = p.coords;
				// https://developers.google.com/maps/documentation/geocoding/start#reverse
				let url = format!("https://maps.googleapis.com/maps/api/geocode/json?sensor=true&latlng={},{}", p.latitude, p.longitude);
				let req = Request::get(&url).body(Nothing).unwrap();
				self.ft = Some(self.fetch_service.fetch(req, callback));
			}
			HandleReverseGeocodingResult(r) => {
				self.address = r.ok().and_then(|r| r.results.first().cloned()).map(|r| r.formatted_address).unwrap_or_default();
			}
		}
		true
	}
}

impl Renderable<Model> for Model {
	fn view(&self) -> Html<Self> {
		html! {
			<strong>{ "Address: " } { &self.address }</strong>
		}
	}
}