use std::process::Command;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct CurlResponse {
    pub status: u16,
    pub body: String,
}

pub fn request(
    url: &str,
    method: &str,
    headers: &[(&str, &str)],
    data: Option<&str>,
) -> Result<CurlResponse, String> {
    let mut args = vec![
        "-sS".to_string(),
        "-L".to_string(),
        "--compressed".to_string(),
        "-w".to_string(),
        "__STATUS__%{http_code}".to_string(),
        "-A".to_string(),
        USER_AGENT.to_string(),
        "-X".to_string(),
        method.to_uppercase(),
    ];

    let mut has_content_type = false;
    for (key, value) in headers {
        if key.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        args.push("-H".to_string());
        args.push(format!("{key}: {value}"));
    }

    if let Some(body) = data {
        if !has_content_type {
            args.push("-H".to_string());
            args.push("Content-Type: application/x-www-form-urlencoded".to_string());
        }
        args.push("--data-raw".to_string());
        args.push(body.to_string());
    }

    args.push(url.to_string());

    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let mut command = Command::new("curl.exe");
    #[cfg(windows)]
    command.creation_flags(0x08000000);

    let output = command
        .args(&args)
        .output()
        .map_err(|e| format!("curl not found: {e}"))?;

    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl exited: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker = stdout
        .rfind("__STATUS__")
        .ok_or("curl response missing status marker")?;
    let body = stdout[..marker].to_string();
    let status = stdout[marker + "__STATUS__".len()..]
        .trim()
        .parse::<u16>()
        .map_err(|e| format!("invalid curl status: {e}"))?;

    Ok(CurlResponse { status, body })
}
