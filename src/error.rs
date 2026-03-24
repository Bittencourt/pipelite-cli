// Error types -- fully implemented in Task 2
#[allow(dead_code)]
pub fn display_error(err: &anyhow::Error, color: bool) {
    let _ = color;
    eprintln!("error: {err:#}");
}

#[allow(dead_code)]
pub fn exit_code(_err: &anyhow::Error) -> i32 {
    1
}
