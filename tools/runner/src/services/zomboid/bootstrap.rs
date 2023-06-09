use tokio::fs;

const PATCHED_START_SCRIPT: &str = include_str!("./patched-start-server.sh");

pub async fn patch_start_script(script_path: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(script_path).await?;
    if !content.contains("exec") {
        fs::write(script_path, PATCHED_START_SCRIPT).await?;
    }
    Ok(())
}
