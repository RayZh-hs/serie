use std::{path::Path, process::Command};

use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use tempfile::tempdir;

use crate::{
    color::ColorTheme,
    event::UserEvent,
    git::{BranchFilter, Repository, SortCommit},
    test_support::git::GitRepository,
    view::branch_filter::BranchFilterDialog,
};

fn git(path: &Path, args: &[&str]) {
    assert!(Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .unwrap()
        .success());
}

fn subjects(path: &Path, filter: BranchFilter, max_count: Option<usize>) -> Vec<String> {
    Repository::load_filtered(path, SortCommit::Topological, max_count, false, filter)
        .unwrap()
        .all_commits()
        .iter()
        .map(|commit| commit.subject.clone())
        .collect()
}

#[test]
fn branch_filter_loads_reachable_history_before_applying_commit_limit() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let git_repo = GitRepository::new(path);
    git_repo.init();
    git_repo.commit("root", "2025-01-01");
    git_repo.checkout_b("topic");
    git_repo.commit("topic", "2025-01-02");
    git_repo.checkout("master");
    git_repo.commit("main", "2025-01-03");
    git(path, &["update-ref", "refs/remotes/origin/topic", "topic"]);
    git(path, &["tag", "topic", "master"]); // same name as branch

    assert_eq!(subjects(path, BranchFilter::All, None).len(), 3);
    assert_eq!(subjects(path, BranchFilter::Auto, None), ["main", "root"]);
    assert_eq!(
        subjects(path, BranchFilter::Branch("refs/heads/topic".into()), None),
        ["topic", "root"]
    );
    assert_eq!(
        subjects(
            path,
            BranchFilter::Branch("refs/remotes/origin/topic".into()),
            Some(1)
        ),
        ["topic"]
    );
    git_repo.merge(&["refs/heads/topic"], "2025-01-04");
    assert_eq!(subjects(path, BranchFilter::Auto, None).len(), 4);
}

#[test]
fn auto_tracks_checkout_and_detached_head_and_deleted_branch_falls_back() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let git_repo = GitRepository::new(path);
    git_repo.init();
    git_repo.commit("root", "2025-01-01");
    git_repo.checkout_b("topic");
    git_repo.commit("topic", "2025-01-02");
    assert_eq!(subjects(path, BranchFilter::Auto, None), ["topic", "root"]);
    git(path, &["checkout", "--detach", "master"]);
    assert_eq!(subjects(path, BranchFilter::Auto, None), ["root"]);
    git_repo.branch_d("topic");
    let repo = Repository::load_filtered(
        path,
        SortCommit::Topological,
        None,
        false,
        BranchFilter::Branch("refs/heads/topic".into()),
    )
    .unwrap();
    assert_eq!(repo.branch_filter(), &BranchFilter::All);
}

#[test]
fn dialog_keeps_hidden_branches_available_and_scrolls_in_small_terminals() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let git_repo = GitRepository::new(path);
    git_repo.init();
    git_repo.commit("root", "2025-01-01");
    git_repo.checkout_b("topic");
    git_repo.commit("topic", "2025-01-02");
    let repo = Repository::load_filtered(
        path,
        SortCommit::Topological,
        None,
        false,
        BranchFilter::Branch("refs/heads/master".into()),
    )
    .unwrap();
    let mut dialog = BranchFilterDialog::new(&repo);
    assert_eq!(
        dialog.selected(),
        BranchFilter::Branch("refs/heads/master".into())
    );
    dialog.handle_event(UserEvent::GoToBottom);
    assert_eq!(
        dialog.selected(),
        BranchFilter::Branch("refs/heads/topic".into())
    );
    let mut terminal = Terminal::new(TestBackend::new(50, 5)).unwrap();
    terminal
        .draw(|f| dialog.render(f, f.area(), &ColorTheme::default()))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("topic (local)"));
    terminal
        .draw(|f| dialog.render(f, Rect::new(0, 0, 1, 1), &ColorTheme::default()))
        .unwrap();
    dialog.handle_event(UserEvent::GoToTop);
    assert_eq!(dialog.selected(), BranchFilter::All);
    dialog.handle_event(UserEvent::NavigateDown);
    assert_eq!(dialog.selected(), BranchFilter::Auto);
}
