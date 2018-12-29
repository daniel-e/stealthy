// http://patorjk.com/software/taag/#p=display&f=ANSI%20Shadow&t=stealthy
pub fn get_logo() -> Vec<String> {

    let mut s = Vec::new();
    s.push(String::from("███████╗████████╗███████╗ █████╗ ██╗  ████████╗██╗  ██╗██╗   ██╗"));
    s.push(String::from("██╔════╝╚══██╔══╝██╔════╝██╔══██╗██║  ╚══██╔══╝██║  ██║╚██╗ ██╔╝"));
    s.push(String::from("███████╗   ██║   █████╗  ███████║██║     ██║   ███████║ ╚████╔╝"));
    s.push(String::from("╚════██║   ██║   ██╔══╝  ██╔══██║██║     ██║   ██╔══██║  ╚██╔╝"));
    s.push(String::from("███████║   ██║   ███████╗██║  ██║███████╗██║   ██║  ██║   ██║"));
    s.push(String::from("╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚══════╝╚═╝   ╚═╝  ╚═╝   ╚═╝ v0.0.2"));
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
