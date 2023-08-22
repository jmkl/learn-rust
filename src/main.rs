use clap::{arg, Command};
use image::imageops::FilterType;
use image::{GenericImageView, ImageBuffer, Rgba};
use reqwest;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tungstenite::connect;
use url::Url;
use uuid::Uuid;

static LCLHST: &str = "127.0.0.1:8188";
static HELP: &str = r#"

▄▄▄  ▄• ▄▌.▄▄ · ▄▄▄▄▄
▀▄ █·█▪██▌▐█ ▀. •██         some tools for :
▐▀▀▄ █▌▐█▌▄▀▀▀█▄ ▐█.▪       ..........ComfyUI API
▐█•█▌▐█▄█▌▐█▄▪▐█ ▐█▌·       ..........Image Resize
.▀  ▀ ▀▀▀  ▀▀▀▀  ▀▀▀

USAGE:
  file  inject ComfyUI workflow.json file using websocket
  img   image processing stuff...
                -i --inputdir       image's input directory
                -o --outputdir      image's output directory
                -s --size           image width
"#;

fn listen_to(_id: Uuid) {
    let (mut socket, _response) =
        connect(Url::parse(format!("ws://{}/ws?clientId={}", LCLHST, &_id).as_str()).unwrap())
            .expect("Can't connect");
    loop {
        let msg = socket.read().unwrap();
        if msg.is_text() {
            let val: Value = serde_json::from_str(&msg.to_string()).expect("Error parsing");
            let _type = &val["type"].to_string();
            let _data = &val["data"];
            let msg_type = _type.as_str().replace("\"", "");

            match msg_type.as_str() {
                "progress" => {
                    println!("progress :{:?}", _data["value"]);
                }
                "executing" => {
                    let node = _data["node"].is_null();
                    println!("executing :{:?}", node);
                }

                "executed" => {
                    println!("executed :{:?}", _data["output"]["images"][0]["filename"]);
                }
                "status" => {
                    let queue = &_data["status"]["exec_info"]["queue_remaining"];
                    if queue.as_i64().unwrap() == 0_i64 {
                        break;
                    }
                }
                _ => println!("\n"),
            }
        }
    }
}

fn parse_json_file(file_path: &Path) {
    let _id = Uuid::new_v4();
    let _contents = fs::read_to_string(&file_path).expect("Error Reading");
    let v: Value = serde_json::from_str(&_contents).expect("Error parsing");
    let params = json!({
        "prompt":v,
        "client_id":&_id.to_string()
    });

    let client = reqwest::blocking::Client::new();
    let _res = client
        .post(format!("http://{}/prompt", LCLHST))
        .body(params.to_string())
        .send();
    listen_to(_id);
}

fn resize_img(img_path: &PathBuf, size: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let reader = image::open(img_path).unwrap();
    let (w, h) = reader.dimensions();
    image::imageops::resize(
        &reader.to_rgba8(),
        size,
        (size * h) / w,
        FilterType::Lanczos3,
    )
}

fn image_processing(indir: &str, outdir: &str, size: u32) {
    let img_dir = fs::read_dir(indir).unwrap();
    for entry in img_dir.into_iter() {
        let img_path = entry.unwrap().path();
        let img_name = &img_path.file_name().to_owned();
        let out_image = Path::new(&outdir)
            .join(format!("new_{}", img_name.unwrap().to_str().unwrap()).to_string());
        println!(
            "save as :{}",
            out_image
                .file_name()
                .unwrap()
                .to_os_string()
                .to_string_lossy()
        );

        let img_result = resize_img(&img_path, size);
        img_result.save(out_image).unwrap();
    }
}

fn cli() -> Command {
    Command::new("rust_me")
        .allow_external_subcommands(true)
        .about(
            r#"

▄▄▄  ▄• ▄▌.▄▄ · ▄▄▄▄▄
▀▄ █·█▪██▌▐█ ▀. •██         some tools for :
▐▀▀▄ █▌▐█▌▄▀▀▀█▄ ▐█.▪       ..........ComfyUI API
▐█•█▌▐█▄█▌▐█▄▪▐█ ▐█▌·       ..........Image Resize
.▀  ▀ ▀▀▀  ▀▀▀▀  ▀▀▀"#,
        )
        .subcommand(
            Command::new("file")
                .about("inject ComfyUI workflow.json file using websocket")
                .arg(arg!(<JSONFILE> "the json file we are talking about")),
        )
        .subcommand(
            Command::new("img")
                .about(
                    r#"image processing stuff...
    -i --inputdir       image's input directory
    -o --outputdir      image's output directory
    -s --size           image width"#,
                )
                .args([
                    arg!(-i --inputdir <DIR> "image's input directory").required(true),
                    arg!(-o --outputdir <DIR> "image's output directory").required(true),
                    arg!(-s --size <DIR> "image width")
                        .value_parser(clap::value_parser!(u32).range(100..)),
                ]),
        )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("file", sub_mathces)) => {
            let workflow_file = sub_mathces.get_one::<String>("JSONFILE").unwrap();
            parse_json_file(Path::new(workflow_file));
        }
        Some(("img", sub_matches)) => {
            let input = sub_matches
                .get_one::<String>("inputdir")
                .map(|s| s.as_str())
                .unwrap_or_else(|| "");
            let output = sub_matches
                .get_one::<String>("outputdir")
                .map(|s| s.as_str())
                .unwrap_or_else(|| "");
            let size = sub_matches.get_one::<u32>("size").unwrap_or_else(|| &300);

            if !Path::new(input).exists() || !Path::new(output).exists() {
                cli().print_help().unwrap();
                return;
            }
            println!(
                r#"Processing images....

            inputdir        : {input}
            outputdir       : {output}
            size            : {size}
            
            "#
            );

            image_processing(input, output, *size);
        }
        _ => println!("{HELP}"),
    }
}
