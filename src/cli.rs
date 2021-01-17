use clap::{App, Arg};

pub fn new<'a>() -> App<'a, 'a> {
    let result = App::new("pg-to-sqlite3")
        .version("0.0.0")
        .about("Dump a postgres database into a sqlite db as best possible")
        .arg(
            Arg::with_name("SRC")
                .long("src")
                .takes_value(true)
                .required(true)
                .help("a postgres connection string to your source database"),
        )
        .arg(
            Arg::with_name("schema")
                .long("schema")
                .takes_value(true)
                .default_value("public")
                .help("comma-separated schemas from which to copy"),
        )
        .arg(
            Arg::with_name("DEST")
                .long("dest")
                .takes_value(true)
                .required(true)
                .help("a path to a sqlite3 file"),
        );
    // TODO: --progress
    // TODO: --data-only | --schema-only
    // TODO: respoect PGHOST PGOPTIONS PGPORT PGUSER and listen for password
    return result;
}
// TODO: tests of cli-parsing

#[test]
fn test_parsing_src_and_dest() {
    let cli = new();
    let args = vec![
        "pg-to-sqlite3",
        "--src",
        "postgres://user:pw@dbhost.com:5432",
        "--dest",
        "./my.db",
    ];
    let matches = cli.get_matches_from_safe(args).unwrap();
    println!("{:?}", matches);
    let src = matches.value_of("SRC").unwrap_or("missing");
    let dst = matches.value_of("DEST").unwrap_or("missing");
    assert_eq!(src, "postgres://user:pw@dbhost.com:5432");
    assert_eq!(dst, "./my.db");
}
