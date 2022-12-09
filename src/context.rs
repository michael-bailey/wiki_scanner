use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use url::Url;

use crate::scan_page;

pub enum ContextState {
	NotStarted,
	Running,
	Finished(Vec<Url>),
}

#[allow(dead_code)]
pub struct Context {
	start_url: Url,
	page_title: String,
	ring_creation: Mutex<HashMap<usize, usize>>,
	ring_completion: Mutex<HashMap<usize, usize>>,
	page_cache: Mutex<HashMap<Url, Vec<Url>>>,
	mapped_paths: Mutex<HashMap<Url, Url>>,
	state: Mutex<ContextState>,
}

/// public functions
impl Context {
	pub fn new(for_title: String, starting_with: Url) -> Arc<Self> {
		let ctx = Arc::new(Context {
			page_title: for_title,
			ring_creation: Default::default(),
			ring_completion: Default::default(),
			page_cache: Default::default(),
			mapped_paths: Default::default(),
			start_url: starting_with,
			state: Mutex::new(ContextState::NotStarted),
		});

		let sleep_task = ctx.clone();

		tokio::spawn(async move { sleep_task.save_process().await });
		ctx
	}

	pub fn start() {}

	pub async fn get_result() -> Vec<Url> {}
}

/// protected methods
impl Context {
	pub(super::) async fn found_page(&self, at_url: Url) {
		let lock = self.mapped_paths.lock().await;

		let mut current_link = at_url.clone();
		let mut link_chain = vec![current_link.clone()];

		while let Some(next) = lock.get(&current_link) {
			link_chain.push(next.clone());
			current_link = next.clone();

			if *next == self.start_url {
				println!("got ");
			}
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

	pub async fn can_run(&self, distance: usize) -> bool {
		if distance == 0 {
			return true;
		}

		let creation_lock = self.ring_creation.lock().await;
		let completion_lock = self.ring_completion.lock().await;

		creation_lock[&(distance - 1)] == completion_lock[&(distance - 1)]
	}
}

/// private methods
impl Context {
	pub async fn start_node(
		self: &Arc<Self>,
		current_url: Url,
		next_url: Url,
		page_title: String,
		distance: usize,
	) {
		// create entry for backtracking
		let mut lock = self.mapped_paths.lock().await;
		lock.insert(next_url.clone(), current_url);

		tokio::spawn(scan_page(self.clone(), next_url, page_title, distance));
	}
}

/// cache methods
impl Context {
	#[allow(dead_code)]
	pub async fn cache_has_url(&self, base: &Url) -> bool {
		self.page_cache.lock().await.contains_key(base)
	}

	#[allow(dead_code)]
	pub async fn cache_insert_urls(&self, base: Url, urls: Vec<Url>) {
		let mut lock = self.page_cache.lock().await;
		lock.insert(base, urls);
	}

	#[allow(dead_code)]
	pub async fn cache_find_urls(&self, base: &Url) -> Option<Vec<Url>> {
		let lock = self.page_cache.lock().await;
		lock.get(base).cloned()
	}

	async fn save_process(&self) {
		loop {
			sleep(Duration::from_secs(10)).await;
			let lock = &*self.page_cache.lock().await;
			let map = lock
				.iter()
				.map(|(k, v)| {
					(
						k.to_string(),
						v.iter().map(|u| u.to_string()).collect::<Vec<_>>(),
					)
				})
				.collect::<HashMap<_, _>>();

			let url_cache = serde_json::to_string_pretty::<
				HashMap<String, Vec<String>>,
			>(&map);

			// if matches, execute block
			// if doesn't match, don't execute block
			if let Err(err) = &url_cache {
				err.is_data();
			}

			let Ok(url_cache) = url_cache else {
				println!("could not serialise cache");
				return;
			};

			let write_res = tokio::fs::write("./cache.json", url_cache).await;

			if write_res.is_err() {
				println!(r#"got error writing to file"#);
			}
		}
	}
}
