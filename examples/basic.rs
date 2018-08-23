#![recursion_limit = "512"]
#![feature(box_syntax, nll)]

#[macro_use] extern crate yew;
#[macro_use] extern crate stdweb;
#[macro_use] extern crate smart_default;
#[macro_use] extern crate derive_new;
extern crate yew_geolocation;

use yew::prelude::*;
use yew::services::ConsoleService;
use yew_geolocation::*;

fn main() {
	yew::initialize();
	App::<Model>::new().mount_to_body();
	yew::run_loop();
}

#[derive(Debug)]
enum Msg {
	Position(Position),
	// Position(Position2),
	PositionError(PositionError),
}

#[derive(new)]
struct Model {
	link: ComponentLink<Model>,
	#[new(value = "GeolocationService::new()")]
	geolocation_service: GeolocationService,
	#[new(value = "ConsoleService::new()")]
	console: ConsoleService,
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
		let mut r = Self::new(link);
		r.change(props);
		let success_cb = r.link.send_back(|arg: Position| Msg::Position(arg));
		// let success_cb = r.link.send_back(|arg: Position2| Msg::Position(arg));
		let error_cb = r.link.send_back(|arg: PositionError| Msg::PositionError(arg));
		let options = PositionOptions { enableHighAccuracy: true, ..Default::default() };
		r.geolocation_service.get_current_position(success_cb, Some(error_cb), Some(options));
		r
	}
	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		self.console.log(&format!("msg: {:?}", msg));
		use self::Msg::*;
		match msg {
			// Position(p) => self.console.log(&format!("pos: {}", p.get_timestamp())),
			Position(p) => self.console.log(&format!("pos: {:?}", p)),
			_ => self.console.log(&format!("unhandled msg: {:?}", msg))
		}
		true
	}
	fn change(&mut self, props: Self::Properties) -> ShouldRender {
		true
	}
}

impl Renderable<Model> for Model {
	fn view(&self) -> Html<Self> {
		html! {
		}
	}
}