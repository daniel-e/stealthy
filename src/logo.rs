// font: doom
// http://www.network-science.de/ascii/
pub fn get_logo() -> Vec<String> {

    let mut s = Vec::new();
    s.push(String::from("     _             _ _   _           "));
    s.push(String::from("    | |           | | | | |          "));
    s.push(String::from(" ___| |_ ___  __ _| | |_| |__  _   _ "));
    s.push(String::from("/ __| __/ _ \\/ _` | | __| '_ \\| | | |"));
    s.push(String::from("\\__ \\ ||  __/ (_| | | |_| | | | |_| |"));
    s.push(String::from("|___/\\__\\___|\\__,_|_|\\__|_| |_|\\__, |"));
    s.push(String::from("                                __/ |"));
    s.push(String::from("                               |___/ "));
    s
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::get_logo;

    // Just a test for test coverage.
    #[test]
    fn test_get_logo() {
        assert!(get_logo().len() > 5);
    }
}
