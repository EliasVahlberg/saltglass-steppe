use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use crate::{IssueCategory, IssueSeverity};

#[derive(Default)]
pub struct IssueReporter {
    pub active: bool,
    pub step: IssueStep,
    pub description: String,
    pub steps: Vec<String>,
    pub current_step: String,
    pub expected: String,
    pub actual: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub cursor_pos: usize,
}

#[derive(Default, PartialEq)]
pub enum IssueStep {
    #[default]
    Description,
    Steps,
    Expected,
    Actual,
    Severity,
    Category,
    Review,
}

impl IssueReporter {
    pub fn open(&mut self) {
        self.active = true;
        self.step = IssueStep::Description;
        self.clear_all();
    }

    pub fn close(&mut self) {
        self.active = false;
        self.clear_all();
    }

    fn clear_all(&mut self) {
        self.description.clear();
        self.steps.clear();
        self.current_step.clear();
        self.expected.clear();
        self.actual.clear();
        self.severity = IssueSeverity::Medium;
        self.category = IssueCategory::Gameplay;
        self.cursor_pos = 0;
    }

    pub fn next_step(&mut self) {
        match self.step {
            IssueStep::Description => {
                if !self.description.trim().is_empty() {
                    self.step = IssueStep::Steps;
                }
            }
            IssueStep::Steps => self.step = IssueStep::Expected,
            IssueStep::Expected => {
                if !self.expected.trim().is_empty() {
                    self.step = IssueStep::Actual;
                }
            }
            IssueStep::Actual => {
                if !self.actual.trim().is_empty() {
                    self.step = IssueStep::Severity;
                }
            }
            IssueStep::Severity => self.step = IssueStep::Category,
            IssueStep::Category => self.step = IssueStep::Review,
            IssueStep::Review => {} // Submit handled elsewhere
        }
    }

    pub fn prev_step(&mut self) {
        match self.step {
            IssueStep::Description => {}
            IssueStep::Steps => self.step = IssueStep::Description,
            IssueStep::Expected => self.step = IssueStep::Steps,
            IssueStep::Actual => self.step = IssueStep::Expected,
            IssueStep::Severity => self.step = IssueStep::Actual,
            IssueStep::Category => self.step = IssueStep::Severity,
            IssueStep::Review => self.step = IssueStep::Category,
        }
    }

    pub fn add_step(&mut self) {
        if !self.current_step.trim().is_empty() {
            self.steps.push(self.current_step.clone());
            self.current_step.clear();
        }
    }

    pub fn remove_last_step(&mut self) {
        self.steps.pop();
    }

    pub fn push_char(&mut self, c: char) {
        match self.step {
            IssueStep::Description => self.description.push(c),
            IssueStep::Steps => self.current_step.push(c),
            IssueStep::Expected => self.expected.push(c),
            IssueStep::Actual => self.actual.push(c),
            _ => {}
        }
    }

    pub fn pop_char(&mut self) {
        match self.step {
            IssueStep::Description => { self.description.pop(); }
            IssueStep::Steps => { self.current_step.pop(); }
            IssueStep::Expected => { self.expected.pop(); }
            IssueStep::Actual => { self.actual.pop(); }
            _ => {}
        }
    }

    pub fn cycle_severity(&mut self) {
        self.severity = match self.severity {
            IssueSeverity::Low => IssueSeverity::Medium,
            IssueSeverity::Medium => IssueSeverity::High,
            IssueSeverity::High => IssueSeverity::Critical,
            IssueSeverity::Critical => IssueSeverity::Low,
        };
    }

    pub fn cycle_category(&mut self) {
        self.category = match self.category {
            IssueCategory::Gameplay => IssueCategory::UI,
            IssueCategory::UI => IssueCategory::Performance,
            IssueCategory::Performance => IssueCategory::Save,
            IssueCategory::Save => IssueCategory::Combat,
            IssueCategory::Combat => IssueCategory::AI,
            IssueCategory::AI => IssueCategory::Map,
            IssueCategory::Map => IssueCategory::Other,
            IssueCategory::Other => IssueCategory::Gameplay,
        };
    }

    pub fn is_complete(&self) -> bool {
        !self.description.trim().is_empty() &&
        !self.expected.trim().is_empty() &&
        !self.actual.trim().is_empty()
    }
}

pub fn render_issue_reporter(f: &mut Frame, reporter: &IssueReporter) {
    if !reporter.active {
        return;
    }

    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("Issue Reporter")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let content = match reporter.step {
        IssueStep::Description => {
            let text = format!(
                "Step 1/6: Issue Description\n\n{}\n\n[Enter] Next | [Esc] Cancel",
                if reporter.description.is_empty() { 
                    "Describe the issue you encountered..." 
                } else { 
                    &reporter.description 
                }
            );
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Steps => {
            let mut text = "Step 2/6: Reproduction Steps\n\n".to_string();
            text.push_str("Current steps:\n");
            for (i, step) in reporter.steps.iter().enumerate() {
                text.push_str(&format!("{}. {}\n", i + 1, step));
            }
            text.push_str(&format!("\nAdd step: {}\n", reporter.current_step));
            text.push_str("\n[Enter] Add Step | [Enter on empty] Next Step | [Backspace] Remove Last | [Esc] Cancel");
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Expected => {
            let text = format!(
                "Step 3/6: Expected Behavior\n\n{}\n\n[Enter] Next | [Esc] Cancel",
                if reporter.expected.is_empty() { 
                    "What did you expect to happen?" 
                } else { 
                    &reporter.expected 
                }
            );
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Actual => {
            let text = format!(
                "Step 4/6: Actual Behavior\n\n{}\n\n[Enter] Next | [Esc] Cancel",
                if reporter.actual.is_empty() { 
                    "What actually happened?" 
                } else { 
                    &reporter.actual 
                }
            );
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Severity => {
            let text = format!(
                "Step 5/6: Severity\n\nCurrent: {:?}\n\n[Space] Cycle | [Enter] Next | [Esc] Cancel",
                reporter.severity
            );
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Category => {
            let text = format!(
                "Step 6/6: Category\n\nCurrent: {:?}\n\n[Space] Cycle | [Enter] Next | [Esc] Cancel",
                reporter.category
            );
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
        IssueStep::Review => {
            let mut text = "Review Issue Report\n\n".to_string();
            text.push_str(&format!("Description: {}\n\n", reporter.description));
            text.push_str("Steps:\n");
            for (i, step) in reporter.steps.iter().enumerate() {
                text.push_str(&format!("{}. {}\n", i + 1, step));
            }
            text.push_str(&format!("\nExpected: {}\n", reporter.expected));
            text.push_str(&format!("Actual: {}\n", reporter.actual));
            text.push_str(&format!("Severity: {:?}\n", reporter.severity));
            text.push_str(&format!("Category: {:?}\n\n", reporter.category));
            text.push_str("[Enter] Submit | [Backspace] Back | [Esc] Cancel");
            Paragraph::new(text).wrap(Wrap { trim: true })
        }
    };

    f.render_widget(content, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
