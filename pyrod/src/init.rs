use std::process::Command;
///We are the init system, and with great power comes great responsibility
///Allow panics here because if this fucked up then you really are fucked
///info on mounting proc/sys taken from http://git.2f30.org/fs/file/bin/rc.init.html
pub fn init() {
    //this doesn't need to be mounted in tmpfs because we're already in RAM
    std::fs::create_dir("/tmp").expect("Could not create /tmp");

    //mount proc
    Command::new("/bin/mount")
        .args(vec![
            "-t",
            "proc",
            "-o",
            "nosuid,noexec,nodev",
            "proc",
            "/proc",
        ])
        .output()
        .expect("Could not mount /proc");
}
