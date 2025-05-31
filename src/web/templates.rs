use axum::response::Html;
use super::routes::ConversionResult;

// Simple template rendering function
fn render_template(template_name: &str, replacements: &[(String, String)]) -> Html<String> {
    // Get template content based on template name
    let content = match template_name {
        "conversion_result.html" => include_str!("templates/conversion_result.html"),
        "currencies_list.html" => include_str!("templates/currencies_list.html"),
        "error.html" => include_str!("templates/error.html"),
        _ => return Html(format!("<p>Template not found: {}</p>", template_name)),
    };
    
    let mut content = content.to_string();
    
    for (key, value) in replacements {
        // Replace {{ key }} with value (without curly braces)
        content = content.replace(&format!("{{{{ {} }}}}", key), value);
        // Also handle the Handlebars-style {{{key}}} syntax
        content = content.replace(&format!("{{{{{{{}}}}}}}", key), value);
    }
    
    // Handle each loop for currencies
    if template_name == "currencies_list.html" && content.contains("{{#each currencies}}") {
        if let Some(currencies_value) = replacements.iter().find(|(k, _)| k == "currencies_json") {
            let currencies: Vec<String> = serde_json::from_str(&currencies_value.1).unwrap_or_default();
            let mut items_html = String::new();
            
            for currency in currencies {
                items_html.push_str(&format!(
                    "<div class=\"bg-gray-50 rounded p-2 text-sm\">\n    <span class=\"font-medium\">{}</span>\n</div>\n",
                    currency
                ));
            }
            
            content = content.replace(
                "{{#each currencies}}\n        <div class=\"bg-gray-50 rounded p-2 text-sm\">\n            <span class=\"font-medium\">{{this}}</span>\n        </div>\n        {{/each}}",
                &items_html
            );
        }
    }
    
    Html(content)
}

pub fn render_index() -> Html<String> {
    // Embed the template directly in the binary
    let content = include_str!("templates/index.html");
    Html(content.to_string())
}

pub fn render_conversion_result(result: ConversionResult) -> Html<String> {
    let replacements = vec![
        ("amount".to_string(), result.amount),
        ("from".to_string(), result.from),
        ("to".to_string(), result.to),
        ("result".to_string(), result.result),
        ("rate".to_string(), result.rate),
        ("timestamp".to_string(), result.timestamp),
    ];
    
    render_template("conversion_result.html", &replacements)
}

// Removed unused render_currencies_list function

pub fn render_error(message: String) -> Html<String> {
    let replacements = vec![
        ("message".to_string(), message),
    ];
    
    render_template("error.html", &replacements)
}
