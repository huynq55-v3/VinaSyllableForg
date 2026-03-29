use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let vocab_path = "vocab.json";
    let data_dir = "./data";
    let output_file = "dataset.bin";

    // Khớp với max_vocab_len trong generate_vocab của bạn
    let max_vocab_len: u16 = 16_000;

    // 1. Đọc file vocab.json vào Ram
    println!("Đang nạp vocab.json...");
    let mut vocab_content = String::new();
    File::open(vocab_path)?.read_to_string(&mut vocab_content)?;

    // Parse JSON thành HashMap<String, u16>
    let vocab_json: HashMap<String, u16> = serde_json::from_str(&vocab_content)
        .expect("Không thể parse vocab.json. Hãy chắc chắn file tồn tại và đúng định dạng.");

    // Lấy ID của BOS và EOS (mặc định là 2 và 3 nếu không tìm thấy)
    let bos_id = *vocab_json.get("[BOS]").unwrap_or(&2);
    let eos_id = *vocab_json.get("[EOS]").unwrap_or(&3);

    // Regex đã được cập nhật để tách rời chữ số (\d)
    let re = Regex::new(r"(?u)(\n|\t|[ ]|\\{1,2}[\w]|_|\d|[^\W_\d]+|[^\w\s])").unwrap();

    // Mở file nhị phân để ghi
    let mut output = File::create(output_file)?;
    let mut total_tokens: usize = 0;
    let mut fallback_count: usize = 0;

    println!("--- BẮT ĐẦU ENCODE DỮ LIỆU ---");

    // 2. Quét qua thư mục data
    for entry in WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with('.'))
            {
                continue;
            }

            let mut content = String::new();
            if let Ok(_) = File::open(path)?.read_to_string(&mut content) {
                // [BOS] - Đánh dấu bắt đầu một phẩm/kinh (file)
                output.write_all(&bos_id.to_le_bytes())?;
                total_tokens += 1;

                // Tokenize nội dung
                for cap in re.captures_iter(&content) {
                    let token = cap[0].to_lowercase();

                    if let Some(&id) = vocab_json.get(&token) {
                        // NẾU TÌM THẤY: Ghi ID (dạng 16-bit Little Endian)
                        output.write_all(&id.to_le_bytes())?;
                        total_tokens += 1;
                    } else {
                        // NẾU KHÔNG TÌM THẤY (BYTE FALLBACK): Bẻ thành byte UTF-8
                        for b in token.bytes() {
                            // Cú pháp ID = max_vocab_len + byte (16000 -> 16255)
                            let byte_id = max_vocab_len + (b as u16);
                            output.write_all(&byte_id.to_le_bytes())?;
                            total_tokens += 1;
                            fallback_count += 1;
                        }
                    }
                }

                // [EOS] - Đánh dấu kết thúc phẩm/kinh
                output.write_all(&eos_id.to_le_bytes())?;
                total_tokens += 1;
            }
        }
    }

    println!("--- HOÀN TẤT! ---");
    println!("Đã lưu file binary: {}", output_file);
    println!("Tổng số tokens đã encode: {}", total_tokens);
    println!(
        "Số lượng Byte Fallback (Từ lạ bị bẻ nhỏ): {}",
        fallback_count
    );

    Ok(())
}
