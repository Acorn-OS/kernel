#![feature(vec_into_raw_parts)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod path {
    use super::*;

    pub fn create_paths() {
        assert!(
            Command::new("rm")
                .arg("-rf")
                .arg("isoroot")
                .status()
                .expect("failed to run command")
                .success(),
            "failed to remove {}",
            "isoroot"
        );
        fs::create_dir_all("isoroot").expect("failed to create isoroot directory");
        fs::create_dir_all("build").expect("failed to create build directory");
    }

    pub fn build() -> PathBuf {
        PathBuf::from("build")
            .canonicalize()
            .expect("failed to canonicalize build path")
    }

    pub fn isoroot() -> PathBuf {
        PathBuf::from("isoroot")
            .canonicalize()
            .expect("failed to canonicalize isoroot path")
    }
}

fn cargo() -> Command {
    Command::new("cargo")
}

fn tool(name: &str) -> Command {
    Command::new(format!("{}/tools/{name}", path::build().display()))
}

mod thirdparty {
    macro_rules! thirdparty_str {
        ($name:literal) => {
            concat!("thirdparty/", $name)
        };
    }
    pub mod limine {
        macro_rules! limine_str {
            ($name:literal) => {
                concat!(thirdparty_str!("limine/"), $name)
            };
        }
        pub const LIMINE_CFG: &str = limine_str!("limine.cfg");
        pub const LIMINE_SYS: &str = limine_str!("limine.sys");
    }
}

struct Builder {
    path: PathBuf,
    out_path: Option<PathBuf>,
    unstable: bool,
}

struct BuildOutput {}

impl Builder {
    fn set_from_flags(&self, cmd: &mut Command) {
        if self.unstable {
            cmd.args(["-Z", "unstable-options"]);
        }
        if let Some(out_path) = self.out_path.as_ref() {
            cmd.arg("--out-dir").arg(out_path);
        }
    }

    fn build(&self) -> BuildOutput {
        assert!(
            {
                let mut cmd = cargo();
                cmd.arg("build").current_dir(&self.path);
                self.set_from_flags(&mut cmd);
                cmd.status().expect("failed to build kernel").success()
            },
            "failed to build '{}': failed status",
            self.path.display()
        );
        BuildOutput {}
    }
}

struct Projects {
    kernel_elf: PathBuf,
    usrspc_modules: Vec<PathBuf>,
}

impl Projects {
    pub fn build() -> Self {
        let kernel_out_path = path::build();
        Builder {
            path: PathBuf::from("kernel"),
            unstable: true,
            out_path: Some(kernel_out_path.clone()),
        }
        .build();
        let tools_out_path: PathBuf = format!("{}/tools", path::build().display()).into();
        Builder {
            path: PathBuf::from("tools"),
            unstable: true,
            out_path: Some(tools_out_path),
        }
        .build();
        let usrspc_out_path: PathBuf = format!("{}/usrspc", path::build().display()).into();
        Builder {
            path: PathBuf::from("usrspc"),
            unstable: true,
            out_path: Some(usrspc_out_path),
        }
        .build();
        Self {
            kernel_elf: format!("{}/kernel", kernel_out_path.display()).into(),
            usrspc_modules: vec![path::build().join("usrspc/ps2")],
        }
    }
}

struct ISORoot;

impl ISORoot {
    const USRSPC_PATH: &str = "usrspc";
    const BOOT_PATH: &str = "boot";

    fn boot_path(path: &str) -> String {
        format!("{}/{path}", Self::BOOT_PATH)
    }

    fn create(
        kernel_elf: &Path,
        usrspc_modules: &Vec<PathBuf>,
        limine_cfg: &Path,
        limine_sys: &Path,
    ) -> Self {
        let isoroot = Self;
        isoroot.mkdir(Self::BOOT_PATH);
        isoroot.mkdir(Self::USRSPC_PATH);
        isoroot.put(Self::boot_path("acorn.elf"), kernel_elf);
        isoroot.put(Self::boot_path("limine.cfg"), limine_cfg);
        isoroot.put(Self::boot_path("limine.sys"), limine_sys);
        for module in usrspc_modules {
            isoroot.put_module(module)
        }
        isoroot
    }

    fn mkdir(&self, path: impl AsRef<str>) {
        fs::create_dir_all(path::isoroot().join(path.as_ref()))
            .expect("failed to create boot subdirectory");
    }

    fn put_module(&self, file: &Path) {
        let path = format!(
            "{}/{}",
            Self::USRSPC_PATH,
            file.file_name().expect("invalid file").to_string_lossy()
        );
        if self.exists(&path) {
            panic!("overwriting module");
        }
        self.put(&path, file);
    }

    fn put(&self, path: impl AsRef<str>, file: &Path) {
        let path = path.as_ref();
        fs::copy(
            file.clone(),
            format!("{}/{path}", path::isoroot().display()),
        )
        .expect(&format!(
            "failed to copy file {} to {}/{path}",
            file.display(),
            path::isoroot().display()
        ));
    }

    fn exists(&self, path: impl AsRef<str>) -> bool {
        let path = path.as_ref();
        PathBuf::from(format!("{}/{path}", path::isoroot().display())).exists()
    }

    fn path(&self) -> PathBuf {
        path::isoroot()
    }
}

pub struct Initrd(PathBuf);

impl Initrd {
    pub fn create(files: Vec<impl Into<PathBuf>>, out_file: PathBuf) -> Initrd {
        assert!(
            tool("initrd")
                .arg("--output")
                .arg(&out_file)
                .args(files.into_iter().map(|e| Into::<PathBuf>::into(e)))
                .status()
                .expect("failed to start initrd tool")
                .success(),
            "initrd tool ran unsuccessfully"
        );
        Self(out_file)
    }

    pub fn output(&self) -> &Path {
        &self.0
    }
}

struct Image(PathBuf);

impl Image {
    pub fn create(iso_root: ISORoot) -> Self {
        let image = PathBuf::from(format!("{}/image", path::build().display()));
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
                .arg("-d")
                .arg(iso_root.path())
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
        Self(image)
    }

    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

fn run_qemu(image: Image) {
    let mut qemu_command = Command::new("qemu-system-x86_64");
    qemu_command
        .arg("-drive")
        .arg(format!("format=raw,file={}", image.path().display()))
        .arg("-s")
        .arg("-S")
        .arg("--no-reboot")
        .args(["-m", "4G"])
        .args(["-monitor", "stdio"])
        .args(["-d", "int"])
        //.args(["-d", "int,unimp,mmu,cpu_reset,guest_errors,page"])
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

#[test]
fn test() {
    fn cargo_test(dir: &str) {
        let cmd = cargo()
            .arg("test")
            .current_dir(dir)
            .status()
            .expect(&format!("failed to test '{dir}'"));
        assert!(cmd.success(), "failed test '{dir}'",);
    }
    cargo_test("tools");
    cargo_test("dep");
}

fn main() {
    path::create_paths();
    let projects = Projects::build();
    let iso_root = ISORoot::create(
        &projects.kernel_elf,
        &projects.usrspc_modules,
        &PathBuf::from(thirdparty::limine::LIMINE_CFG),
        &PathBuf::from(thirdparty::limine::LIMINE_SYS),
    );
    let initrd = Initrd::create(
        vec!["build/usrspc/ps2"],
        format!("{}/initrd", path::build().display()).into(),
    );
    iso_root.put_module(initrd.output());
    let image = Image::create(iso_root);
    run_qemu(image);
}
