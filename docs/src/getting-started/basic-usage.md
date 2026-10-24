# Basic Usage

Run `serie` in the directory where your git repository exists.

```
$ cd <your git repository>
$ serie
```

If you want to specify it using command line options, see [Command Line Options](./command-line-options.md).

Press `F` in the commit list to open the branch filter dialog. Choose **All** to
show all history, **Auto** to follow the current worktree's checkout (including a
detached HEAD), or a named local or remote branch to show its reachable history.
Use Up/Down or `j`/`k` to select, Page Up/Down to scroll, Enter to apply, and Esc
to cancel. The filter is preserved by refresh (`R`). Auto rechecks HEAD on refresh;
without a checkout it shows all branches. If a named branch is deleted, refresh
falls back to All. Filtering only changes the displayed history.

The graph highlights the selected commit's first-parent lineage and dims other
connections, including merge-side lines. Commit circles retain their colors.
Wide Sixel graphs automatically center on the selected circle when selection
changes; `h`/`l` and Left/Right also let you pan manually.
