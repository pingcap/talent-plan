TODO organize this

Every md file has a html relative symlink to index.html:

`ln -s index.html CONTRIBUTING.html`

The static website loads the page content from markdown dynamically,
based on the URL of its HTML page.

Project text doesn't link directly to documentation resources containing
solutions - students should learn where to get the answers from the
pre-requisites and lessons.

Project text may include inline links to pages that offer explanatios of terms
and concepts.

Keep the project summary (the **Task**, **Goals**, etc. text) synced between
plan.md and the project description.

Common sections of project descriptions:

- The project summary (directly after h1)
- "Introduction"
- "Project spec"

Headers do not capitalize every word; words after a colon
are capitalized:

> ## Project spec

> ## Lesson: Tools and good bones

In projects, be clear on when the student should start hacking, and what they
should be hacking, by writing an imperative statement. Format that command in
italics:

> <i>Use [crates.io](https://crates.io) to find the documentation
for the `clap` crate, and implement the command line interface
such that the `cli_*` test cases pass.</i>

> _Try it now._

## Developing a new section

Each section provides a similar project to previous sections, while expanding
the scope and building off of learnings from previous sections. Each project
lends itself to being extended by the next section's project.

New sections are developed in pairs, to benefit from the feedback.

Approximate workflow:

- Author 1 and 2 collaborate on an outline for the new section, adding it to
  plan.md, following the format of existing sections. This includes supporting lessons
  and readings. Submit PR.

- Author 1 writes the entire project description; optionally with test cases if
  that makes the writing easier. Add new background readings or otherwise modify
  the section plan as needed. Submit PR. Author 2 reviews.

- Author 2 writes the remaining test cases and an example solution project. Add
  new background readings or otherwise modify the section plan as needed. Submit
  PR. Author 1 reviews.

- Author 2 makes changes to the project text for clarification, improvement,
  scope expansion, etc. Add new background readings or otherwise modify the
  section plan as needed. Submit PR. Author 1 reviews.

##

When writing a project, look for steps where the design could be specified differently,
where there are multiple solutions, where there is deeper understanding to be gained,
and ask questions formulated to get the reader to think more deeply.

Projects should be written to take between 2 - 4 hours to implement.

When writing a project, look for optional "extension" steps that teach
additional practical subjects, but which either aren't necessary to complete the
project or require more time and skill to implement. Extenion steps go at the
end of a project.

## Contributor notes

- each md file needs an html relative symlink to index.html
- lessons need a .slides.html symlink
- links are generally to markdown files, not html files
  - exception: links to slides from plan go to the hosted website
  - all links rewritten when browsed locally
- in markdown, write links as relative to current directory
- keep "task", "goals", etc in project intros in sync between "plan.md" and
  project pages

