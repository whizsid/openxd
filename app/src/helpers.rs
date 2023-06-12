use regex::Regex;

pub fn remove_symbols_and_extra_spaces(org: String) -> String {
    let symbol_rgx = Regex::new("^[a-zA-Z0-9\\s]").unwrap();
    let org_without_symbols = symbol_rgx.replace(&org, "");
    let multi_space_rgx = Regex::new("\\s+").unwrap();

    multi_space_rgx.replace(&org_without_symbols, " ").to_string()
}
