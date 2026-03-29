use regex::Regex;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Read, Write};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let data_dir = "./data";
    let output_file = "vocab.json";

    // ĐẶT GIỚI HẠN VOCAB Ở ĐÂY
    let max_vocab_len: usize = 16_000;

    // Bộ lọc Regex mạnh mẽ hơn
    let re = Regex::new(r"(?u)(\n|\t|[ ]|\\{1,2}[\w]|_|\d|[^\W_\d]+|[^\w\s])").unwrap();
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    // Quét file và đếm tần suất
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
    let mut current_id = 0;

    // 1. Nạp Special Tokens
    let special_tokens = vec!["[PAD]", "[UNK]", "[BOS]", "[EOS]"];
    for token in special_tokens {
        vocab.insert(token.to_string(), current_id);
        current_id += 1;
    }

    // 2. Nạp CỨNG các chữ số từ 0 đến 9 (Đảm bảo chắc chắn có)
    for digit in 0..=9 {
        let digit_str = digit.to_string();
        if !vocab.contains_key(&digit_str) {
            vocab.insert(digit_str, current_id);
            current_id += 1;
        }
    }

    // 3. Sắp xếp các token thực tế theo tần suất giảm dần
    let mut sorted_counts: Vec<(&String, &usize)> = word_counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    // 4. Bơm token vào Vocab cho đến khi chạm mức MAX_VOCAB_LEN
    for (token, _) in sorted_counts {
        // Kiểm tra điều kiện dừng
        if vocab.len() >= max_vocab_len {
            break;
        }

        // Chỉ thêm nếu chưa tồn tại (tránh trùng với special tokens hoặc 0-9)
        if !vocab.contains_key(token) {
            vocab.insert(token.clone(), current_id);
            current_id += 1;
        }
    }

    // 5. Xuất JSON
    let mut file = File::create(output_file)?;
    file.write_all(serde_json::to_string_pretty(&vocab)?.as_bytes())?;

    println!("--- Hoàn tất! ---");
    println!("Max Vocab quy định: {}", max_vocab_len);
    println!("Số lượng Token thực tế trong vocab.json: {}", vocab.len());

    Ok(())
}
