//! # hikvision-rs
//!
//! High-level asynchronous library for controlling cameras from Hikvision using the PTZ API
//! ```rust
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cam = hikvision_rs::Cam::new("127.0.0.1", "1208", Some("admin", "12345"), 500).await?;
//!     cam.zoom_cam(10).await?;
//! 
//!     Ok(())
//! }
//! ```

use reqwest::{Error, Response};
use tokio::time;

pub mod error;


enum TypeEvent {
    Rotate,
    Zoom,
    Tilt,
}


impl TypeEvent {
    fn get_str<'a>(&'a self) -> &'a str {
        match self {
            Self::Rotate => "rotation",
            Self::Zoom => "zoom",
            Self::Tilt => "tilt",
        }
    }
}


struct CamParam {
    data: isize,
    is_init: bool,
    last_trigger: time::Instant,
}


impl Default for CamParam {
    fn default() -> Self {
        Self {
            data: 0,
            is_init: true,
            last_trigger: time::Instant::now(),
        }
    }
}


/// The structure of the camera allows you to communicate with it at a high level
pub struct Cam {
    address: String,
    client: reqwest::Client,
    pan: CamParam,
    tilt: CamParam,
    zoom: CamParam,
    movement_speed: usize,
}


impl Cam {
    /// Creating an object to connect to the camera. If there is no login and password, then the `user_passwd` field should have the value `None`
    pub async fn new<S>(addr: S, port: S, user_passwd: Option<(S, S)>, movment_speed_ms: usize) -> Result<Self, Box<dyn std::error::Error>> where S: Into<String> {
        let (addr, test_addr) = match user_passwd {
            Some((user, passwd)) => {
                let user = user.into();
                let passwd = passwd.into();
                let addr = addr.into();
                let port = port.into();

                (format!("http://{}:{}@{}:{}/ISAPI/PTZCtrl/channels/1/Momentary", user, passwd, addr, port),
                format!("http://{}:{}@{}:{}/ISAPI/PTZCtrl/channels/1/capabilities", user, passwd, addr, port))
            },
            None => {
                let addr = addr.into();
                let port = port.into();

                (format!("http://{}:{}/ISAPI/PTZCtrl/channels/1/Momentary", addr, port),
                format!("http://{}:{}/ISAPI/PTZCtrl/channels/1/capabilities", addr, port))
            },
        };
        let _client = reqwest::Client::new();

        let test_conn = reqwest::get(test_addr).await?.text().await?;
        return if test_conn.contains("Document Error: Unauthorized") {
            Err(Box::new(error::ErrorAuthorize))
        } else {
            Ok(Self {
                address: addr,
                client: _client, 
                pan: CamParam::default(),
                tilt: CamParam::default(),
                zoom: CamParam::default(),
                movement_speed: movment_speed_ms,
            })
        }
    }

    async fn send_data(&mut self) -> Result<Response, Error> {
        self.client.put(&self.address).body(format!("<PTZData>
                <pan>{}</pan>
                <tilt>{}</tilt>
                <zoom>{}</zoom>
                <Momentary>
                    <duration>{}</duration>
                </Momentary>
            </PTZData>", self.pan.data, self.tilt.data, self.zoom.data, self.movement_speed)).send().await
    }

    async fn cam_event(&mut self, unit: isize, type_event: TypeEvent) -> Result<Response, Box<dyn std::error::Error>>{
        if unit > 100 || unit < -100 {
            return Err(Box::new(error::OutOfRangeUnitError::new(unit, type_event)));   
        }

        let time = time::Instant::now();
        let event = match type_event {
            TypeEvent::Rotate => &mut self.pan,
            TypeEvent::Zoom => &mut self.zoom,
            TypeEvent::Tilt => &mut self.tilt,
        };

        if time.duration_since(event.last_trigger).as_millis() + 50 < time::Duration::from_millis(self.movement_speed as u64).as_millis() && !event.is_init {
            return Err(Box::new(error::QuickRequsetError::new(self.movement_speed, type_event)))
        } else {
            if event.is_init {
                event.is_init = false;
            }

            event.last_trigger = time::Instant::now();
        }

        event.data += unit;

        return match self.send_data().await {
            Ok(res) => Ok(res), 
            Err(err) => Err(Box::new(err)),
        }
    }

    /// Rotate the camera, `rot` can vary -100..=100
    pub async fn rotate_cam(&mut self, rot: isize) -> Result<Response, Box<dyn std::error::Error>> {
        self.cam_event(rot, TypeEvent::Rotate).await
    }

    /// Zoom the camera lens, `zoom` can vary from -100..=100 
    pub async fn zoom_cam(&mut self, zoom: isize) -> Result<Response, Box<dyn std::error::Error>> {
        self.cam_event(zoom, TypeEvent::Zoom).await
    }

    /// Tilt the camera, `til` can vary from -100..=100 
    pub async fn tilt_cam(&mut self, til: isize) -> Result<Response, Box<dyn std::error::Error>> {
        self.cam_event(til, TypeEvent::Tilt).await
    }

    pub async fn change_movement_speed(&mut self, ms: usize) {
        self.movement_speed = ms;
    }
}
