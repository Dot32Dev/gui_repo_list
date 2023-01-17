use serde::Deserialize;
use iced::widget::{self, column, container, row, text, scrollable};
use iced::widget::scrollable::{Properties};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme, alignment,
};

pub fn main() -> iced::Result {
    RepoList::run(Settings::default())
}

struct RepoList {
    list: List,
}

enum List {
    Loading { username: String},
    Loaded { repositories: Repositories },
    Errored,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Repositories, Error>),
    Search(String),
}

impl Application for RepoList {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (RepoList, Command<Message>) {
        (
            RepoList {
                list: List::Loading { username: "dot32iscool".to_string()},
            },
            Command::perform(Repositories::search("dot32iscool".to_string()), Message::Loaded),
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
                List::Loading {username: _} => Command::none(),
                _ => {
                    self.list = List::Loading { username: username.clone()};

                    Command::perform(Repositories::search(username), Message::Loaded)
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.list {
            List::Loading {username: _} => {
                column![text("Loading...").size(40),]
                    .width(Length::Shrink)
            }
            List::Loaded { repositories } => column![
                repositories.view(),
                // text_button("Keep searching!").on_press(Message::Search)
            ]
            .spacing(20)
            .align_items(Alignment::End),
            List::Errored => column![
                text("Whoops! Something went wrong...").size(40),
                text_button("Try again").on_press(Message::Search("dot32iscool".to_string()))
            ]
            .spacing(20)
            .align_items(Alignment::End),
        };

        container(content)
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
}

#[derive(Deserialize, Debug, Clone)]
struct Repo {
    name: String,
    description: Option<String>,
    stargazers_count: u16,
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
        ).into()
    }

    async fn search(username: String) -> Result<Repositories, Error> {
        // Get repos from github api
        let res = reqwest::Client::new()
        .get(&format!("https://api.github.com/users/{}/repos?per_page=100", username))
        .header("User-Agent", "repo_list") // Required by github api
        .send().await?;
        
        // Parse response into a vector of repos
        let text = res.text().await?;
        let mut repos: Vec<Repo> = serde_json::from_str(&text).expect("Failed to parse repos");
        
        // sort repos by stargazer count
        repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));

        Ok(Repositories {
            list: repos,
        })
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

fn text_button(text: &str) -> widget::Button<'_, Message> {
    widget::button(text).padding(10)
}