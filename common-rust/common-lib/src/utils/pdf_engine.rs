use baml_client::models::{ClassLevel, CompleteLessonContent, ContentSubPointText, Term};
use derive_typst_intoval::{IntoDict, IntoValue};
use std::fs;
use typst::foundations::{Bytes, Dict, IntoValue};
use typst_as_lib::TypstEngine;

use crate::AgentError;

// File paths - these should be in your Golem agent's filesystem
static TEMPLATE_PATH: &str = "/templates/template.typ";
static FONT_PATH: &str = "/fonts/times-new-roman.ttf";
static WATERMARK_PATH: &str = "/templates/images/watermark.png";

pub fn pdf_engine(
    lessons: Vec<CompleteLessonContent>,
    subject_name: &str,
    class_year: &str,
    mode: &str, // "pupil" or "teacher"
) -> Result<Vec<u8>, AgentError> {
    // Read template file at runtime
    let template_content = fs::read_to_string(TEMPLATE_PATH).map_err(|e| AgentError {
        message: format!("Could not read template file: {}", e),
        code: "TEMPLATE_READ_ERROR".to_string(),
    })?;

    // Read font file at runtime
    let font_bytes = fs::read(FONT_PATH).map_err(|e| AgentError {
        message: format!("Could not read font file: {}", e),
        code: "FONT_READ_ERROR".to_string(),
    })?;

    // Read watermark image at runtime
    let watermark_bytes = fs::read(WATERMARK_PATH).map_err(|e| AgentError {
        message: format!("Could not read watermark image: {}", e),
        code: "IMAGE_READ_ERROR".to_string(),
    })?; // Optional, so use .ok()

    // Build the typst engine with the template and fonts
    let template = TypstEngine::builder()
        .main_file(template_content.as_str())
        .fonts([font_bytes.as_slice()])
        .build();

    // convert incoming class string to corresponding string typst requires
    let class_str = match class_year {
        "PRIMARY_1" => "1",
        "PRIMARY_2" => "2",
        "PRIMARY_3" => "3",
        "PRIMARY_4" => "4",
        "PRIMARY_5" => "5",
        _ => "",
    };
    // Convert lessons to typst input format
    let input = TemplateInput {
        subject_name: subject_name.to_string(),
        class_year: class_str.to_string(),
        mode: mode.to_string(),
        lessons: lessons.into_iter().map(|l| l.into()).collect(),
        watermark_image: Some(Bytes::new(watermark_bytes)),
    };

    // Compile the template
    let doc = template
        .compile_with_input(input)
        .output
        .map_err(|e| AgentError {
            message: format!("Typst compilation failed: {:?}", e),
            code: "TYPST_COMPILE_ERROR".to_string(),
        })?;

    // Generate PDF
    let options = Default::default();
    let pdf = typst_pdf::pdf(&doc, &options).map_err(|e| AgentError {
        message: format!("PDF generation failed: {:?}", e),
        code: "PDF_GENERATION_ERROR".to_string(),
    })?;

    // Optionally write to file
    // fs::write(OUTPUT, &pdf).map_err(|e| AgentError {
    //     message: format!("Could not write PDF: {}", e),
    //     code: "FILE_WRITE_ERROR".to_string(),
    // })?;

    // Write to file (optional - for debugging/caching)
    // if let Err(e) = fs::write(OUTPUT_PATH, &pdf) {
    //     // Just log the error, don't fail if we can't write
    //     eprintln!("Warning: Could not write PDF to {}: {}", OUTPUT_PATH, e);
    // }

    Ok(pdf)
}

// Main input structure matching template's expectations
#[derive(Debug, Clone, IntoValue, IntoDict)]
struct TemplateInput {
    subject_name: String,
    class_year: String,
    mode: String,
    lessons: Vec<Lesson>,
    watermark_image: Option<Bytes>,
}

impl From<TemplateInput> for Dict {
    fn from(value: TemplateInput) -> Self {
        value.into_dict()
    }
}

// Lesson structure matching your template
#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Lesson {
    age_range: String,
    class_level: String,
    subject: String,
    week: i32,
    term: String,
    topic_title: String,
    duration_mins: i32,
    introduction: String,
    objectives: Vec<Objective>,
    materials: Vec<String>,
    prior_knowledge: Vec<String>,
    content_sections: Vec<TypstContentSection>,
    lesson_steps: Vec<TypstLessonStep>,
    key_points: Vec<String>,
    mcq_questions: Vec<McqQuestion>,
    theoretical_questions: Vec<TypstTheoreticalQuestion>,
    conclusion: String,
    teacher_tips: String,
    remediation: String,
    formative_assessment: String,
    summative_assessment: String,
    extension_activities: Vec<String>,
    primary_sources: Vec<String>,
    success_criteria: Vec<String>,
    textbook_references: Vec<String>,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Objective {
    objective: String,
    taxonomy_level: String,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct TypstContentSection {
    header: String,
    body: String,
    sub_points: Option<Vec<SubPoint>>,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct SubPoint {
    sub_number: String,
    text: SubPointText,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct SubPointText {
    body: String,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct TypstLessonStep {
    step_number: i32,
    phase: String,
    duration_mins: i32,
    teacher_actions: String,
    pupil_activities: String,
    teaching_strategy: String,
    assessment: Option<String>,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct McqQuestion {
    question: String,
    option_a: String,
    option_b: String,
    option_c: String,
    correct_answer: String,
    explanation: String,
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct TypstTheoreticalQuestion {
    question: String,
    parts: Vec<String>,
    model_answer: String,
    marking_scheme: String,
}

// Helper function to convert ClassLevel enum to string
fn class_level_to_string(level: ClassLevel) -> String {
    match level {
        ClassLevel::Primary1 => "PRIMARY_1".to_string(),
        ClassLevel::Primary2 => "PRIMARY_2".to_string(),
        ClassLevel::Primary3 => "PRIMARY_3".to_string(),
        ClassLevel::Primary4 => "PRIMARY_4".to_string(),
        ClassLevel::Primary5 => "PRIMARY_5".to_string(),
        ClassLevel::Jss1 => "JSS_1".to_string(),
        ClassLevel::Jss2 => "JSS_2".to_string(),
        ClassLevel::Jss3 => "JSS_3".to_string(),
    }
}

// Helper function to convert Term enum to string
fn term_to_string(term: Term) -> String {
    match term {
        Term::First => "FIRST".to_string(),
        Term::Second => "SECOND".to_string(),
        Term::Third => "THIRD".to_string(),
    }
}

// Helper function to extract text from ContentSubPointText enum
fn extract_text_from_subpoint(text: ContentSubPointText) -> String {
    match text {
        ContentSubPointText::String(s) => s,
        ContentSubPointText::ContentSection(cs) => cs.body,
    }
}

// Implement conversion from CompleteLessonContent to Lesson
impl From<CompleteLessonContent> for Lesson {
    fn from(content: CompleteLessonContent) -> Self {
        Lesson {
            age_range: content.age_range,
            class_level: class_level_to_string(content.class_level),
            subject: content.subject,
            week: content.week,
            term: term_to_string(content.term),
            topic_title: content.topic_title,
            duration_mins: content.duration_mins,
            introduction: content.introduction,
            objectives: content
                .objectives
                .into_iter()
                .map(|o| Objective {
                    objective: o.objective,
                    taxonomy_level: o.taxonomy_level,
                })
                .collect(),
            materials: content.materials,
            prior_knowledge: content.prior_knowledge,
            content_sections: content
                .content_sections
                .into_iter()
                .map(|cs| TypstContentSection {
                    header: cs.header,
                    body: cs.body,
                    sub_points: cs.sub_points.map(|sps| {
                        sps.into_iter()
                            .map(|sp| SubPoint {
                                sub_number: sp.sub_number,
                                text: SubPointText {
                                    body: extract_text_from_subpoint(sp.text),
                                },
                            })
                            .collect()
                    }),
                })
                .collect(),
            lesson_steps: content
                .lesson_steps
                .into_iter()
                .map(|ls| TypstLessonStep {
                    step_number: ls.step_number,
                    phase: ls.phase,
                    duration_mins: ls.duration_mins,
                    teacher_actions: ls.teacher_actions,
                    pupil_activities: ls.pupil_activities,
                    teaching_strategy: ls.teaching_strategy,
                    assessment: ls.assessment,
                })
                .collect(),
            key_points: content.key_points,
            mcq_questions: content
                .mcq_questions
                .into_iter()
                .map(|mcq| McqQuestion {
                    question: mcq.question,
                    option_a: mcq.option_a,
                    option_b: mcq.option_b,
                    option_c: mcq.option_c,
                    correct_answer: mcq.correct_answer,
                    explanation: mcq.explanation,
                })
                .collect(),
            theoretical_questions: content
                .theoretical_questions
                .into_iter()
                .map(|tq| TypstTheoreticalQuestion {
                    question: tq.question,
                    parts: tq.parts,
                    model_answer: tq.model_answer,
                    marking_scheme: tq.marking_scheme,
                })
                .collect(),
            conclusion: content.conclusion,
            teacher_tips: content.teacher_tips,
            remediation: content.remediation,
            formative_assessment: content.formative_assessment,
            summative_assessment: content.summative_assessment,
            extension_activities: content.extension_activities,
            primary_sources: content.primary_sources,
            success_criteria: content.success_criteria,
            textbook_references: content.textbook_references,
        }
    }
}
