use common_lib::{
    utils::{fetch_lessons, pdf_engine::pdf_engine},
    PdfAgent, PdfFile,
};
use golem_rust::agent_implementation;

use crate::PdfImpl;

#[agent_implementation]
impl PdfAgent for PdfImpl {
    fn new(name: String) -> Self {
        Self { _name: name }
    }

    async fn pdf_generator(&mut self, subject: String, class: String, mode: String) -> PdfFile {
        let manual_records = fetch_lessons(subject.as_str(), class.as_str()).await;
        let manual_records = match manual_records {
            Ok(records) => records,
            Err(err) => {
                println!("Error: {}", err.message);
                return PdfFile {
                    content_type: "text/plain".to_string(),
                    data: err.message.into_bytes(),
                };
            }
        };
        let pdf_bytes = pdf_engine(manual_records, &subject, &class, &mode);

        match pdf_bytes {
            Ok(pdf) => PdfFile {
                content_type: "application/pdf".to_string(),
                data: pdf,
            },
            Err(err) => {
                println!("Error: {}", err.message);
                PdfFile {
                    content_type: "text/plain".to_string(),
                    data: err.message.into_bytes(),
                }
            }
        }
    }
}
