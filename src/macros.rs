macro_rules! continue_on_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}

macro_rules! continue_on_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => continue,
        }
    };
}
