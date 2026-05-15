// JWT auth — will be wired in a later phase.
// Use dev-token in Authorization header for now.

pub async fn require_admin() -> Result<(), std::convert::Infallible> {
    Ok(())
}
