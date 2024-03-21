//! # hikvision-rs
//!
//! High-level asynchronous library for controlling cameras from Hikvision using the PTZ API
//! ```rust
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cam = hikvision_rs::Cam::new("127.0.0.1", Some("admin", "12345"), 500).await?;
//!     cam.zoom_cam(10).await?;
//! 
//!     Ok(())
//! }```

use reqwest::{Error, Response};
use std::fmt;
use tokio::time;


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


/// `ErrorAuthorize`, usually occurs when the login or password is incorrect or due to the lack of certain access rights to the camera
pub struct ErrorAuthorize;


impl std::error::Error for ErrorAuthorize {} 


impl fmt::Display for ErrorAuthorize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to log in and access the camera. Check that your login and password are correct, and also check on the website in the Configuration -> System -> Authentication -> Web Authentication section, the value should be set to digest/basic. Also check Configuration -> PTZ -> Enable PTZ Control, this item should be checked")
    }
}


impl fmt::Debug for ErrorAuthorize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to log in and access the camera. Check that your login and password are correct, and also check on the website in the Configuration -> System -> Authentication -> Web Authentication section, the value should be set to digest/basic. Also check Configuration -> PTZ -> Enable PTZ Control, this item should be checked")
    }
}


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


/// `QuickRequestError` usually occurs because you try to send the same action to the camera very quick
pub struct QuickRequsetError {
    timeout: usize,
    event: TypeEvent,
}


impl QuickRequsetError {
    fn new(_timeout: usize, _event: TypeEvent) -> Self {
        Self {
            timeout: _timeout + 50,
            event: _event,
        }
    }
}


impl std::error::Error for QuickRequsetError {}


impl fmt::Display for QuickRequsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "You are making request for the <{}> action too quickly. {}ms must have passed since the last request", self.event.get_str(), self.timeout)
    }
}


impl fmt::Debug for QuickRequsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "You are making request for the <{}> action too quickly. {}ms must have passed since the last request", self.event.get_str(), self.timeout)
    }
}

/// Any action is allowed only in the range -100..=100 units of measurement
pub struct OutOfRangeUnitError {
    data: isize,
    event: TypeEvent,
}


impl OutOfRangeUnitError {
    fn new(_data: isize, _event: TypeEvent) -> Self {
        Self {
            data: _data,
            event: _event,
        }
    }
}


impl std::error::Error for OutOfRangeUnitError {}


impl fmt::Display for OutOfRangeUnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The unit of measurment for the <{}> event does ot lie in the range -100..=100, its value {}", self.event.get_str(), self.data)
    }
}


impl fmt::Debug for OutOfRangeUnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The unit of measurment for the <{}> event does ot lie in the range -100..=100, its value {}", self.event.get_str(), self.data)
    }
}


impl Cam {
    /// Creating an object to connect to the camera. If there is no login and password, then the `user_passwd` field should have the value `None`
    pub async fn new<S>(addr: S, user_passwd: Option<(S, S)>, movment_speed_ms: usize) -> Result<Self, Box<dyn std::error::Error>> where S: Into<String> {
        let (addr, test_addr) = match user_passwd {
            Some((user, passwd)) => {
                let user = user.into();
                let passwd = passwd.into();
                let addr = addr.into();

                (format!("http://{}:{}@{}/ISAPI/PTZCtrl/channels/1/Momentary", user, passwd, addr),
                format!("http://{}:{}@{}/ISAPI/PTZCtrl/channels/1/capabilities", user, passwd, addr))
            },
            None => {
                let addr = addr.into();

                (format!("http://{}/ISAPI/PTZCtrl/channels/1/Momentary", addr),
                format!("http://{}/ISAPI/PTZCtrl/channels/1/capabilities", addr))
            },
        };
        let _client = reqwest::Client::new();

        let test_conn = reqwest::get(test_addr).await?.text().await?;
        return if test_conn.contains("Document Error: Unauthorized") {
            Err(Box::new(ErrorAuthorize))
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
            return Err(Box::new(OutOfRangeUnitError::new(unit, type_event)));   
        }

        let time = time::Instant::now();
        let event = match type_event {
            TypeEvent::Rotate => &mut self.pan,
            TypeEvent::Zoom => &mut self.zoom,
            TypeEvent::Tilt => &mut self.tilt,
        };

        if time.duration_since(event.last_trigger).as_millis() + 50 < time::Duration::from_millis(self.movement_speed as u64).as_millis() && !event.is_init {
            return Err(Box::new(QuickRequsetError::new(self.movement_speed, type_event)))
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


#[cfg(test)]
mod test {
    use crate::*;


    #[tokio::test]
    async fn testing() {
        let cam = Cam::new(addr, user_passwd, movment_speed_ms)
    }
}
