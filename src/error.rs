/// `ErrorAuthorize`, usually occurs when the login or password is incorrect or due to the lack of certain access rights to the camera
use std::fmt;
use crate::TypeEvent;


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


/// `QuickRequestError` usually occurs because you try to send the same action to the camera very quick
pub struct QuickRequsetError {
    timeout: usize,
    event: TypeEvent,
}


impl QuickRequsetError {
    pub(crate) fn new(_timeout: usize, _event: TypeEvent) -> Self {
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
    data: i8,
    event: TypeEvent,
}


impl OutOfRangeUnitError {
    pub(crate) fn new(_data: i8, _event: TypeEvent) -> Self {
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
