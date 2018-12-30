extern crate error_chain;
extern crate libcorporation;
extern crate env_logger;

fn main() {
    env_logger::init();

    if let Err(ref e) = libcorporation::run() {
        use std::io::Write;
        use error_chain::ChainedError;

        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        std::process::exit(1);
    }
}