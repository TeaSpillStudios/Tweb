use log::info;
use markdown::file_to_html;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct MarkdownLoader {
    cache: HashMap<String, String>,
    root_path: String,
}

impl MarkdownLoader {
    pub fn load_page(&mut self, page_name: &str) -> String {
        if self.root_path.is_empty() {
            return self.cache.get(page_name).unwrap().to_owned();
        }

        if self.cache.contains_key(page_name) && !crate::LIVE_MODE {
            info!("Serving from cache.");

            self.cache
                .get(page_name)
                .unwrap()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        } else {
            if crate::LIVE_MODE {
                info!("Live mode is on. Regenerating HTML");
            } else {
                info!("Regenerating HTML");
            }

            if page_name == "" {
                self.cache.insert(
                    String::from(page_name),
                    file_to_html(Path::new(&self.root_path))
                        .expect("Failed to load the Markdown file!"),
                );
            } else {
                let page_file_name = match page_name.ends_with(".md") {
                    true => page_name.to_string(),
                    false => page_name.to_string() + ".md",
                };
                self.cache.insert(
                    page_name.to_string(),
                    file_to_html(Path::new(&page_file_name)).expect(&format!(
                        "Failed to load the specified Markdown file `{page_name}`!"
                    )),
                );
            }

            self.cache
                .get(page_name)
                .unwrap()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        }
    }

    pub fn validate_page(&self, page_name: &str) -> bool {
        let page_file_name = match page_name.ends_with(".md") {
            true => page_name.to_string(),
            false => page_name.to_string() + ".md",
        };

        match page_name == "" {
            true => true,
            false => Path::new(&page_file_name).is_file(),
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.root_path = path;
    }

    pub fn get_page_name(&mut self, page_name: &str) -> String {
        let mut page_file_name = String::new();

        if page_name != "" {
            page_file_name = match page_name.ends_with(".md") {
                true => page_name.to_string(),
                false => page_name.to_string() + ".md",
            };
        } else {
            page_file_name = self.root_path.clone();
        }

        let markdown_title = fs::read_to_string(page_file_name).unwrap();

        markdown_title
            .lines()
            .next()
            .unwrap_or("Default title")
            .split_once(' ')
            .unwrap()
            .1
            .to_string()
    }
}
