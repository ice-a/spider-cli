use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::core;

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Curl,
    Python,
    Java,
    Go,
    Fetch,
    Postman,
    Insomnia,
    Jmeter,
}

pub fn run(
    file: &Path,
    format: &ExportFormat,
    output: Option<&Path>,
    index: Option<usize>,
    template: bool,
    lang: &str,
) -> Result<()> {
    let content = fs::read_to_string(file)?;
    let har: serde_json::Value = serde_json::from_str(&content)?;

    let entries = har["log"]["entries"].as_array().ok_or_else(|| anyhow::anyhow!("无效的 HAR 文件"))?;

    let selected: Vec<&serde_json::Value> = match index {
        Some(i) => entries.get(i).into_iter().collect(),
        None => entries.iter().collect(),
    };

    let mut result = String::new();

    for entry in &selected {
        let req = &entry["request"];
        let method = req["method"].as_str().unwrap_or("GET");
        let url = req["url"].as_str().unwrap_or("");

        match format {
            ExportFormat::Curl => {
                result.push_str(&format!("curl -X {} '{}'", method, url));
                if let Some(headers) = req["headers"].as_array() {
                    for h in headers {
                        if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                            result.push_str(&format!(" -H '{}: {}'", name, value));
                        }
                    }
                }
                if let Some(body) = req["postData"]["text"].as_str() {
                    result.push_str(&format!(" -d '{}'", body));
                }
                result.push('\n');
            }
            ExportFormat::Python => {
                result.push_str("import requests\n\n");
                result.push_str(&format!("response = requests.{}(\n", method.to_lowercase()));
                result.push_str(&format!("    '{}',\n", url));
                result.push_str("    headers={\n");
                if let Some(headers) = req["headers"].as_array() {
                    for h in headers {
                        if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                            result.push_str(&format!("        '{}': '{}',\n", name, value));
                        }
                    }
                }
                result.push_str("    },\n");
                if let Some(body) = req["postData"]["text"].as_str() {
                    result.push_str(&format!("    data='{}',\n", body));
                }
                result.push_str(")\nprint(response.status_code)\nprint(response.text)\n");
            }
            ExportFormat::Java => {
                result.push_str("OkHttpClient client = new OkHttpClient();\n");
                result.push_str(&format!("Request request = new Request.Builder()\n    .url(\"{}\")\n", url));
                if let Some(headers) = req["headers"].as_array() {
                    for h in headers {
                        if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                            result.push_str(&format!("    .addHeader(\"{}\", \"{}\")\n", name, value));
                        }
                    }
                }
                result.push_str(&format!("    .{}()\n", method.to_lowercase()));
                result.push_str("    .build();\nResponse response = client.newCall(request).execute();\n");
            }
            ExportFormat::Go => {
                result.push_str(&format!("req, _ := http.NewRequest(\"{}\", \"{}\", nil)\n", method, url));
                if let Some(headers) = req["headers"].as_array() {
                    for h in headers {
                        if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                            result.push_str(&format!("req.Header.Set(\"{}\", \"{}\")\n", name, value));
                        }
                    }
                }
                result.push_str("resp, _ := http.DefaultClient.Do(req)\ndefer resp.Body.Close()\n");
            }
            ExportFormat::Fetch => {
                result.push_str(&format!("fetch('{}', {{\n  method: '{}',\n", url, method));
                result.push_str("  headers: {\n");
                if let Some(headers) = req["headers"].as_array() {
                    for h in headers {
                        if let (Some(name), Some(value)) = (h["name"].as_str(), h["value"].as_str()) {
                            result.push_str(&format!("    '{}': '{}',\n", name, value));
                        }
                    }
                }
                result.push_str("  },\n");
                if let Some(body) = req["postData"]["text"].as_str() {
                    result.push_str(&format!("  body: '{}',\n", body));
                }
                result.push_str("}).then(r => r.json()).then(console.log)\n");
            }
            ExportFormat::Postman => {
                let postman = serde_json::json!({
                    "info": { "name": "Exported from Reptool", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json" },
                    "item": [{
                        "name": url,
                        "request": {
                            "method": method,
                            "header": req["headers"],
                            "url": url,
                            "body": req["postData"]
                        }
                    }]
                });
                result.push_str(&serde_json::to_string_pretty(&postman)?);
            }
            _ => {
                result.push_str(&format!("[{}] {} {}\n", 0, method, url));
            }
        }
        result.push('\n');
    }

    if template {
        result.push_str(&generate_template(lang, &selected)?);
    }

    match output {
        Some(o) => {
            fs::write(o, &result)?;
            core::success(&format!("已导出到 {}", o.display()));
        }
        None => print!("{}", result),
    }

    Ok(())
}

fn generate_template(lang: &str, entries: &[&serde_json::Value]) -> Result<String> {
    let mut template = String::new();

    match lang {
        "python" => {
            template.push_str("# === Reptool 生成的完整爬虫模板 ===\n\n");
            template.push_str("import requests\nimport json\n\n");
            template.push_str("class Spider:\n");
            template.push_str("    def __init__(self):\n");
            template.push_str("        self.session = requests.Session()\n");
            template.push_str("        self.base_url = ''\n\n");
            template.push_str("    def request(self, method, url, **kwargs):\n");
            template.push_str("        resp = self.session.request(method, url, **kwargs)\n");
            template.push_str("        print(f'{method} {url} → {resp.status_code}')\n");
            template.push_str("        return resp\n\n");

            for (i, entry) in entries.iter().enumerate() {
                let req = &entry["request"];
                let method = req["method"].as_str().unwrap_or("GET");
                let url = req["url"].as_str().unwrap_or("");
                template.push_str(&format!("    def api_{}(self):\n", i));
                template.push_str(&format!("        return self.request('{}', '{}')\n\n", method, url));
            }
        }
        "rust" => {
            template.push_str("// === Reptool 生成的完整爬虫模板 ===\n\n");
            template.push_str("use reqwest;\nuse serde_json::Value;\n\n");
            template.push_str("#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
            template.push_str("    let client = reqwest::Client::builder()\n");
            template.push_str("        .danger_accept_invalid_certs(true)\n");
            template.push_str("        .build()?;\n\n");

            for (i, entry) in entries.iter().enumerate() {
                let req = &entry["request"];
                let method = req["method"].as_str().unwrap_or("GET");
                let url = req["url"].as_str().unwrap_or("");
                template.push_str(&format!("    // API {}\n", i));
                template.push_str(&format!("    let resp = client.{}(\"{}\").send().await?;\n", method.to_lowercase(), url));
                template.push_str(&format!("    println!(\"Status: {{}}\", resp.status());\n"));
                template.push_str("    let body = resp.text().await?;\n");
                template.push_str("    println!(\"{}\", body);\n\n");
            }
            template.push_str("    Ok(())\n}\n");
        }
        _ => {}
    }

    Ok(template)
}
