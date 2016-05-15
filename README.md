# The Article Archiver

*NOTE: Project unfinished and abandoned!*

The internet doesn’t forget, they say. Except when it does. More than once I went to an old bookmark only to be greeted by a 404 page. The goal of this project was to create local backup copies of these cool/insightful/interesting articles (I originally had a Python script that parsed a text file and executed a bunch of `wget` calls, but it didn’t work reliably).

Why is this project abandoned? After being annoyed with the bugs of my Python script, I set out to build a “proper” solution based on Rust. Halfway through the project, I happened to discover a simpler way: OneNote. The [OneNote Clipper](https://www.onenote.com/clipper) integrates with your web browser and is able to extract the contents of an article into OneNote. It works reasonably well and saves me approximately one or two months working on this project.

Still, the source code might be useful to folks out there, which is why the source code along with these notes is published on GitHub. Maybe someone can learn from it or wants to continue hacking on it.

## The Design

The project is split into two components:

1. The scraper.
2. The viewer.

### The Scraper

The basic data structure is a directed acyclic graph (DAG) of resources (HTML, CSS, JS, images) and their dependencies (see [`tasks/mod.rs`](https://github.com/msiemens/archer-rs/blob/master/src/tasks/mod.rs)). The main loop traverses the DAG in order to find open tasks. Tasks may be:

* Download a resource ([`download/`](https://github.com/msiemens/archer-rs/tree/master/src/download/)).
* Parse a resource to discover dependencies ([`parser/`](https://github.com/msiemens/archer-rs/tree/master/src/parser)).
* Store a resource in a SQLite database ([`storage/`](https://github.com/msiemens/archer-rs/tree/master/src/storage)).

#### Download Pool

For performance reasons the download part is executed in parallel. There’s a download pool implementation (see [`download/pool.rs`](https://github.com/msiemens/archer-rs/blob/master/src/download/pool.rs)) which fetches URLs from the main thread and feeds the resulting HTTP response back to the main loop for parsing.

#### Resource Parsing

Parsing serves two tasks: It discovers new resources the website depends on (think JS/CSS files, images etc.) and rewrites the URLs so they point to the viewer.

#### Storage

A SQLite file serves as a storage mechanism. The scraper puts the resources there and the viewer uses it to – well – let you view the websites.

#### Run It

To execute the downloader, run `$ cargo run` in your console.

### The Viewer

The viewer has two parts: a server and a JS-based web client. The server is written in Rust and based on [Iron](https://github.com/iron/iron) while the web client is written in JavaScript ES6 and uses [React](https://facebook.github.io/react/) and [Semantic UI](http://semantic-ui.com/) under the hood.

For serving the web client, there are two ways.
* In development, you can run `npm start` to compile the client and serve it on http://localhost:8080/. The backend is expected to be available at http://localhost:3000. To run the backend server, execute `$ cargo run -- --server`.
* In release mode, the client is bundeled into the executable using a `build.rs` script that inserts the compiled files into the Rust source. Don’t forget to run `$ npm build` first to compile the client. Then compile and run the server using `$ cargo run --release -- --server`.

## What’s left to do

The scraper is working so far, but is still very unpolished. Open tasks would be:

* Handle redirects properly. This is currently completely untested.
* Use GZIP when requesting content. Save some bandwidth.
* Send a `User-Agent` header. That’s what good scrapers do.
* Add support for `robots.txt` files. Don’t be a dick.
* Add a comprehensive test suite. Because reasons.

The viewer is work in progress. There’s a basic overview page that displays the websites that have been downloaded as well as an unfinished *Add Website to Queue* dialog. Note that currently no websites are shown as the appropriate table isn’t filled by the scraper. What’s left to do here:

* Finish the overview page and *Add Website* dialog.
* Add a Queue overview page.
* Add a page to add/modify/remove tags.

---

If you have questions about the project, feel free to ping me at Twitter ([@siem3m](https://twitter.com/siem3m)) or open a GitHub issue.