use serde::Deserialize;
use iced::widget::{column, container, row, text, scrollable, text_input, button, image};
use iced::widget::scrollable::{Properties};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme, alignment,
};
use iced::futures::StreamExt;

pub fn main() -> iced::Result {
    RepoList::run(Settings::default())
}

struct RepoList {
    input_value: String,
    list: List,
}

enum List {
    Loading,
    Loaded { repositories: Repositories },
    Errored,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Repositories, Error>),
    Search(String),
    InputChanged(String),
    OpenLink(String),
}

impl Application for RepoList {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (RepoList, Command<Message>) {
        (
            RepoList {
                input_value: "dot32iscool".to_string(),
                list: List::Loading,
            },
            Command::perform(Repositories::search("dot32iscool".to_string()), Message::Loaded),
            // Command::none(),
        )
    }

    fn title(&self) -> String {
        "Repo List".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(Ok(repositories)) => {
                self.list = List::Loaded { repositories };

                Command::none()
            }
            Message::Loaded(Err(_error)) => {
                self.list = List::Errored;

                Command::none()
            }
            Message::Search(username) => match self.list {
                List::Loading => Command::none(),
                _ => {
                    self.list = List::Loading;

                    Command::perform(Repositories::search(username), Message::Loaded)
                }
            }
            Message::InputChanged(input) => {
                self.input_value = input;
                Command::none()
            }
            Message::OpenLink(link) => {
                webbrowser::open(&link).unwrap();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.list {
            List::Loading => column![
                // column![text("Loading...").size(40),]
                //     .width(Length::Shrink)
                container(
                    text("Loading...")
                        .size(40)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
            ],
            List::Loaded { repositories } => column![
                repositories.view(),
            ]
            .spacing(20)
            .align_items(Alignment::End),
            List::Errored => column![
                container(
                    text("Could not find this user")
                        .size(40)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
                // text_button("Back to Dot32").on_press(Message::Search("dot32iscool".to_string()))
            ]
            .spacing(20)
            .align_items(Alignment::End),
        };

        container(column![
            container(row![
                text("@"),
                text_input(
                    "Search for a GitHub user",
                    &self.input_value,
                    Message::InputChanged,
                ).on_submit(Message::Search(self.input_value.clone())),
                button(text("Search")).on_press(Message::Search(self.input_value.clone()))
            ].align_items(Alignment::Center)).width(300.into()).padding(10),
            content
        ].align_items(Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

#[derive(Debug, Clone)]
struct Repositories {
    list: Vec<Repo>,
    avatar: image::Handle,
    username: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Repo {
    name: String,
    description: Option<String>,
    stargazers_count: u16,
}

#[derive(Deserialize, Debug, Clone)]
struct User {
    login: String,
    avatar_url: String,
}

impl Repositories {
    // const TOTAL: u16 = 807;

    fn view(&self) -> Element<Message> {
        // display repos in a column
        let mut repos = column![];
        // keep track of the index of the repo
        for (i, repo) in self.list.iter().enumerate() {
            repos = repos.push(
                container(
                    row![
                        // text(format!("{}-", i + 1))
                        //     .size(30)
                        //     .style(Color::from([0.5, 0.5, 0.5])),
                        // print index with padded 0s
                        text(format!("{:0>2}", i + 1))
                            .size(30)
                            .style(Color::from([0.5, 0.5, 0.5])),
                        column!(
                            text(&repo.name)
                                .size(30)
                                .width(Length::Fill)
                                .style(if &repo.name == "gui_repo_list" { Color::from([0.5, 0.5, 0.5])} else { Color::from([0.8, 0.8, 0.8]) }),
                            text(&repo.description.clone().unwrap_or("No description".to_string()))
                                .size(20)
                                .style(Color::from([0.5, 0.5, 0.5]))
                        ).width(Length::Fill).spacing(5),
                        text(format!("{} Stars", repo.stargazers_count))
                            .size(20)
                            .style(Color::from([0.5, 0.5, 0.5])),
                    ]
                    // .align_items(Alignment::Center)
                    .spacing(20)
                )
                .max_width(500)
                .padding(20),
            );
        };

        // repos.into()

        column![
            container(row![
                image(self.avatar.clone())
                    .width(Length::Units(50))
                    .height(Length::Units(50)),
                text(&self.username).size(20),
                //open link to github profile
                button(text("Open Profile"))
                    .on_press(Message::OpenLink(format!("https://github.com/{}", self.username)))
            ].align_items(Alignment::Center).spacing(20)).align_x(alignment::Horizontal::Center).width(Length::Fill),
            scrollable(
                container(repos)
                .padding(20)
                .width(Length::Fill)
                // ALIGNMENT
                .align_x(alignment::Horizontal::Center)
            )
            .vertical_scroll(
                Properties::new()
                    .width(10)
                    .margin(0)
                    .scroller_width(5),
            )
            .height(Length::Fill)
        ]
        .into()
    }

    async fn search(username: String) -> Result<Repositories, Error> {
        // Get repos from github api
        let res = reqwest::Client::new()
        .get(&format!("https://api.github.com/users/{}/repos?per_page=100", username))
        .header("User-Agent", "repo_list") // Required by github api
        .send().await?;

        // // Steam response
        // let mut text = String::new();
        // let mut stream = res.bytes_stream();
        // while let Some(item) = stream.next().await {
        //     let chunk = item?;
        //     text.push_str(&String::from_utf8(chunk.to_vec()).unwrap());
        //     println!("Loaded {} characters", text.len());
        // }
        
        // Parse response into a vector of repos
        let text = res.text().await?;
        // let mut repos: Vec<Repo> = serde_json::from_str(&text).expect("Failed to parse repos");
        let mut repos: Vec<Repo> = match serde_json::from_str(&text) {
            Ok(repos) => repos,
            Err(_) => return Err(Error::APIError),
        };
        
        // sort repos by stargazer count
        repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));

        // Get user info from github api
        let res = reqwest::Client::new()
        .get(&format!("https://api.github.com/users/{}", username))
        .header("User-Agent", "repo_list") // Required by github api
        .send().await?;

        // Parse response into a user
        let text = res.text().await?;
        println!("{}", text);
        let user: User = match serde_json::from_str(&text) {
            Ok(user) => user,
            Err(_) => return Err(Error::APIError),
        };

        println!("{}", user.avatar_url);

        // Get user avatar
        let avatar = Self::fetch_image(user.avatar_url).await?;

        Ok(Repositories {
            list: repos,
            avatar: avatar,
            username: user.login,
        })
    }

    async fn fetch_image(url: String) -> Result<image::Handle, reqwest::Error> {
        let bytes = reqwest::get(&url).await?.bytes().await?;
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        dbg!(error);

        Error::APIError
    }
}