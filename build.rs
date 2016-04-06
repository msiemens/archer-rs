extern crate mime_guess;
extern crate serde;
extern crate serde_json;

use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
// use std::process::{Command, Stdio};

use mime_guess::guess_mime_type;

macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::prelude::*;
            if let Err(e) = write!(&mut ::std::io::stderr(), "{}\n", format_args!($($arg)*)) {
                panic!("Failed to write to stderr.\
                    \nOriginal error output: {}\
                    \nSecondary error writing to stderr: {}", format!($($arg)*), e);
            }
        }
    )
}

fn main() {
    let profile = env::var("PROFILE").unwrap();

    if profile == "release" {
        env::set_current_dir(&Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())).unwrap();

        // Let the source code know it's a release build
        // println!("cwd: {}", env::current_dir().unwrap().display());
        println!("cargo:rustc-cfg=release");

        /*
        // Install NPM dependencies
        print_err!("Installing NPM dependencies...");
        let proc = Command::new("npm.cmd")
            .arg("install")
            .arg("--only=prod")
            .stdout(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("Failed to install NPM dependencies: {}", e));
        proc.wait();

        print_err!("Installing NPM dev dependencies...");
        Command::new("npm.cmd")
            .arg("install")
            .arg("--only=dev")
            .status()
            .unwrap_or_else(|e| panic!("Failed to install NPM dev dependencies: {}", e));

        // Build release version of the web UI
        print_err!("Building web UI...");
        Command::new("npm.cmd")
            .arg("run")
            .arg("build")
            .status()
            .unwrap_or_else(|e| panic!("Failed to build web UI: {}", e));
        */

        // Generate assets router
        // print_err!("Generating assets router...");
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("server_assets.rs");

        let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let assets_path = Path::new(&root_dir).join("public");
        let manifest_path = assets_path.join("manifest.json");

        let manifest_file = File::open(&manifest_path).unwrap();
        let mut files: Vec<String> = serde_json::from_reader::<File,
                                                               BTreeMap<String,
                                                                        String>>(manifest_file)
                                         .unwrap()
                                         .values()
                                         .cloned()
                                         .collect();
        files.push("index.html".to_owned());

        let mut dest_file = File::create(&dest_path).unwrap();
        dest_file.write_all(b"
        {
            let mut router = Router::new();

            use iron::prelude::*;
        ")
                 .unwrap();
        // router.get(\"/\", handler);

        for file in &files {
            let path = assets_path.join(file);
            let mut f = File::open(&path).unwrap();
            let mut contents = Vec::new();
            f.read_to_end(&mut contents).unwrap();

            write!(&mut dest_file,
                   "
            fn handle_{name}(_: &mut Request) -> IronResult<Response> {{
                use std::str::FromStr;

                use mime::Mime;

                let contents: &[u8] = &{contents:?};
                Ok(Response::with((status::Ok, contents, Mime::from_str(\"{mime}\").unwrap())))
            }}

            router.get(\"/{url}\", handle_{name});
                    ",
                   name = path.file_name().unwrap().to_str().unwrap().replace('.', "_"),
                   url = file.replace("index.html", ""),
                   contents = contents,
                   mime = guess_mime_type(&path))
                .unwrap();
        }

        dest_file.write_all(b"
            return router;
        }
        ").unwrap();
    }
}
