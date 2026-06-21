#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub pages: Vec<ExtractedPage>,
}

#[derive(Debug, Clone)]
pub struct ExtractedPage {
    pub page_number: i32,
    pub text: String,
}
