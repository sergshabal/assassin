use assassin::tick::Tick;

pub trait DataFeed {
	fn next_tick(&mut self) -> Option<Tick>;
}

pub trait Model {
	fn get_name(&self) -> &'static str;
	fn process_tick(&mut self, Tick);
}