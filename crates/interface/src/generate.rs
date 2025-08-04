// use std::{
//     collections::{BTreeSet, HashMap},
//     io::{Cursor, Read as _},
// };

// use reqwest::blocking::Client;
// use scraper::{ElementRef, Html, Selector};
// use zip::ZipArchive;

// pub fn generate_items() -> Vec<Iso639> {
//     let client = Client::new();
//     let url = "https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes";
//     let html = client.get(url).send().unwrap().text().unwrap();
//     let html = Html::parse_document(&html);
//     let table_select = Selector::parse("table.wikitable").unwrap();
//     let table = html.select(&table_select).next().unwrap();
//     let mut table = parse_html_table_to_map(&table.html())
//         .into_iter()
//         .filter(|v| v.keys().len() != 2)
//         .map(Iso639::try_from)
//         .collect::<Result<Vec<Iso639>, _>>()
//         .unwrap();

//     let url = "https://en.wikipedia.org/wiki/ISO_639_macrolanguage#List_of_macrolanguages";
//     let html = client.get(url).send().unwrap().text().unwrap();
//     let html = Html::parse_document(&html);
//     let table2 = html.select(&table_select).next().unwrap();
//     let table2 = parse_html_table_to_map(&table2.html())
//         .into_iter()
//         .map(Iso639::try_from)
//         .collect::<Result<Vec<Iso639>, _>>()
//         .unwrap()
//         .into_iter()
//         .filter(|v| v.name != "total codes" && v.name != "Name of macrolanguage");
//     for item in table2 {
//         match table.iter_mut().find(|v| v.overlap(&item)) {
//             Some(s) => s.merge(item),
//             None => table.push(item),
//         }
//     }
//     let mut lang = table
//         .into_iter()
//         .map(|v| (v.set3.iter().next().unwrap().clone(), v))
//         .collect::<HashMap<String, _>>();
//     let extend = parse_headings_and_lists(&html);
//     for (heading, items) in extend {
//         lang.get_mut(&heading).unwrap().set3.extend(items);
//     }
//     let mut table = lang.into_values().collect::<Vec<_>>();

//     for (code, lang) in zip().unwrap() {
//         if table.iter().find(|v| v.set3.contains(&code)).is_none() {
//             table.push(Iso639 {
//                 name: lang,
//                 native_name: None,
//                 set1: None,
//                 set2: Default::default(),
//                 set3: vec![code].into_iter().collect(),
//             })
//         }
//     }

//     table
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct Iso639 {
//     pub name: String,
//     pub native_name: Option<String>,
//     pub set1: Option<String>,
//     pub set2: BTreeSet<String>,
//     pub set3: BTreeSet<String>,
// }

// impl Iso639 {
//     pub fn overlap(&self, other: &Self) -> bool {
//         self.set1.is_some() && self.set1 == other.set1
//             || self.set2.iter().find(|v| other.set2.contains(*v)).is_some()
//             || self.set3.iter().find(|v| other.set3.contains(*v)).is_some()
//     }

//     pub fn merge(&mut self, other: Self) {
//         self.set1 = self.set1.clone().or(other.set1);

//         self.set2.extend(other.set2);
//         self.set3.extend(other.set3);
//     }
// }

// impl TryFrom<HashMap<String, String>> for Iso639 {
//     type Error = ();

//     fn try_from(mut value: HashMap<String, String>) -> Result<Self, Self::Error> {
//         if let Some(name) = value.remove("ISO Language Names") {
//             let set1 = value.remove("Set 1").ok_or(())?;
//             let set2 = value.remove("Set 2").ok_or(())?;
//             let set3 = value.remove("Set 3").ok_or(())?;
//             let native_name = value.remove("Endonym(s)");
//             Ok(Self {
//                 name,
//                 native_name,
//                 set1: Some(set1),
//                 set2: set2.split(',').map(|s| s.trim().to_string()).collect(),
//                 set3: vec![
//                     set3.split_once("\u{a0}+")
//                         .map(|v| v.0.to_owned())
//                         .unwrap_or(set3),
//                 ]
//                 .into_iter()
//                 .collect(),
//             })
//         } else if let Some(name) = value.remove("Name of macrolanguage") {
//             let set1 = value.remove("ISO 639-1").ok_or(())?;
//             let set1 = match set1.starts_with("(-)") {
//                 true => None,
//                 false => Some(set1),
//             };
//             let set2 = value.remove("ISO 639-2").ok_or(())?;
//             let set2 = match set2.starts_with("(-)") {
//                 true => vec![],
//                 false => set2
//                     .split("/")
//                     .map(|v| v.trim().to_owned())
//                     .collect::<Vec<_>>(),
//             };
//             let set3 = value.remove("ISO 639-3").ok_or(())?;
//             let native_name = value.remove("Endonym(s)");
//             Ok(Self {
//                 name,
//                 native_name,
//                 set1,
//                 set2: set2.into_iter().collect(),
//                 set3: vec![set3].into_iter().collect(),
//             })
//         } else {
//             unreachable!()
//         }
//     }
// }

// fn parse_headings_and_lists(html: &Html) -> Vec<(String, Vec<String>)> {
//     let container_selector = Selector::parse("#mw-content-text > div").unwrap();

//     let mut result = Vec::new();

//     if let Some(container) = html.select(&container_selector).next() {
//         let mut children = container.children().filter_map(ElementRef::wrap).peekable();

//         while let Some(el) = children.next() {
//             if el
//                 .value()
//                 .has_class("mw-heading", scraper::CaseSensitivity::AsciiCaseInsensitive)
//             {
//                 if let Some(v) = el.select(&Selector::parse("h4").unwrap()).next() {
//                     let heading_text = v.text().collect::<Vec<_>>().join(" ").trim().to_string();
//                     while let Some(v) = children.peek() {
//                         if v.value().name() == "ul" {
//                             break;
//                         }

//                         if v.value()
//                             .has_class("mw-heading", scraper::CaseSensitivity::AsciiCaseInsensitive)
//                         {
//                             break;
//                         }
//                         children.next();
//                     }
//                     if let Some(next_el) = children.peek() {
//                         if next_el.value().name() == "ul" {
//                             let ul_el = children.next().unwrap(); // consume it
//                             let items = ul_el
//                                 .select(&Selector::parse("li").unwrap())
//                                 .map(|li| li.text().next().unwrap().to_owned())
//                                 .collect::<Vec<_>>();

//                             result.push((heading_text, items));
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     result
// }

// fn parse_html_table_to_map(html_table: &str) -> Vec<HashMap<String, String>> {
//     let document = Html::parse_fragment(html_table);
//     let row_selector = Selector::parse("tr").unwrap();
//     let cell_selector = Selector::parse("th, td").unwrap();

//     let mut rows = document.select(&row_selector);

//     fn extract_cells_with_colspan(
//         row: scraper::element_ref::ElementRef,
//         cell_selector: &Selector,
//     ) -> Vec<String> {
//         let mut result = Vec::new();
//         for cell in row.select(cell_selector) {
//             let text = cell.text().collect::<String>().trim().to_string();
//             let colspan = cell
//                 .value()
//                 .attr("colspan")
//                 .and_then(|v| v.parse::<usize>().ok())
//                 .unwrap_or(1);
//             for _ in 0..colspan {
//                 result.push(text.clone());
//             }
//         }
//         result
//     }

//     let headers: Vec<String> = if let Some(header_row) = rows.next() {
//         extract_cells_with_colspan(header_row, &cell_selector)
//     } else {
//         return Vec::new();
//     };

//     rows.map(|row| {
//         let cells = extract_cells_with_colspan(row, &cell_selector);

//         let mut map = HashMap::new();
//         for (header, cell) in headers.iter().zip(cells.iter()) {
//             map.entry(header.clone())
//                 .and_modify(|existing: &mut String| existing.push_str(&format!(",{}", cell)))
//                 .or_insert_with(|| cell.clone());
//         }
//         map
//     })
//     .collect()
// }

// fn zip() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
//     let url = "https://iso639-3.sil.org/sites/iso639-3/files/downloads/iso-639-3_Code_Tables_20250415.zip";

//     let response = reqwest::blocking::get(url)?;
//     let bytes = response.bytes()?;

//     let reader = Cursor::new(bytes);

//     let mut zip = ZipArchive::new(reader)?;

//     let mut target_index = None;
//     let target_filename = "iso-639-3_Name_Index.tab";
//     for i in 0..zip.len() {
//         let file = zip.by_index(i)?;
//         if file.name().ends_with(target_filename) {
//             target_index = Some(i);
//             break;
//         }
//     }
//     let mut file = zip.by_index(target_index.unwrap())?;

//     let mut contents = String::new();
//     file.read_to_string(&mut contents)?;

//     Ok(contents
//         .lines()
//         .skip(1)
//         .map(|line| {
//             let mut items = line.split("	");
//             (
//                 items.next().unwrap().to_owned(),
//                 items.next().unwrap().to_owned(),
//             )
//         })
//         .collect::<Vec<_>>())
// }

// #[cfg(test)]
// mod tests {
//     use crate::generate::generate_items;

//     #[test]
//     fn generate() {
//         let items = generate_items();
//         println!("{:#?}", items);
//     }
// }
