use common_lib::PdfAgent;
use golem_rust::agent_implementation;

use crate::PdfImpl;

#[agent_implementation]
impl PdfAgent for PdfImpl {
    fn new(name: String) -> Self {
        Self { _name: name }
    }

    fn pdf_generator(&mut self, subject: String, class: String) {}
}
