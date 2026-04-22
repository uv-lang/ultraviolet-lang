use std::process;

pub fn print_help() -> ! {
    println!(
        "Ultraviolet lang v{}\
        \n\nUsage: ultraviolet-cli <file-path>
        ",
        env!("CARGO_PKG_VERSION")
    );

    process::exit(0);
}
