//! Gantt chart diagrams for project management.

use std::fmt;

use crate::canvas::{Canvas, Layer};
use crate::error::{FigoError, Result};
use crate::style::{BorderStyle, Charset};

/// Time unit for the Gantt scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Hour,
    Day,
    Week,
    Month,
}

/// A single task in a Gantt chart.
#[derive(Debug, Clone)]
pub struct GanttTask {
    pub name: String,
    pub start: usize,
    pub duration: usize,
    pub progress: u8,
    pub milestone: bool,
    pub depends_on: Option<String>,
}

/// A section grouping tasks together.
#[derive(Debug, Clone)]
pub struct GanttSection {
    pub label: String,
    pub tasks: Vec<GanttTask>,
}

/// Builder for Gantt chart diagrams.
pub struct GanttChart {
    width: usize,
    charset: Charset,
    #[allow(dead_code)]
    time_unit: TimeUnit,
    total_units: usize,
    sections: Vec<GanttSection>,
    today_offset: Option<usize>,
    color: bool,
}

impl GanttChart {
    /// Create a new Gantt chart builder.
    pub fn new(width: usize, charset: Charset, time_unit: TimeUnit, total_units: usize) -> Self {
        Self {
            width,
            charset,
            time_unit,
            total_units,
            sections: Vec::new(),
            today_offset: None,
            color: false,
        }
    }

    /// Add a section with tasks.
    pub fn add_section(mut self, section: GanttSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a single task to the last added section.
    pub fn add_task(mut self, task: GanttTask) -> Self {
        if self.sections.is_empty() {
            self.sections.push(GanttSection { label: String::new(), tasks: Vec::new() });
        }
        self.sections.last_mut().unwrap().tasks.push(task);
        self
    }

    /// Set the today marker offset.
    pub fn today_marker(mut self, offset: usize) -> Self {
        self.today_offset = Some(offset);
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render and return as a `String`.
    pub fn build(&self) -> Result<String> {
        if self.sections.is_empty() || self.sections.iter().all(|s| s.tasks.is_empty()) {
            return Err(FigoError::MissingFields("no tasks specified".into()));
        }

        let glyphs = BorderStyle::Single.glyphs(self.charset);
        let label_width = 20usize.min(self.width / 4).max(10);
        let chart_width = self.width.saturating_sub(label_width + 2);
        let col_per_unit = chart_width / self.total_units.max(1);
        let chart_width = col_per_unit * self.total_units;

        let task_count: usize = self.sections.iter().map(|s| s.tasks.len()).sum();
        let section_count: usize = self.sections.iter().filter(|s| !s.label.is_empty()).count();
        let total_rows = task_count + section_count;
        let total_height = total_rows + 4; // header + tasks + borders

        let display_w = label_width + 2 + chart_width;
        let mut canvas = Canvas::new(display_w, total_height);

        // Outer border
        canvas.draw_rect(0, 0, display_w, total_height, &glyphs)?;

        // Separator between labels and chart area
        canvas.put_vertical(label_width + 1, 1, total_height - 2, glyphs.vertical);
        canvas.put(label_width + 1, 0, glyphs.tee_down);
        canvas.put(label_width + 1, total_height - 1, glyphs.tee_up);

        // Time scale header — only emit numeric labels when a single unit
        // column is wide enough to hold the digits without overlap. When the
        // chart is cramped (col_per_unit is small) we draw tick marks and
        // every Nth label so headers remain readable.
        let chart_start = label_width + 2;
        let label_step = if col_per_unit >= 3 {
            1
        } else if col_per_unit >= 2 {
            5
        } else {
            10
        };
        for u in 0..self.total_units {
            let x = chart_start + u * col_per_unit;
            if u % label_step == 0 || u + 1 == self.total_units {
                let label = format!("{u}");
                canvas.put_str(x, 1, &label);
            } else {
                canvas.put_layered(x, 1, glyphs.tee_down, Layer::Grid, None);
            }
        }

        // Separator under header
        canvas.put_horizontal(chart_start, 2, chart_width, glyphs.horizontal);

        // Draw tasks
        let mut row = 3;
        for section in &self.sections {
            if !section.label.is_empty() {
                canvas.put_str(1, row, &section.label);
                row += 1;
            }
            for task in &section.tasks {
                let indent = if section.label.is_empty() { "" } else { "  " };
                canvas.put_str(1, row, &format!("{indent}{}", task.name));

                let bar_start = chart_start + task.start * col_per_unit;
                let bar_len = (task.duration * col_per_unit).max(1);

                if task.milestone {
                    let diamond = match self.charset {
                        Charset::Unicode => "◆",
                        Charset::Ascii => "*",
                    };
                    canvas.put_str(bar_start, row, diamond);
                } else {
                    let filled = (bar_len as f64 * task.progress as f64 / 100.0) as usize;
                    for i in 0..bar_len {
                        let ch = if i < filled {
                            match self.charset {
                                Charset::Unicode => '█',
                                Charset::Ascii => '#',
                            }
                        } else {
                            match self.charset {
                                Charset::Unicode => '░',
                                Charset::Ascii => '.',
                            }
                        };
                        canvas.put(bar_start + i, row, ch);
                    }
                }

                row += 1;
            }
        }

        // Today marker
        if let Some(today_off) = self.today_offset {
            let today_x = chart_start + today_off * col_per_unit;
            if today_x < chart_start + chart_width {
                canvas.put_vertical(today_x, 1, total_height - 2, glyphs.vertical);
            }
        }

        Ok(canvas.render(self.color))
    }

    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for GanttChart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_gantt() {
        let gc = GanttChart::new(80, Charset::Ascii, TimeUnit::Day, 14).add_section(GanttSection {
            label: "Sprint 1".into(),
            tasks: vec![
                GanttTask {
                    name: "Task A".into(),
                    start: 0,
                    duration: 5,
                    progress: 100,
                    milestone: false,
                    depends_on: None,
                },
                GanttTask {
                    name: "Task B".into(),
                    start: 5,
                    duration: 7,
                    progress: 50,
                    milestone: false,
                    depends_on: Some("Task A".into()),
                },
            ],
        });
        let out = gc.build().unwrap();
        assert!(out.contains("Sprint 1"));
        assert!(out.contains("Task A"));
    }
}
