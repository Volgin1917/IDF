// JWT auth — will be wired in a later phase.
// Use dev-token in Authorization header for now.

#[allow(dead_code)]
pub async fn require_admin() -> Result<(), std::convert::Infallible> {
    Ok(())
}
