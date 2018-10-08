extern crate bindgen;
extern crate cc;
extern crate make_cmd;

use bindgen::EnumVariation;
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn compiler_prefix() -> String {
    let cfg = cc::Build::new();
    let compiler = cfg.get_compiler();
    String::from(compiler.path().to_str().unwrap())
        .replace("-gcc", "")
}

fn real_env() -> String {
    let target = env::var("TARGET").unwrap();
    let s: Vec<&str> = target.split_terminator("-").collect();
    s.get(s.len() - 1).unwrap().to_string()
}

fn link_triple() -> String {
    format!("-{}-{}-{}-{}",
            env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
            env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or("unknown".to_string()),
            env::var("CARGO_CFG_TARGET_OS").unwrap(),
            //env::var("CARGO_CFG_TARGET_ENV").unwrap()
            real_env()
    )
}

fn link_libs() {
    let s = link_triple();
    let t = "static=";

    println!("cargo:rustc-link-lib=asound");

    println!("cargo:rustc-link-search=openssl/");
    println!("cargo:rustc-link-lib={}ssl", t);
    println!("cargo:rustc-link-lib={}crypto", t);

    println!("cargo:rustc-link-search=pjproject/third_party/lib/");
    println!("cargo:rustc-link-lib={}gsmcodec{}", t, s);
    println!("cargo:rustc-link-lib={}ilbccodec{}", t, s);
    println!("cargo:rustc-link-lib={}speex{}", t, s);
    println!("cargo:rustc-link-lib={}g7221codec{}", t, s);
    println!("cargo:rustc-link-lib={}resample{}", t, s);
    println!("cargo:rustc-link-lib={}srtp{}", t, s);
    //println!("cargo:rustc-link-lib={}yuv{}", t, s);
    //println!("cargo:rustc-link-lib={}webrtc{}", t, s);

    println!("cargo:rustc-link-search=pjproject/pjlib/lib/");
    println!("cargo:rustc-link-lib={}pj{}", t, s);

    println!("cargo:rustc-link-search=pjproject/pjlib-util/lib/");
    println!("cargo:rustc-link-lib={}pjlib-util{}", t, s);

    println!("cargo:rustc-link-search=pjproject/pjnath/lib/");
    println!("cargo:rustc-link-lib={}pjnath{}", t, s);

    println!("cargo:rustc-link-search=pjproject/pjmedia/lib/");
    println!("cargo:rustc-link-lib={}pjmedia{}", t, s);
    println!("cargo:rustc-link-lib={}pjmedia-codec{}", t, s);
    //println!("cargo:rustc-link-lib={}pjmedia-videodev{}", t, s);
    println!("cargo:rustc-link-lib={}pjmedia-audiodev{}", t, s);
    println!("cargo:rustc-link-lib={}pjsdp{}", t, s);

    println!("cargo:rustc-link-search=pjproject/pjsip/lib/");
    println!("cargo:rustc-link-lib={}pjsip{}", t, s);
    println!("cargo:rustc-link-lib={}pjsip-simple{}", t, s);
    println!("cargo:rustc-link-lib={}pjsip-ua{}", t, s);
    println!("cargo:rustc-link-lib={}pjsua{}", t, s);
}


fn check_path(p: String) -> Option<()> {
    if Path::new(&p).exists() {
        Some(())
    } else {
        None
    }
}

fn check_ssl_built_status() -> Option<()> {
    check_path("openssl/libssl.a".to_string())?;
    check_path("openssl/libcrypto.a".to_string())
}

fn check_pj_built_status() -> Option<()> {
    let lt = format!("{}.a", link_triple());

    check_path(format!("pjproject/third_party/lib/libg7221codec{}", lt))?;
    check_path(format!("pjproject/third_party/lib/libgsmcodec{}", lt))?;
    check_path(format!("pjproject/third_party/lib/libilbccodec{}", lt))?;
    check_path(format!("pjproject/third_party/lib/libresample{}", lt))?;
    check_path(format!("pjproject/third_party/lib/libspeex{}", lt))?;
    check_path(format!("pjproject/third_party/lib/libsrtp{}", lt))?;
    //check_path(format!("pjproject/third_party/lib/libwebrtc{}", lt))?;
    //check_path(format!("pjproject/third_party/lib/libyuv{}", lt))?;

    check_path(format!("pjproject/pjlib/lib/libpj{}", lt))?;

    check_path(format!("pjproject/pjlib-util/lib/libpjlib-util{}", lt))?;

    check_path(format!("pjproject/pjnath/lib/libpjnath{}", lt))?;

    check_path(format!("pjproject/pjmedia/lib/libpjmedia{}", lt))?;
    check_path(format!("pjproject/pjmedia/lib/libpjmedia-codec{}", lt))?;
    //check_path(format!("pjproject/pjmedia/lib/libpjmedia-videodev{}", lt))?;
    check_path(format!("pjproject/pjmedia/lib/libpjmedia-audiodev{}", lt))?;
    check_path(format!("pjproject/pjmedia/lib/libpjsdp{}", lt))?;

    check_path(format!("pjproject/pjsip/lib/libpjsip{}", lt))?;
    check_path(format!("pjproject/pjsip/lib/libpjsip-simple{}", lt))?;
    check_path(format!("pjproject/pjsip/lib/libpjsip-ua{}", lt))?;
    check_path(format!("pjproject/pjsip/lib/libpjsua{}", lt))
}

fn compile_ssl() {
    make_cmd::make()
        .arg("clean")
        .current_dir("./openssl/")
        .spawn().unwrap().wait().unwrap();

    let mut c = Command::new("./Configure");

    if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
        c.arg(format!("--cross-compile-prefix={}-", compiler_prefix()));
    } else {
        c.arg("--cross-compile-prefix=");
    }

    c.arg("linux-generic32");

    c.current_dir("./openssl/");
    c.spawn().unwrap().wait().unwrap();


    make_cmd::make()
        .current_dir("./openssl/")
        .spawn().unwrap().wait().unwrap();
}

fn compile_pj() {
    create_config();

    let mut c = Command::new("sh");

    c.arg("aconfigure");

    if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
        let target = compiler_prefix();

        c.arg(format!("--target={}", target));
        c.arg(format!("--host={}", target));
    }

    c.arg("--disable-video");
    c.arg("--disable-libwebrtc");

    c.current_dir("./pjproject/");
    c.spawn().unwrap().wait().unwrap();

    make_cmd::make()
        .arg("dep")
        .current_dir("./pjproject/")
        .spawn().unwrap().wait().unwrap();

    make_cmd::make()
        .current_dir("./pjproject/")
        .spawn().unwrap().wait().unwrap();
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        .header("pjproject/pjlib/include/pj/config_site.h")

        .header("pjproject/pjsip/include/pjsua-lib/pjsua.h")
        .header("pjproject/pjsip/include/pjsua-lib/pjsua_internal.h")
        .header("pjproject/pjsip/include/pjsip.h")
        .header("pjproject/pjsip/include/pjsip_ua.h")

        .header("pjproject/pjnath/include/pjnath.h")

        .header("pjproject/pjmedia/include/pjmedia.h")
        .header("pjproject/pjmedia/include/pjmedia_audiodev.h")
        //.header("pjproject/pjmedia/include/pjmedia_videodev.h")
        .header("pjproject/pjmedia/include/pjmedia-codec.h")

        .header("pjproject/pjlib-util/include/pjlib-util.h")

        .header("pjproject/pjlib/include/pjlib.h")

        .default_enum_style(EnumVariation::Consts)

        .generate_comments(true)

        .layout_tests(false)
        .whitelist_type(r"pj.*")
        .whitelist_type(r"PJ.*")
        .whitelist_var(r"pj.*")
        .whitelist_var(r"PJ.*")
        .whitelist_function(r"pj.*")
        .whitelist_function(r"PJ.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn create_config() {
    let mut config_h = OpenOptions::new()
        .write(true)
        .create(true)
        .open("pjproject/pjlib/include/pj/config_site.h")
        .unwrap();

    config_h.write(match env::var("CARGO_CFG_ENDIAN")
        .unwrap_or("little".to_string()).as_str() {
        "little" => r#"
    #define PJ_IS_LITTLE_ENDIAN 1
    #define PJ_IS_BIG_ENDIAN 0
    "#,

        "big" => r#"
    #define PJ_IS_LITTLE_ENDIAN 0
    #define PJ_IS_BIG_ENDIAN 1
    "#,

        _ => unreachable!()
    }.as_bytes()).unwrap();
}

pub fn main() {
    let mut target = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("target")
        .unwrap();

    let mut last_target_str = String::new();
    let target_str = env::var("TARGET").unwrap();
    target.read_to_string(&mut last_target_str).expect("Failed to read target!");

    if last_target_str != target_str || check_ssl_built_status().is_none() {
        compile_ssl();
    }

    if check_pj_built_status().is_none() {
        compile_pj();
    }

    target.seek(SeekFrom::Start(0)).unwrap();
    target.set_len(0).unwrap();

    target.write(target_str.as_bytes()).expect("Failed to write target!");


    if !Path::new(&format!("{}/bindings.rs", env::var("OUT_DIR").unwrap())).exists() {
        generate_bindings();
    }

    link_libs();
}