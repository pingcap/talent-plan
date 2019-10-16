# Contributing to PNA Rust

Contributions of any kind are welcome.

File bugs on the [issue tracker], no matter how small. If something in the
course didn't make sense to you then it won't make sense to somebody else and
needs to be fixed. That includes the language: this course is intended to be
accessible to those with modest English-language comprehension. Non-technical
words should be simple, and grammar should be easy to follow.

General feedback and suggestions can be submitted to the issue tracker or to the
#rust-training channel on the [TiKV Slack].

When you see something inconsistent or confusing, consider fixing it directly
and sending a pull request.

For those looking for something to contribute, on the issue tracker, issues for
the Rust course are tagged `p: rust`. The `help wanted` and `good first issue`
tags may help you find something interesting.

[issue tracker]: https://github.com/pingcap/talent-plan/issues/
[TiKV Slack]: https://join.slack.com/t/tikv-wg/shared_invite/enQtNTUyODE4ODU2MzI0LWVlMWMzMDkyNWE5ZjY1ODAzMWUwZGVhNGNhYTc3MzJhYWE0Y2FjYjliYzY1OWJlYTc4OWVjZWM1NDkwN2QxNDE

## Developing a new project

Each project expands the scope of the previous, and builds off of
learnings from previous projects. Each project lends itself to being extended by
the next section's project.

When writing a project, look for steps where the design could be specified in
multiple ways, where there are multiple solutions, where there is deeper
understanding to be gained, and ask questions formulated to get the reader to
think more deeply.

Projects should be written to take between 2 - 4 hours to implement.

When writing a project, look for optional "extension" steps that teach
additional practical subjects, but which either aren't necessary to complete the
project or require more time and skill to implement. Extension steps usually go
at the end of a project.

Project text doesn't link directly to documentation resources containing
solutions - students should learn where to get the answers from the
previous "building blocks" sections.

Project text may include inline links to pages that offer explanatios of terms
and concepts.

Some subjects are revisited multiple times. In particular it is common for one
project to introduce a subject with basic tasks, then the subsequent project to
expand on that same subject with deeper tasks.


## Style notes

Building blocks pages generally begin with the same encouraging phrase, and
project pages generally end with the same encouraging phrase.

Headers do not capitalize every word; words after a colon
are capitalized:

```
## Project spec

## Lesson: Tools and good bones
```

In projects, be clear on when the student should start hacking, and what they
should be hacking, by writing an imperative statement. Format that command in
italics:

```
_Try it now._
```

If there are project extension sections, the first one is preceeded by a
thematic break (`---`).

For markdown readability headers are always preceeded by two line breaks.

One-paragraph side-notes are in italics and preceeded with "Note:", like `_Note:
whatever._`

Larger tangents that don't move the project forward are in their own sections,
with their own headers, named "Aside: ...". They should be related to the
subject matter and be interesting enough to justify the large detour.

Internal hyperlinks are relative to the current directory, not absolute.

For building blocks exercises, if the description spans multiple paragraphs,
begin the description in a paragraph separate from the exercise name:

```
- **Exercise: Write a thread pool**.

  A [thread pool] runs jobs (functions) on a set of reusable threads, which can
  be more efficient than spawning a new thread for every job.

  Create a simple thread pool with the following type signature:
```


## Maintenace notes

Keep the project summary (the `**Task**`, `**Goals**`, etc. text) synced between
plan.md and the project description.

New documentation files that are not part of a project or building-blocks, and
not part of standard top-level project files go in `docs/` to keep the GitHub
directory listing clean, and keep the rendered README above "the fold".

Miscellaneous files that are not part of the user-visible projcet live in
`docs/etc`.
