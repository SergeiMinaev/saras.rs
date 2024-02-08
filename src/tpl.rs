pub fn tpl(mut text: String, tag: &str, is_true: bool) -> String {
    let delim = format!("{{% IF {tag} %}}");
    while text.contains(&delim) {
        let parts: Vec<&str> = text.splitn(2, &delim).collect();
        if is_true {
            text = parts.join("");
            let parts: Vec<&str> = text.splitn(2, "{% ENDIF %}").collect();
            text = parts.join("");
        } else {
            let parts2: Vec<&str> = text.splitn(2, "{% ENDIF %}").collect();
            text = parts[0].to_owned() + &parts2[1..].join("");
        }
    }
    return text
}
