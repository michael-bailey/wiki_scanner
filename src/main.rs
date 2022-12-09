mod context;
mod html_checker;

use std::{pin::Pin, str::FromStr, sync::Arc, time::Duration};

use context::Context;

use async_recursion::async_recursion;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use soup::prelude::*;
use tokio::time::sleep;
use url::{ParseError::RelativeUrlWithoutBase, Url};

use crate::context::ContextState;

fn main() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(async {
			let url =
				Url::from_str("https://en.wikipedia.org/wiki/Cocaine").unwrap();
			let title = "Nun".into();

			let ctx = Context::new(title, url);
			ctx.start();

			loop {
				if let ContextState::Finished(path) = ctx.get_status() {}
			}

			println!("found route:");
			for url in route.unwrap().iter().rev().enumerate() {
				println!("{}: {}", url.0, url.1);
			}
		});
}

#[derive(Debug)]
enum ScanError {
	UrlExistsInContext,
}

async fn scan_page(
	ctx: Arc<Context>,
	url: Url,
	page_title: String,
	distance: usize,
) {
	ctx.ring_start(distance).await;

	while !ctx.can_run(distance).await {
		println!("[{}] is waiting to scan", url);
		sleep(Duration::from_millis(100)).await;
	}

	if ctx.cache_has_url(&url).await {
		return;
	}

	// clean up all computation artefacts when done
	let scan_result = {
		let html_string = get_html(&url).await;
		test_page(ctx.clone(), html_string, page_title, distance)
	};

	ctx.ring_finish(distance).await;

	match scan_result {
		Ok(_title) => Ok(vec![url]),
		Err(mut scan_result) => {
			loop {
				if let Err(err) = next_page_result {
					break;
				}

				let Ok(vec) = next_page_result;
			}

			let Ok(mut path) = path else {
				return path;
			};

			path.append(&mut vec![url]);

			Ok(path)
		}
	}
}

async fn get_html(url: &Url) -> String {
	loop {
		let req_res = reqwest::get(url.clone()).await;

		let Ok(res) = req_res else {
			println!("[{}] got error when requesting page", url);
			continue;
		};

		if let Ok(html_string) = res.text().await {
			break html_string;
		};

		println!("[{}] got error when requesting page", url);
		sleep(Duration::from_millis(100)).await;
	}
}

fn check_title() -> bool {
	let title = html
		.recursive(true)
		.tag("span")
		.find_all()
		.map(|el| el.children().next().map(|n| n.text()))
		.collect::<Vec<_>>()[0]
		.clone();

	// println!("got title: {:?}", title);

	let title_matches = title.clone().map_or(false, |v| v == page_title);
}
