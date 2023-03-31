use crate::markdown_loader::MarkdownLoader;

fn format_html(page_name: &str, css: String, html: String) -> String {
    format!(
        "
<!DOCTYPE html>
<head>
    <title>{}</title>
    <meta charset='utf-8'>

    {}
</head>

<body>
{}
</body>",
        page_name, css, html
    )
}

pub fn compose_html(page_name: &str, markdown_loader: &mut MarkdownLoader) -> String {
    let page_exists = markdown_loader.validate_page(page_name);

    let status = match page_exists {
        true => "HTTP/1.1 200 OK",
        false => "HTTP/1.1 404 PAGE_NOT_FOUND",
    };

    let data = match page_exists {
        true => format_html(
            &markdown_loader.get_page_name(page_name),
            String::from(crate::CSS),
            markdown_loader.load_page(page_name),
        ),

        false => format_html(
            "404",
            String::from(crate::CSS),
            String::from("<h1>Error: 404</h1><p>Page not found.</p>"),
        ),
    };

    format!("{status}\r\nContent-Length: {}\r\n\r\n{data}", data.len())
}
