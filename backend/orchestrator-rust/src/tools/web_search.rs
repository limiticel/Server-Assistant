use std::process::Command;

use serde_json::{json, Value};

use crate::registry::{object_schema, Registry, Tool, ToolResult};

pub fn register(registry: &mut Registry) {
    registry.register(Tool {
        name: "web_search".to_string(),
        description: "Busca informacoes na internet usando DuckDuckGo Instant Answer.".to_string(),
        roles: vec!["admin".to_string(), "dev".to_string()],
        input_schema: object_schema(
            json!({
                "query": {
                    "type": "string",
                    "description": "Termo ou pergunta para buscar na internet."
                },
                "max_results": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 8,
                    "default": 5
                }
            }),
            vec!["query"],
        ),
        handler: web_search,
    });
}

fn web_search(args: &Value, _registry: &Registry) -> ToolResult {
    let query = required_string(args, "query")?;
    let search_query = normalize_query(query);
    let max_results = args
        .get("max_results")
        .and_then(Value::as_u64)
        .unwrap_or(5)
        .clamp(1, 8) as usize;

    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        percent_encode(&search_query)
    );
    let output = Command::new(curl_command())
        .arg("-L")
        .arg("-s")
        .arg("--max-time")
        .arg("20")
        .arg(&url)
        .output()
        .map_err(|err| format!("Failed to execute curl: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "Search request failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("Invalid search response JSON: {err}"))?;
    let mut results = Vec::new();

    if let Some(items) = response.get("Results").and_then(Value::as_array) {
        collect_results(items, &mut results, max_results);
    }
    if results.len() < max_results {
        if let Some(items) = response.get("RelatedTopics").and_then(Value::as_array) {
            collect_results(items, &mut results, max_results);
        }
    }

    if results.is_empty()
        && response
            .get("AbstractText")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .is_empty()
    {
        results = bing_rss_search(&search_query, max_results)?;
    }

    Ok(json!({
        "query": query,
        "search_query": search_query,
        "source": if results.is_empty() { "duckduckgo_instant_answer" } else { "web_search" },
        "heading": response.get("Heading").and_then(Value::as_str).unwrap_or_default(),
        "abstract": response
            .get("AbstractText")
            .and_then(Value::as_str)
            .or_else(|| response.get("Answer").and_then(Value::as_str))
            .unwrap_or_default(),
        "abstract_url": response.get("AbstractURL").and_then(Value::as_str).unwrap_or_default(),
        "official_website": response.get("OfficialWebsite").and_then(Value::as_str).unwrap_or_default(),
        "results": results
    }))
}

fn normalize_query(query: &str) -> String {
    let lower = query.to_lowercase();

    for prefix in ["site oficial da ", "site oficial do ", "site oficial de "] {
        if let Some(index) = lower.find(prefix) {
            let entity = query[index + prefix.len()..].trim();
            if !entity.is_empty() {
                return format!("{entity} official website");
            }
        }
    }

    query.to_owned()
}

fn bing_rss_search(query: &str, max_results: usize) -> Result<Vec<Value>, String> {
    let url = format!(
        "https://www.bing.com/search?q={}&format=rss",
        percent_encode(query)
    );
    let output = Command::new(curl_command())
        .arg("-L")
        .arg("-s")
        .arg("--max-time")
        .arg("20")
        .arg(&url)
        .output()
        .map_err(|err| format!("Failed to execute curl for RSS fallback: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "RSS fallback failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let xml = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for item in xml.split("<item>").skip(1) {
        if results.len() >= max_results {
            break;
        }

        let title = extract_xml_field(item, "title");
        let url = extract_xml_field(item, "link");
        let snippet = extract_xml_field(item, "description");
        let published_at = extract_xml_field(item, "pubDate");

        if title.is_empty() || url.is_empty() {
            continue;
        }

        results.push(json!({
            "title": html_decode(&title),
            "url": html_decode(&url),
            "snippet": html_decode(&snippet),
            "published_at": html_decode(&published_at)
        }));
    }

    Ok(results)
}

fn extract_xml_field(item: &str, field: &str) -> String {
    let start_tag = format!("<{field}>");
    let end_tag = format!("</{field}>");
    let Some(start) = item.find(&start_tag) else {
        return String::new();
    };
    let content_start = start + start_tag.len();
    let Some(relative_end) = item[content_start..].find(&end_tag) else {
        return String::new();
    };
    strip_html(item[content_start..content_start + relative_end].trim())
}

fn html_decode(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn collect_results(items: &[Value], results: &mut Vec<Value>, max_results: usize) {
    for item in items {
        if results.len() >= max_results {
            return;
        }

        if let Some(nested) = item.get("Topics").and_then(Value::as_array) {
            collect_results(nested, results, max_results);
            continue;
        }

        let title = item
            .get("Text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        let url = item
            .get("FirstURL")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();

        if title.is_empty() || url.is_empty() {
            continue;
        }

        results.push(json!({
            "title": strip_html(title),
            "url": url
        }));
    }
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Invalid or missing parameter: {key}"))
}

fn percent_encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn strip_html(input: &str) -> String {
    let mut output = String::new();
    let mut inside_tag = false;

    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }

    output
}

fn curl_command() -> &'static str {
    if cfg!(windows) {
        "curl.exe"
    } else {
        "curl"
    }
}
