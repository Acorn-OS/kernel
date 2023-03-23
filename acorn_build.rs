use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const BUILD_DIR: &str = "build";

fn cargo() -> Command {
    Command::new("cargo")
}

struct Builder {
    path: PathBuf,
    build_path: PathBuf,
}

struct BuildOutput {}

impl Builder {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            build_path: PathBuf::from(BUILD_DIR)
                .canonicalize()
                .expect("missing build directory"),
        }
    }

    fn build(&self) -> BuildOutput {
        macro_rules! cmd {
            () => {{
                let mut cmd = cargo();
                cmd.arg("build").current_dir(&self.path);
                cmd
            }};
        }

        assert!(
            cmd!()
                .args(["-Z", "unstable-options"])
                .arg("--out-dir")
                .arg(&self.build_path)
                .status()
                .expect("failed to build kernel")
                .success(),
            "failed to build kernel: failed status"
        );
        BuildOutput {}
    }
}

fn main() {
    fs::create_dir_all("isoroot").expect("failed to create isoroot directory");
    fs::create_dir_all("build").expect("failed to create build directory");
    let kernel_builder = Builder::new(PathBuf::from("kernel"));
    kernel_builder.build();
    fs::create_dir_all("isoroot/boot").expect("failed");
    assert!(
        Command::new("rm")
            .arg("-rf")
            .arg(format!("{BUILD_DIR}/*"))
            .status()
            .expect("failed to run command")
            .success(),
        "failed to clean {BUILD_DIR}"
    );
    assert!(
        Command::new("rm")
            .arg("-rf")
            .arg(format!("isoroot/*"))
            .status()
            .expect("failed to run command")
            .success(),
        "failed to clean {BUILD_DIR}"
    );
    fs::copy("build/kernel", "isoroot/boot/acorn.elf").expect("failed to copy kernel");
    fs::copy("limine.cfg", "isoroot/boot/limine.cfg").expect("failed to copy limine.cfg");
    fs::copy("thirdparty/limine/limine.sys", "isoroot/boot/limine.sys")
        .expect("failed to copy limine.sys");
    let image = PathBuf::from("build/image");
    assert!(
        Command::new("fallocate")
            .args(["-l", "1G"])
            .arg(&image)
            .status()
            .expect("failed to run command")
            .success(),
        "failed to fallocate {}",
        image.display()
    );
    assert!(
        Command::new("parted")
            .arg("-s")
            .arg(&image)
            .args(["mklabel", "msdos", "mkpart", "primary", "ext2", "1", "100%"])
            .status()
            .expect("failed to run command")
            .success(),
        "failed call to parted"
    );
    assert!(
        Command::new("mkfs.ext2")
            .arg(&image)
            .args(["-d", "isoroot"])
            .args(["-E", "offset=1048576"])
            .status()
            .expect("failed to run command")
            .success(),
        "failed to format partition"
    );
    assert!(
        Command::new("thirdparty/limine/limine-deploy")
            .arg(&image)
            .arg("--quiet")
            .status()
            .expect("failed to run command")
            .success(),
        "failed to deploy limine"
    );
    let mut qemu_command = Command::new("qemu-system-x86_64");
    qemu_command
        .arg("-drive")
        .arg(format!("format=raw,file={}", image.display()))
        .arg("-s")
        .arg("-S")
        .arg("--no-reboot")
        .args(["-m", "4G"])
        .args(["-monitor", "stdio"])
        .args(["-d", "int,unimp,mmu,cpu_reset,guest_errors,page"])
        .args(["-D", "qemu.log"])
        .args(["-serial", "file:qemu.serial.log"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    println!("running qemu: {qemu_command:?}",);
    assert!(
        qemu_command
            .status()
            .expect("failed to run command")
            .success(),
        "failed to run qemu"
    );
}
