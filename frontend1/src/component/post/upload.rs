use alloc::{string::String, vec::Vec};

use yew::{
    html,
    services::{
        fetch::FetchTask,
        reader::{File, FileChunk, FileData, ReaderService, ReaderTask},
    },
    ChangeData, Component, ComponentLink, Html, ShouldRender,
};

use blog_common::dto::post::UploadImage;

use crate::{
    util::{request, Error},
    val,
};

struct UploadFileStatus {
    filename: String,
    remote_path: String,
    is_uploaded: bool,
}

pub struct Model {
    fetch_task: Vec<Option<FetchTask>>,
    link: ComponentLink<Model>,
    // reader: ReaderService,
    tasks: Vec<ReaderTask>,
    choose_files: Vec<File>,
    uploading_files: Vec<UploadFileStatus>,
}

pub enum Msg {
    Loaded(FileData),
    AppendFiles(Vec<File>),
    Upload,
    Response(Result<UploadImage, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            fetch_task: Vec::new(),
            // reader: ReaderService::new(),
            link,
            tasks: Vec::new(),
            choose_files: Vec::new(),
            uploading_files: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Loaded(file) => {
                let mut url = String::from(val::BLOG_IMAGE_SAVE_URI);
                url.push_str(file.name.as_str());
                self.uploading_files.push(UploadFileStatus {
                    filename: file.name,
                    remote_path: String::new(),
                    is_uploaded: false,
                });
                self.fetch_task.push(Some(request::post_binary(
                    url.as_str(),
                    file.content,
                    self.link.callback(Msg::Response),
                )));
                false
            },
            Msg::AppendFiles(files) => {
                self.choose_files.clear();
                self.choose_files.extend(files);
                false
            },
            Msg::Upload => {
                let files = self.choose_files.to_owned();
                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::Loaded);
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.push(task);
                }
                self.choose_files.clear();
                false
            },
            Msg::Response(Ok(i)) => {
                // self.fetch_task = None;
                for f in self.uploading_files.iter_mut() {
                    if f.filename.eq(&i.original_filename) {
                        f.remote_path = i.path.to_owned();
                        f.is_uploaded = true;
                        break;
                    }
                }
                true
            },
            Msg::Response(Err(e)) => true,
        }
    }

    fn view(&self) -> Html {
        html! {
            <>
                <div>
                    <input type="file" multiple=true accept="image/*" onchange={self.link.callback(move |value| {
                            let mut result = Vec::new();
                            if let ChangeData::Files(files) = value {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .into_iter()
                                    .map(|v| File::from(v.unwrap()));
                                result.extend(files);
                            }
                            Msg::AppendFiles(result)
                        })}/>
                    <button type="button" onclick={self.link.callback(|_| Msg::Upload)}>
                        { "Upload" }
                    </button>
                </div>
                <ul>
                    {
                        for self.uploading_files.iter().map(|f| {
                            if f.is_uploaded {
                                html!{
                                    <li>{"!["}{ &f.filename }{"]("}{ &f.remote_path }{" \""}{ &f.filename }{"\")"}</li>
                                }
                            } else {
                                html! {
                                    <li>{"Uploading: "}{ &f.filename }</li>
                                }
                            }
                        })
                    }
                </ul>
            </>
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }
}
