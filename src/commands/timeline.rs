//! Command handlers for sequence, banner, and gantt diagrams.

use super::JsonCharset;
use figo::diagrams::banner;
use figo::diagrams::gantt::{GanttChart, GanttSection, GanttTask, TimeUnit};
use figo::diagrams::sequence::SequenceDiagram;
use figo::error::Result;
use serde::Deserialize;

// -- Sequence -------------------------------------------------------------------

#[derive(Deserialize)]
struct SequenceInput {
    width: usize,
    charset: JsonCharset,
    participants: Vec<String>,
    messages: Vec<SequenceMessageJson>,
    #[serde(default)]
    color: bool,
}

#[derive(Deserialize)]
struct SequenceMessageJson {
    from: String,
    to: String,
    label: String,
}

pub fn run_sequence(input: &str) -> Result<String> {
    let inp: SequenceInput = serde_json::from_str(input)?;
    let mut sd = SequenceDiagram::new(inp.width, inp.charset.into()).color(inp.color);
    for p in &inp.participants {
        sd = sd.add_participant(p);
    }
    for m in &inp.messages {
        sd = sd.add_message(&m.from, &m.to, &m.label);
    }
    sd.build()
}

// -- Banner -------------------------------------------------------------------

#[derive(Deserialize)]
struct BannerInput {
    width: usize,
    #[allow(dead_code)]
    charset: JsonCharset,
    text: String,
    #[serde(default)]
    #[allow(dead_code)]
    color: bool,
}

pub fn run_banner(input: &str) -> Result<String> {
    let inp: BannerInput = serde_json::from_str(input)?;
    banner::draw_banner(&inp.text, inp.width)
}

// -- Gantt -------------------------------------------------------------------

#[derive(Deserialize)]
struct GanttInput {
    width: usize,
    charset: JsonCharset,
    time_unit: String,
    #[serde(default)]
    #[allow(dead_code)]
    start_date: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    end_date: Option<String>,
    total_units: Option<usize>,
    today_marker: Option<usize>,
    sections: Vec<GanttSectionJson>,
    #[serde(default)]
    #[allow(dead_code)]
    color: bool,
}

#[derive(Deserialize)]
struct GanttSectionJson {
    label: String,
    tasks: Vec<GanttTaskJson>,
}

#[derive(Deserialize)]
struct GanttTaskJson {
    name: String,
    start: usize,
    duration: usize,
    #[serde(default)]
    progress: u8,
    #[serde(default)]
    milestone: bool,
    #[serde(default)]
    depends_on: Option<String>,
}

pub fn run_gantt(input: &str) -> Result<String> {
    let inp: GanttInput = serde_json::from_str(input)?;
    let time_unit = match inp.time_unit.as_str() {
        "hour" => TimeUnit::Hour,
        "week" => TimeUnit::Week,
        "month" => TimeUnit::Month,
        _ => TimeUnit::Day,
    };
    let total_units = inp.total_units.unwrap_or(30);
    let mut gc =
        GanttChart::new(inp.width, inp.charset.into(), time_unit, total_units).color(inp.color);
    if let Some(today) = inp.today_marker {
        gc = gc.today_marker(today);
    }
    for s in inp.sections {
        let tasks: Vec<GanttTask> = s
            .tasks
            .into_iter()
            .map(|t| GanttTask {
                name: t.name,
                start: t.start,
                duration: t.duration,
                progress: t.progress,
                milestone: t.milestone,
                depends_on: t.depends_on,
            })
            .collect();
        gc = gc.add_section(GanttSection { label: s.label, tasks });
    }
    gc.build()
}
