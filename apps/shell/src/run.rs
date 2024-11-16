use alloc::vec;
use raca_std::{
    fs::{File, OpenMode}, path::Path, task::Process
};

pub fn try_run(path: Path) -> Option<()> {
    if let Ok(mut file) = File::open(path, OpenMode::Read) {

        let mut buf = vec![0; file.size() as usize];
        file.read(&mut buf);
        file.close();

        let process = Process::new(buf.leak(), "temp", 0, 0);
        process.run();
        //loop {
        //    let mut buf = [0;1];
        //    pipe2_read.read(&mut buf);
        //    write!(fd, "{}", buf[0] as char).unwrap();
        //}
        //loop{}
        //let code = wait();
        //println!("exit code: {}", code);
        //loop{

        //}
        Some(())
    } else {
        None
    }
}
