// TODO: Profile with coz
// TODO: Figure out user interface

#![feature(custom_derive)]
#![feature(box_patterns)]
#![feature(plugin)]
#![plugin(clippy)]
#![plugin(peg_syntax_ext)]
#![plugin(phf_macros)]
#![plugin(serde_macros)]

extern crate bodyparser;
#[macro_use]
extern crate chan;
#[macro_use]
extern crate clap;
extern crate crypto;
extern crate cssparser;
extern crate daggy;
extern crate html5ever;
extern crate hyper;
extern crate iron;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate logger;
#[macro_use]
extern crate mime;
extern crate mount;
extern crate num_cpus;
extern crate phf;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate router;
extern crate rusqlite;
extern crate scoped_threadpool;
extern crate serde;
extern crate serde_json;
extern crate staticfile;
extern crate tendril;
extern crate time;
extern crate url;

use tasks::State;


mod download;
mod parser;
mod server;
mod storage;
mod tasks;


fn main() {
    // Initialize logging
    log4rs::init_file("logging.toml", Default::default()).unwrap();
    debug!("Starting up");

    let args = clap_app!(archer =>
        (version: "1.0")
        (author: "Markus Siemens <markus@m-siemens.de>")
        (about: "The Website Archiver")
        (@arg server: --server "Start the web server")
    )
                   .get_matches();

    if args.occurrences_of("server") == 1 {
        server::run();
    } else {
        let start_time = time::now();

        // Set up website map
        // TODO: Replace with db access
        let websites = storage::load_queued();

        // TODO: Remove
        storage::purge_resources();

        // Go!
        // TODO: The CPU count might be not the best heuristic here...
        debug!("Starting downloads");
        let mut ctx = tasks::TaskContext::new(&websites, (num_cpus::get() * 2) as u32);
        while let State::Running = ctx.process() {}

        print!("Done. Run time: ");
        print_duration(time::now() - start_time);
    }
}


/// Print a duration formatted beautifully
fn print_duration(duration: time::Duration) {
    let days = duration.num_days();
    let mut hours = duration.num_hours();
    let mut minutes = duration.num_minutes();
    let mut seconds = duration.num_seconds();

    if days > 0 {
        print!("{}d ", days);
        hours -= days * 24;
        minutes -= hours * 60 * 24;
        seconds -= seconds * 60 * 60 * 24;
    }

    if hours > 0 {
        print!("{}h ", hours);
        minutes -= hours * 60;
        seconds -= seconds * 60 * 60;
    }

    if minutes > 0 {
        print!("{}h ", minutes);
        seconds -= seconds * 60;
    }

    println!("{}s", seconds);
}
