pub fn display_error(e: impl std::error::Error) -> String {
    let mut res = e.to_string();

    // God, forgive me.

    if let Some(source) = e.source() {
        res = format!("{}\nCaused by:\n\t{}", res, source);

        let mut temp = source;
        while let Some(source) = temp.source() {
            res = format!("{}\nCaused by:\n\t{}", res, source);
            temp = source;
        }
    }

    res
}
