# HIKVISION-RS
 High-level asynchronous library for controlling cameras from `Hikvision` using the PTZ API
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
     let mut cam = hikvision::Cam::new("127.0.0.1", 1208, Some(("admin", "12345")), 500).await?;
     cam.zoom_cam(10).await?;

     Ok(())
}
```
If the camera does not respond to your commands, then try the following:

In the camera settings, set the Configuration -> PTZ -> "Basic Settings" -> "Manual Control Speed" parameter to "Compatible".
