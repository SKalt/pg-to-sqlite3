pub fn connect(connection_string: &str) -> postgres::Client {
    match postgres::Client::connect(connection_string, postgres::NoTls) {
        Ok(conn) => return conn,
        Err(e) => panic!("{:?}", e),
    }
}

pub fn must_succeed(response: Result<Vec<postgres::Row>, postgres::Error>) -> Vec<postgres::Row> {
    match response {
        Ok(rows) => return rows,
        Err(e) => panic!("{:?}", e),
    }
}
