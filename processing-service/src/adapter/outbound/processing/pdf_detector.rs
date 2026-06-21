use lopdf::Document;

pub enum PdfType {
    Digital,
    Scanned,
    Invalid(String),
}

pub fn detect(content: &[u8]) -> PdfType {
    let doc = match Document::load_mem(content) {
        Ok(d) => d,
        Err(e) => return PdfType::Invalid(format!("corrupt or invalid PDF: {e}")),
    };

    for page_id in doc.page_iter() {
        if let Ok(content_data) = doc.get_page_content(page_id) {
            let text = String::from_utf8_lossy(&content_data);
            if text.contains("Tj") || text.contains("TJ") || text.contains("BT") {
                return PdfType::Digital;
            }
        }
    }

    PdfType::Scanned
}
