use yew::{*, prelude::*, services::ConsoleService};
use yew_geolocation::*;
use derive_new::*;

fn main() {
	yew::initialize();
	App::<Model>::new().mount_to_body();
	yew::run_loop();
}

#[derive(Debug)]
enum Msg {
	GetPosition,
	StartWatchingPosition,
	StopWatchingPosition,
	CurPosition(Position),
	PositionChanged(Position),
	PositionError(PositionError),
}

#[derive(new)]
struct Model {
	link: ComponentLink<Model>,
	#[new(default)]
	geolocation_service: GeolocationService,
	#[new(value = "ConsoleService::new()")]
	console: ConsoleService,
	#[new(default)]
	watch_task: Option<WatchPositionTask>,
	#[new(default)]
	pos_history: Vec<Position>,
	#[new(default)]
	cur_pos: Option<Position>,
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_props: (), link: ComponentLink<Self>) -> Self {
		Self::new(link)
	}
	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		self.console.log(&format!("msg: {:?}", msg));
		let error_cb = self.link.send_back(Msg::PositionError);
		let options = PositionOptions { enable_high_accuracy: true, timeout_ms: 10000, ..Default::default() };
		use self::Msg::*;
		match msg {
			GetPosition => self.geolocation_service.get_current_position(self.link.send_back(Msg::CurPosition), Some(error_cb), Some(options)),
			StartWatchingPosition => self.watch_task = Some(self.geolocation_service.watch_position(self.link.send_back(Msg::PositionChanged), Some(error_cb), Some(options))),
			StopWatchingPosition => self.watch_task = None,
			CurPosition(p) => self.cur_pos = Some(p),
			PositionChanged(p) => self.pos_history.push(p),
			PositionError(e) => self.console.error(&e.message),
		}
		true
	}
}

impl Renderable<Model> for Model {
	fn view(&self) -> Html<Self> {
		let render_pos = |p: &Position| html! {
			<p>{ p.coords.latitude } {", "} { p.coords.longitude }</p>
		};
		html! {
			<>
				<button onclick=|_| Msg::GetPosition,>{ "call getCurrentPosition()" }</button>
				<p>{ "Result from getCurrentPosition:" }</p>
				{ self.cur_pos.as_ref().map_or_else(|| html! { <></> }, render_pos) }
				<hr/>
				<button onclick=|_| Msg::StartWatchingPosition, disabled=self.watch_task.is_some(),>{ "Start Watching Position" }</button>
				<button onclick=|_| Msg::StopWatchingPosition, disabled=self.watch_task.is_none(),>{ "Stop Watching Position" }</button>
				<p>{ "Position history from watching:" }</p>
				{ for self.pos_history.iter().map(render_pos) }
				<hr/>
			</>
		}
	}
}