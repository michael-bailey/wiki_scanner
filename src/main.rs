mod context;

use std::{str::FromStr, time::Duration};

use context::Context;

use async_recursion::async_recursion;
use futures::{stream::FuturesUnordered, StreamExt};
use soup::prelude::*;
use tokio::time::sleep;
use url::{ParseError::RelativeUrlWithoutBase, Url};

fn main() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(async {
			let ctx = Context::new();
			let url =
				Url::from_str("https://en.wikipedia.org/wiki/Cocaine").unwrap();
			let title = "Nun".into();

			let route = scan_page(&ctx, url, title, 0).await;

			println!("found route:");
			for url in route.iter().rev().enumerate() {
				println!("{}: {}", url.0, url.1);
			}
		});
}

#[async_recursion]
async fn scan_page(
	ctx: &Context,
	url: Url,
	page_title: String,
	distance: usize,
) -> Vec<Url> {
	println!("new scan at: {}", url);
	// println!("for title: {}", page_title);
	println!("with distance: {}", distance);
	ctx.ring_start(distance).await;

	while !ctx.can_run(distance).await {
		println!("[{}] is waiting to scan", url);
		sleep(Duration::from_millis(100)).await;
	}

	println!("[{}] can now scan", url);

	let req = reqwest::get(url.clone()).await.unwrap();

	println!("[{}] got response", url);

	let html_string = req.text().await.unwrap();

	let scan_result = {
		let html = Soup::new(&html_string);

		println!("[{}] parsed html", url);

		let title = html
			.recursive(true)
			.tag("span")
			.find_all()
			.map(|el| el.children().next().map(|n| n.text()))
			.collect::<Vec<_>>()[0]
			.clone();

		println!("got title: {:?}", title);

		let title_matches = title.clone().map_or(false, |v| v == page_title);

		println!("[{}] title matches? {}", url, title_matches);

		if title_matches {
			Ok(title.unwrap())
		} else {
			let nodes = html
				.recursive(true)
				.tag("a")
				.attr_name("href")
				.find_all()
				.filter_map(|el| {
					let attrs = el.attrs();
					let href = attrs["href"].as_str();

					let mut url_res = Url::parse(href);

					if let Err(RelativeUrlWithoutBase) = url_res {
						let base_url = Url::parse("https://en.wikipedia.org/")
							.expect("base url is wrong");
						url_res = base_url.join(href);
					}

					if let Ok(url) = url_res {
						let valid =
							url.host().is_some()
								&& url
									.host()
									.expect("This should no longer execute")
									.to_string() == "en.wikipedia.org"
								&& url.query().is_none() && url
								.fragment()
								.is_none() && !url.path().contains('#')
								&& !url.path().contains(':') && !url
								.path()
								.contains('.') && !url.path().contains("Main_Page");
						// println!("checking url validity: {} {}", url, valid);
						if valid {
							Some(url)
						} else {
							None
						}
					} else {
						None
					}
				})
				.map(|url| {
					scan_page(ctx, url, page_title.clone(), distance + 1)
				})
				.collect::<FuturesUnordered<_>>();

			println!("valid paths: {}", nodes.len());
			Err(nodes)
		}
	};

	println!("[{}] ring finished", url);
	ctx.ring_finish(distance).await;

	match scan_result {
		Ok(_title) => {
			vec![url]
		}
		Err(mut scan_result) => {
			let mut path = scan_result
				.next()
				.await
				.expect("somehow came to the end of iterator?");
			path.append(&mut vec![url]);
			path
		}
	}
}
