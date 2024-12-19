use std::env;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::code_generator;
use crate::Error;
use crate::RustEdition;
use crate::TAConfig;

const DEFAULT_HEADER_FILE_NAME: &str = "user_ta_header.rs";

pub struct Config {
    out_dir: Option<PathBuf>,
    edition: RustEdition,
    header_file_name: Option<String>,
    ta_config: TAConfig,
}

impl Config {
    pub fn new(edition: RustEdition, ta_config: TAConfig) -> Self {
        Self {
            out_dir: Option::None,
            header_file_name: Option::None,
            edition,
            ta_config,
        }
    }
    pub fn out_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.out_dir = Option::Some(path.into());
        self
    }
    pub fn header_file_name<S: Into<String>>(mut self, file_name: S) -> Self {
        self.header_file_name = Option::Some(file_name.into());
        self
    }
    pub fn build(self, uuid: &str) -> Result<(), Error> {
        let out = match self.out_dir.clone() {
            Some(v) => v,
            None => PathBuf::from(std::env::var("OUT_DIR")?),
        };
        self.write_header_file(out.clone(), uuid)?;
        self.link(out)?;
        Ok(())
    }
}

impl Config {
    fn write_header_file(&self, out: PathBuf, uuid: &str) -> Result<(), Error> {
        let out_header_file_name = out.join(match self.header_file_name.as_ref() {
            Some(v) => v.as_str(),
            None => DEFAULT_HEADER_FILE_NAME,
        });
        let mut buffer = File::create(out_header_file_name.clone())?;
        let header_codes = code_generator::generate(self.edition.clone(), &self.ta_config, uuid)?;
        buffer.write_all(header_codes.as_bytes())?;
        Ok(())
    }

    fn write_and_link_ta_lds(&self, out: PathBuf, ta_dev_kit_dir: PathBuf) -> Result<(), Error> {
        const ENV_TARGET_TA: &str = "TARGET_TA";
        println!("cargo:rerun-if-env-changed={}", ENV_TARGET_TA);
        let mut aarch64_flag = true;
        match env::var(ENV_TARGET_TA) {
            Ok(ref v) if v == "arm-unknown-linux-gnueabihf" || v == "arm-unknown-optee" => {
                println!("cargo:rustc-link-arg=--no-warn-mismatch");
                aarch64_flag = false;
            }
            _ => {}
        };

        let f = BufReader::new(File::open(ta_dev_kit_dir.join("src/ta.ld.S"))?);
        let ta_lds_file_path = out.join("ta.lds");
        let mut ta_lds = File::create(ta_lds_file_path.clone())?;
        for line in f.lines() {
            let l = line?;

            if aarch64_flag {
                if l.starts_with('#')
                    || l == "OUTPUT_FORMAT(\"elf32-littlearm\")"
                    || l == "OUTPUT_ARCH(arm)"
                {
                    continue;
                }
            } else {
                if l.starts_with('#')
                    || l == "OUTPUT_FORMAT(\"elf64-littleaarch64\")"
                    || l == "OUTPUT_ARCH(aarch64)"
                {
                    continue;
                }
            }

            if l == "\t. = ALIGN(4096);" {
                write!(ta_lds, "\t. = ALIGN(65536);\n")?;
            } else {
                write!(ta_lds, "{}\n", l)?;
            }
        }

        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rerun-if-changed={}", ta_lds_file_path.display());
        println!("cargo:rustc-link-arg=-T{}", ta_lds_file_path.display());
        Ok(())
    }

    fn link(&self, out: PathBuf) -> Result<(), Error> {
        const ENV_TA_DEV_KIT_DIR: &str = "TA_DEV_KIT_DIR";
        println!("cargo:rerun-if-env-changed={}", ENV_TA_DEV_KIT_DIR);
        let ta_dev_kit_dir = PathBuf::from(std::env::var(ENV_TA_DEV_KIT_DIR)?);

        self.write_and_link_ta_lds(out.clone(), ta_dev_kit_dir.clone())?;

        let search_path = ta_dev_kit_dir.join("lib");
        println!("cargo:rustc-link-search={}", search_path.display());
        println!("cargo:rustc-link-lib=static=utee");
        println!("cargo:rustc-link-lib=static=utils");
        println!("cargo:rustc-link-arg=-e__ta_entry");
        println!("cargo:rustc-link-arg=-pie");
        println!("cargo:rustc-link-arg=-Os");
        println!("cargo:rustc-link-arg=-Wl,--sort-section=alignment");

        let mut dyn_list = File::create(out.join("dyn_list"))?;
        write!(
            dyn_list,
            "{{ __elf_phdr_info; trace_ext_prefix; trace_level; ta_head; }};\n"
        )?;
        println!("cargo:rustc-link-arg=-Wl,--dynamic-list=dyn_list");

        Ok(())
    }
}
