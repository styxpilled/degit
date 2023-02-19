use clap::{arg, command};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([src] "Source repository"))
        // .arg(arg!(
        //     -d --debug ... "Turn debugging information on"
        // ))
        .get_matches();

    if let Some(src) = matches.get_one::<String>("src") {
        println!("Downloading {}", src);
        degit::degit(src);
    }
}
