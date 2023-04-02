use log::warn;

use crate::markdown_loader::MarkdownLoader;

fn format_html(page_name: &str, description: String, css: String, html: String) -> String {
    format!(
        "
<!DOCTYPE html>
<head>
    <title>{0}</title>
    <meta charset='utf-8'>
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <meta name=\"description\" content=\"{1}\">
    <meta property=\"og:title\" content=\"{0}\">
    <meta property=\"og:description\" content=\"{1}\">

    {2}
</head>

<body>
    <html lang=\"en\">
{3}
    </html>
</body>",
        page_name, description, css, html
    )
}

pub fn compose_html(page_name: &str, markdown_loader: &mut MarkdownLoader) -> String {
    let page_exists = markdown_loader.validate_page(page_name);

    let description = std::fs::read_to_string("description.txt").unwrap_or_else(|_| {
        warn!("No `description.txt` detected!");
        String::from("")
    });

    let status = match page_exists {
        true => "HTTP/1.1 200 OK",
        false => "HTTP/1.1 404 PAGE_NOT_FOUND",
    };

    let data = match page_exists {
        true => format_html(
            &markdown_loader.get_page_name(page_name),
            description.trim().to_string(),
            String::from(crate::CSS),
            markdown_loader.load_page(page_name),
        ),

        false => {
            warn!("Page \"{page_name}\" not found!");

            format_html(
                "404",
                description.trim().to_string(),
                String::from(crate::CSS),
                String::from("<h1>Error: 404</h1><p>Page not found.</p>"),
            )
        }
    };

    format!("{status}\r\nContent-Length: {}\r\n\r\n{data}", data.len())
}
