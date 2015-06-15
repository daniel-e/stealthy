// font: doom
// http://www.network-science.de/ascii/
pub fn get_logo() -> String {

    let mut s = String::new();
    s.push_str("     _             _ _   _           \n");
    s.push_str("    | |           | | | | |          \n");
    s.push_str(" ___| |_ ___  __ _| | |_| |__  _   _ \n");
    s.push_str("/ __| __/ _ \\/ _` | | __| '_ \\| | | |\n");
    s.push_str("\\__ \\ ||  __/ (_| | | |_| | | | |_| |\n");
    s.push_str("|___/\\__\\___|\\__,_|_|\\__|_| |_|\\__, |\n");
    s.push_str("                                __/ |\n");
    s.push_str("                               |___/ \n");
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
        assert!(get_logo().len() > 20);
    }
}
