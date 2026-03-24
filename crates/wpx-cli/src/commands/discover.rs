use wpx_api::WpClient;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

pub async fn handle(client: &WpClient) -> Result<RenderPayload, WpxError> {
    let data = client.discover().await;
    Ok(RenderPayload {
        data,
        summary: Some("Discovery complete".into()),
    })
}
