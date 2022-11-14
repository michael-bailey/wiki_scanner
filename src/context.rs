use std::collections::HashMap;
use tokio::sync::Mutex;
use url::Url;

pub struct Context {
	ring_creation: Mutex<HashMap<usize, usize>>,
	ring_completion: Mutex<HashMap<usize, usize>>,
	page_cache: Mutex<HashMap<Url, Vec<Url>>>,
}

impl Context {}

impl Context {
	pub fn new() -> Self {
		Context {
			shortest_path: Default::default(),
			ring_creation: Default::default(),
			ring_completion: Default::default(),
			page_cache: Default::default(),
		}
	}

	pub async fn ring_start(&self, distance: usize) {
		println!("starting distance {}", distance);
		let mut creation_lock = self.ring_creation.lock().await;
		let mut completion_lock = self.ring_completion.lock().await;
		if !creation_lock.contains_key(&distance) {
			creation_lock.insert(distance, 1);
			completion_lock.insert(distance, 0);
		} else {
			let new_val = creation_lock[&distance] + 1;
			creation_lock.insert(distance, new_val);
		}
	}

	pub async fn ring_finish(&self, distance: usize) {
		println!("finishing distance {}", distance);
		let mut lock = self.ring_completion.lock().await;
		if !lock.contains_key(&distance) {
			lock.insert(distance, 1);
		} else {
			let new_val = lock[&distance] + 1;
			lock.insert(distance, new_val);
		}
	}

	pub async fn cache_has_url(&self, base: Url) -> bool {}

	pub async fn cache_insert_urls(&self, base: Url, urls: Vec<Url>) {}

	pub async fn cache_find_urls(
		&self,
		base: Url,
		urls: Vec<Url>,
	) -> Option<Vec<Url>> {
		None
	}

	pub async fn can_run(&self, distance: usize) -> bool {
		if distance == 0 {
			return true;
		}

		let creation_lock = self.ring_creation.lock().await;
		let completion_lock = self.ring_completion.lock().await;

		creation_lock[&(distance - 1)] == completion_lock[&(distance - 1)]
	}

	pub async fn set_shortest_path(
		&self,
		path: Vec<String>,
	) -> Result<(), String> {
		Ok(())
	}
}
