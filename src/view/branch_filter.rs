use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{
    color::ColorTheme,
    event::UserEvent,
    git::{BranchFilter, Head, Ref, Repository},
};

#[derive(Debug)]
pub struct BranchFilterDialog {
    options: Vec<(BranchFilter, String)>,
    state: ListState,
    height: usize,
}

impl BranchFilterDialog {
    pub fn new(repository: &Repository) -> Self {
        let auto = match repository.head() {
            Head::Branch { name } => format!("Auto — {name}"),
            Head::Detached { .. } => "Auto — detached HEAD".into(),
            Head::None => "Auto — all (no checkout)".into(),
        };
        let mut options = vec![
            (BranchFilter::All, "All".into()),
            (BranchFilter::Auto, auto),
        ];
        let mut branches: Vec<_> = repository
            .all_refs()
            .into_iter()
            .filter_map(|r| match r {
                Ref::Branch { name, .. } => Some((
                    BranchFilter::Branch(format!("refs/heads/{name}")),
                    format!("{name} (local)"),
                )),
                Ref::RemoteBranch { name, .. } => Some((
                    BranchFilter::Branch(format!("refs/remotes/{name}")),
                    format!("{name} (remote)"),
                )),
                _ => None,
            })
            .collect();
        branches.sort_by(|a, b| a.1.cmp(&b.1));
        options.extend(branches);
        let selected = options
            .iter()
            .position(|(filter, _)| filter == repository.branch_filter())
            .unwrap_or(0);
        Self {
            options,
            state: ListState::default().with_selected(Some(selected)),
            height: 1,
        }
    }

    pub fn selected(&self) -> BranchFilter {
        self.options[self.state.selected().unwrap_or(0)].0.clone()
    }

    pub fn handle_event(&mut self, event: UserEvent) {
        let selected = self.state.selected().unwrap_or(0);
        let last = self.options.len() - 1;
        let selected = match event {
            UserEvent::NavigateDown | UserEvent::SelectDown => (selected + 1).min(last),
            UserEvent::NavigateUp | UserEvent::SelectUp => selected.saturating_sub(1),
            UserEvent::PageDown => selected.saturating_add(self.height).min(last),
            UserEvent::PageUp => selected.saturating_sub(self.height),
            UserEvent::GoToTop => 0,
            UserEvent::GoToBottom => last,
            _ => selected,
        };
        self.state.select(Some(selected));
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &ColorTheme) {
        let width = area.width.min(76);
        let height = area.height.min((self.options.len() + 2).min(24) as u16);
        let dialog_area = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        self.height = height.saturating_sub(2).max(1) as usize;
        let list = List::new(
            self.options
                .iter()
                .map(|(_, label)| ListItem::new(label.as_str())),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Show branch ")
                .title_bottom(" ↑/↓: select · Enter: apply · Esc: cancel "),
        )
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("› ");
        f.render_stateful_widget(list, dialog_area, &mut self.state);
    }
}
