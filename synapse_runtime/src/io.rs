#[no_mangle]
pub extern "C" fn synapse_print_int(value: i64) {
    println!("{}", value);
}