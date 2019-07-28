use crate::Console;
use crate::Message;
use crate::tools;
use crate::write_data;

pub fn save_upload(o: Console, msg: Message) {

    if msg.get_filename().is_none() {
        o.error(format!("Could not get filename of received file upload."));
        return;
    } else if msg.get_filedata().is_none() {
        o.error(format!("Could not get data of received file upload."));
        return;
    }

    let fname = msg.get_filename().unwrap();
    let data = msg.get_filedata().unwrap();
    let dst = format!("/tmp/stealthy_{}_{}", tools::random_str(10), &fname);
    o.new_file(msg, fname);

    if write_data(&dst, data) {
        o.status(format!("File written to '{}'.", dst));
    } else {
        o.error(format!("Could not write data of received file upload."));
    }
}
