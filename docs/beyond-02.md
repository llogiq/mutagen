# Beyond mutagen-0.2

The version 0.2 of mutagen is a complete rewrite of the framework while staying true to its original goals (see [Vision Document](./vision.md)).

## Status Quo

In mutagen-0.2, core features are implemented, which makes basic use of mutation testing in Rust possible.

* Basic set of mutations: Arithmetic and logical operations, statement deletion
* running the mutation-analysis via cargo-plugin
* text based report

## Community Feedback

At time of writing, feedback of the community has not been collected.

## Planned Features

### Web Report

A Web-based report is capable of displaying more information that cannot be presented in the textual report. It is planned to add some form of Web-based report to display mutation testing results, which can give a better overview but also display the information about the quality of the test suite in finer detail. Most other mutation testing frameworks for other languages provide similar functionality.

### Testcase Tracking and Skipping

Ideally, only the tests that cover the activated mutation are executed. However, this requires injecting a dynamic check into each unit test that allows to skip tests irrelevant to this mutation, which requires a new procedural macro. The technical limitations have not yet been studied, but this approach seems promising for improving performance of mutation analysis.

### Additional Mutators

The list of implemented mutators in mutagen-0.2 is a minimal set to make mutagen useful. In the next releases, further code patterns are planned to be mutated.

* loops
* string literals
* if conditions
* control flow: return early from functions, break early from loops, ...

### Customization

In mutagen-0.2, customization of mutators is not supported. We will engage with the community to determine what customizations are requested and considered useful.
