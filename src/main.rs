use regex::Regex;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Read, Write};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let data_dir = "./data";
    let output_file = "vocab.json";

    // Bộ lọc Regex mạnh mẽ hơn
    let re = Regex::new(r"(?u)(\n|\t|[ ]|\\{1,2}[\w]|\w+|[^\w\s])").unwrap();
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    for entry in WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && !path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with('.'))
        {
            let mut content = String::new();
            if let Ok(_) = File::open(path)?.read_to_string(&mut content) {
                for cap in re.captures_iter(&content) {
                    let token = cap[0].to_lowercase();
                    *word_counts.entry(token).or_insert(0) += 1;
                }
            }
        }
    }

    // Sử dụng BTreeMap để JSON xuất ra theo thứ tự ID tăng dần
    let mut vocab: BTreeMap<String, usize> = BTreeMap::new();
    let special_tokens = vec!["[PAD]", "[UNK]", "[BOS]", "[EOS]"];

    // 1. Nạp Special Tokens
    for (i, token) in special_tokens.iter().enumerate() {
        vocab.insert(token.to_string(), i);
    }

    // 2. Sắp xếp các token theo tần suất trước khi gán ID
    let mut sorted_counts: Vec<(&String, &usize)> = word_counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    let mut current_id = special_tokens.len();
    for (token, _) in sorted_counts {
        if !vocab.contains_key(token) {
            vocab.insert(token.clone(), current_id);
            current_id += 1;
        }
    }

    // 3. Xuất JSON (Đảo ngược Map để dễ nhìn ID -> Token nếu muốn, hoặc giữ Token -> ID)
    // Ở đây mình giữ Token -> ID theo đúng tiêu chuẩn
    let mut file = File::create(output_file)?;
    file.write_all(serde_json::to_string_pretty(&vocab)?.as_bytes())?;

    println!("--- Hoàn tất! Vocab đã được sắp xếp và xử lý triệt để ---");
    Ok(())
}
