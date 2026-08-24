use hidapi::HidApi;
use tray_icon::Icon;

const STEELSERIES_VID: u16 = 0x1038;

const RIVAL3_WIRELESS_PID: u16 = 0x1830;
const RIVAL3_WIRELESS_INTERFACE: i32 = 3;

const ARCTIS_NOVA_PRO_WIRELESS_PID: u16 = 0x12e0;
const ARCTIS_NOVA_PRO_WIRELESS_INTERFACE: i32 = 4;

#[derive(Clone, Debug, Default)]
pub struct DeviceStatus {
    pub level: Option<u8>,
    pub charging: Option<bool>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub mouse: DeviceStatus,
    pub headset: DeviceStatus,
}

pub fn poll() -> Snapshot {
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            let err = DeviceStatus {
                level: None,
                charging: None,
                error: Some(e.to_string()),
            };
            return Snapshot {
                mouse: err.clone(),
                headset: err,
            };
        }
    };

    Snapshot {
        mouse: read_mouse_battery(&api).into(),
        headset: read_headset_battery(&api).into(),
    }
}

impl From<Result<(u8, bool), String>> for DeviceStatus {
    fn from(result: Result<(u8, bool), String>) -> Self {
        match result {
            Ok((level, charging)) => DeviceStatus {
                level: Some(level),
                charging: Some(charging),
                error: None,
            },
            Err(e) => DeviceStatus {
                level: None,
                charging: None,
                error: Some(e),
            },
        }
    }
}

fn read_mouse_battery(api: &HidApi) -> Result<(u8, bool), String> {
    let device_info = api
        .device_list()
        .find(|d| {
            d.vendor_id() == STEELSERIES_VID
                && d.product_id() == RIVAL3_WIRELESS_PID
                && d.interface_number() == RIVAL3_WIRELESS_INTERFACE
        })
        .ok_or("dongle not found")?;

    let device = device_info.open_device(api).map_err(|e| e.to_string())?;
    device
        .write(&[0x00, 0xAA, 0x01])
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 32];
    let n = device
        .read_timeout(&mut buf, 200)
        .map_err(|e| e.to_string())?;
    if n < 3 {
        return Err("no response".into());
    }

    let level = buf[0];
    let charging = buf[2] != 0;
    Ok((level, charging))
}

fn read_headset_battery(api: &HidApi) -> Result<(u8, bool), String> {
    for device_info in api.device_list().filter(|d| {
        d.vendor_id() == STEELSERIES_VID
            && d.product_id() == ARCTIS_NOVA_PRO_WIRELESS_PID
            && d.interface_number() == ARCTIS_NOVA_PRO_WIRELESS_INTERFACE
    }) {
        let Ok(device) = device_info.open_device(api) else {
            continue;
        };
        if device.write(&[0x06, 0xb0]).is_err() {
            continue;
        }

        let mut buf = [0u8; 64];
        let Ok(n) = device.read_timeout(&mut buf, 200) else {
            continue;
        };
        if n <= 15 {
            continue;
        }

        let charge_byte = buf[15];
        let charging = match charge_byte {
            1 => return Err("headset not connected to base station".into()),
            2 => true,
            8 => false,
            _ => continue,
        };

        let percent = calculate_battery(buf[6], 0, 8);
        return Ok((percent, charging));
    }

    Err("base station not found or did not respond".into())
}

fn calculate_battery(raw: u8, min: u8, max: u8) -> u8 {
    if raw > max {
        return 100;
    }
    (((raw - min) as u32 * 100) / (max - min) as u32) as u8
}

pub fn tooltip_text(snap: &Snapshot) -> String {
    format!(
        "Mouse: {}\nHeadset: {}",
        device_line(&snap.mouse),
        device_line(&snap.headset)
    )
}

fn device_line(status: &DeviceStatus) -> String {
    match status.level {
        Some(level) => {
            let charging = matches!(status.charging, Some(true));
            format!("{level}%{}", if charging { " (charging)" } else { "" })
        }
        None => status
            .error
            .clone()
            .unwrap_or_else(|| "unavailable".to_string()),
    }
}

const ICON_SIZE: u32 = 32;
const MARGIN: u32 = 2;
const GAP: u32 = 6;
const BAR_WIDTH: u32 = (ICON_SIZE - 2 * MARGIN - GAP) / 2;

const BACKGROUND: [u8; 3] = [24, 24, 27];
const TRACK: [u8; 3] = [75, 75, 80];

pub fn make_icon(snap: &Snapshot) -> Icon {
    let mut rgba = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    fill(&mut rgba, BACKGROUND);
    draw_bar(&mut rgba, MARGIN, BAR_WIDTH, &snap.mouse);
    draw_bar(&mut rgba, MARGIN + BAR_WIDTH + GAP, BAR_WIDTH, &snap.headset);
    Icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).expect("icon buffer matches declared size")
}

fn level_color(level: u8) -> [u8; 3] {
    if level <= 15 {
        [220, 60, 60]
    } else if level <= 35 {
        [230, 180, 40]
    } else {
        [60, 190, 90]
    }
}

fn fill(rgba: &mut [u8], color: [u8; 3]) {
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];
        pixel[3] = 255;
    }
}

fn draw_bar(rgba: &mut [u8], x0: u32, width: u32, status: &DeviceStatus) {
    let (color, level) = match status.level {
        Some(l) => (level_color(l), l),
        None => ([100, 100, 100], 0),
    };
    let filled_rows = if status.level.is_none() {
        0
    } else {
        ((level as u32 * ICON_SIZE) / 100).max(2)
    };

    for y in 0..ICON_SIZE {
        let filled = y >= ICON_SIZE - filled_rows;
        let pixel = if filled { color } else { TRACK };
        for x in x0..x0 + width {
            let idx = ((y * ICON_SIZE + x) * 4) as usize;
            rgba[idx] = pixel[0];
            rgba[idx + 1] = pixel[1];
            rgba[idx + 2] = pixel[2];
            rgba[idx + 3] = 255;
        }
    }
}
