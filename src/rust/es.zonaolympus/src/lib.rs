#![no_std]
use aidoku::{
	error::Result,
	helpers::substring::Substring,
	prelude::*,
	std::{defaults::defaults_get, html::Node, net::HttpMethod, net::Request, String, Vec},
	Chapter, Listing, Manga, MangaContentRating, MangaStatus, MangaViewer, Page,
};

extern crate alloc;
use alloc::{borrow::ToOwned, string::ToString};

mod parser;

static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36";
static BASE_URL: &str = "https://zonaolympus.com/";

#[link(wasm_import_module = "net")]
extern "C" {
	fn set_rate_limit(rate_limit: i32);
	fn set_rate_limit_period(period: i32);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn initialize() {
	let rate_limit: i32 = match defaults_get("rateLimit") {
		Ok(limit) => limit.as_int().unwrap_or(10) as i32,
		Err(_) => 10,
	};
	set_rate_limit(rate_limit);
	set_rate_limit_period(60);
}

#[get_manga_list]
fn get_manga_list(page: i32) -> Result<MangaPageResult> {
	let url = format!("{}library?_pg=1&page={}", BASE_URL, page);
	parser::parse_manga_list(url)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	// Aquí se gestionan las categorías de manga sin filtros adicionales
	match listing.name.as_str() {
		"Popular del día" => {
			let url = format!("{}library?order_item=likes_count&order_dir=desc&_pg=1&page={}", BASE_URL, page);
			parser::parse_manga_list(url)
		}
		"Nuevos lanzamientos" => {
			let url = format!("{}library?order_item=creation&order_dir=desc&_pg=1&page={}", BASE_URL, page);
			parser::parse_manga_list(url)
		}
		"Novelas" => {
			let url = format!("{}library?order_item=type&order_dir=desc&type=novel&_pg=1&page={}", BASE_URL, page);
			parser::parse_manga_list(url)
		}
		_ => get_manga_list(page),
	}
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = if id.starts_with("http") {
		id.clone()
	} else {
		format!("{BASE_URL}{id}")
	};
	let html = Request::new(&url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL)
		.html()?;

	let cover = html.select(".book-thumbnail").attr("src").read();
	let title = html.select("h1.element-title").first().own_text().read();
	let title_elements = html.select("h5.card-title");
	let author = title_elements
		.first()
		.attr("title")
		.read()
		.replace(", ", "");
	let artist = title_elements.last().attr("title").read().replace(", ", "");
	let description = html.select("p.element-description").text().read();

	let categories = html
		.select("a.py-2")
		.array()
		.map(|x| {
			x.as_node()
				.expect("node array element should be a node")
				.text()
				.read()
		})
		.collect::<Vec<String>>();

	let status_text = html.select("span.book-status").text().read();
	let status = match status_text.as_str() {
		"Publicándose" => MangaStatus::Ongoing,
		"Finalizado" => MangaStatus::Completed,
		_ => MangaStatus::Unknown,
	};

	let nsfw = if !html.select("i.fa-heartbeat").array().is_empty() {
		MangaContentRating::Nsfw
	} else if categories.iter().any(|x| x == "Ecchi") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	let type_text = html.select("h1.book-type").text().read();
	let viewer = match type_text.as_str() {
		"MANHWA" => MangaViewer::Scroll,
		"MANHUA" => MangaViewer::Scroll,
		_ => MangaViewer::Rtl,
	};

	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
}

fn parse_chapter(element: Node) -> Chapter {
	let url = element
		.select("div.row > .text-right > a")
		.attr("href")
		.read();
	let name = element
		.select("div.row > .text-right > a")
		.own_text()
		.read();
	Chapter {
		id: url.clone(),
		name,
		url,
		lang_code: "es".to_owned(),
		volume: None,
		episode_number: None,
	}
}
