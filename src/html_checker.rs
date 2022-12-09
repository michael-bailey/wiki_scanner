use std::sync::Arc;

use reqwest::Url;
use soup::{NodeExt, QueryBuilderExt, Soup};

use crate::context::Context;

pub enum PageResult {
	TitleMatches,
	LinksFound(Vec<Url>),
}

pub fn test_page(
	ctx: Arc<Context>,
	html_string: String,
	page_title: String,
	distance: usize,
) -> PageResult {
	let html = Soup::new(&html_string);

	// println!("[{}] parsed html", url);

	let title = html
		.recursive(true)
		.tag("span")
		.find_all()
		.map(|el| el.children().next().map(|n| n.text()))
		.collect::<Vec<_>>()[0]
		.clone();

	// println!("got title: {:?}", title);

	let title_matches = title.clone().map_or(false, |v| v == page_title);

	return if title_matches {
		PageResult::TitleMatches
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
					let valid = url.host().is_some()
						&& url
							.host()
							.expect("This should no longer execute")
							.to_string() == "en.wikipedia.org"
						&& url.query().is_none() && url
						.fragment()
						.is_none() && !url.path().contains('#')
						&& !url.path().contains(':')
						&& !url.path().contains('.')
						&& !url.path().contains("Main_Page");
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
			.collect::<Vec<_>>();
		return PageResult::LinksFound(nodes);
	};
}
