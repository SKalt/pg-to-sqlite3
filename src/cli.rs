use clap::{App, Arg, ArgGroup};

pub fn new<'a>() -> App<'a, 'a> {
    let result = App::new("pg-to-sqlite3")
        .version("0.0.0")
        .author("Steven Kalt")
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
                .help("a path to a sqlite3 file or 'STDOUT'"),
        )
        .arg(
            Arg::with_name("overwrite")
                .long("overwrite")
                .takes_value(false)
                .help("whether to overwrite DEST if it exists"),
        )
        .arg(
            Arg::with_name("no_views")
                .long("no-views")
                .takes_value(false)
                .help("whether to omit views"),
        )
        .arg(
            Arg::with_name("progress")
                .long("progress")
                .takes_value(false)
                .help("whether to display a progress bar for data dumps"),
        )
        .arg(
            Arg::with_name("data_only")
                .long("data-only")
                .takes_value(false)
                .help("whether to only produce inserts"),
        )
        .arg(
            Arg::with_name("schema_only")
                .long("schema-only")
                .takes_value(false)
                .help("don't produce any inserts"),
        )
        .group(ArgGroup::with_name("output_type").args(&["data_only", "schema_only"]));
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
