use hidapi::HidApi;

fn main() {
    let api = HidApi::new().unwrap();
    for d in api.device_list().filter(|d| d.vendor_id() == 0x1038) {
        println!(
            "pid={:#06x} iface={:>2} usage_page={:#06x} usage={:#06x} path={:?}",
            d.product_id(),
            d.interface_number(),
            d.usage_page(),
            d.usage(),
            d.path()
        );
    }
}
