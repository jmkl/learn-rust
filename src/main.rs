use clap::{command, Arg};

use image::imageops::FilterType;
use image::{GenericImageView, ImageBuffer, Rgba};
use reqwest;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::{env, fs};
use tungstenite::connect;
use url::Url;
use uuid::Uuid;

static LCLHST: &str = "127.0.0.1:8188";

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

fn parse_json_file(file_path: &Path, _id: Uuid) {
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

fn resize_img(img_path: &PathBuf) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let reader = image::open(img_path).unwrap();
    let (w, h) = reader.dimensions();
    image::imageops::resize(&reader.to_rgba8(), 300, (300 * h) / w, FilterType::Lanczos3)
}

fn image_processing() {
    let path = std::env::current_dir().unwrap();
    // let img_path = Path::join(&path, "img/test.png");
    // let img_result = resize_img(&img_path);
    // img_result.save(Path::join(&path, "out/out.png")).unwrap();

    let img_path = Path::join(&path, "img");
    let img_dir = fs::read_dir(img_path).unwrap();
    // let files = img_dir
    //     .map(|entry| {
    //         let entry = entry.unwrap();
    //         let entry_path = entry.path();
    //         let file_name = entry_path.file_name().unwrap();
    //         let file_name_as_str = file_name.to_str().unwrap();
    //         let file_name_as_string = String::from(file_name_as_str);
    //         file_name_as_string
    //     })
    //     .collect::<Vec<String>>();
    for entry in img_dir.into_iter() {
        let img_path = entry.unwrap().path();
        let img_name = &img_path.file_name().to_owned();
        let out_image = Path::new(&path)
            .join("out")
            .join(format!("new_{}", img_name.unwrap().to_str().unwrap()).to_string());
        println!("{:?}", out_image);

        let img_result = resize_img(&img_path);
        img_result.save(out_image).unwrap();
    }
    //println!("{:?}", files);
}

fn do_stuff() {
    let _id = Uuid::new_v4();
    let matches = command!()
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .required(true)
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .get_matches();

    let file_path = matches.get_one::<std::path::PathBuf>("file").unwrap();
    let workflow_file = Path::new(&file_path);
    if workflow_file.exists() {
        parse_json_file(workflow_file, _id);
    } else {
        println!("corrupted file!... please try again");
    }
}

fn main() {
    image_processing();
}
