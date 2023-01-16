use iced::futures;
use iced::widget::{self, column, container, image, row, text};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};

pub fn main() -> iced::Result {
    RepoList::run(Settings::default())
}

#[derive(Debug)]
enum RepoList {
    Loading,
    Loaded { repositories: Repositories },
    Errored,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Repositories, Error>),
    Search,
}

impl Application for RepoList {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (RepoList, Command<Message>) {
        (
            RepoList::Loading,
            Command::perform(Repositories::search(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        "Repo List".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(Ok(repositories)) => {
                *self = RepoList::Loaded { repositories };

                Command::none()
            }
            Message::Loaded(Err(_error)) => {
                *self = RepoList::Errored;

                Command::none()
            }
            Message::Search => match self {
                RepoList::Loading => Command::none(),
                _ => {
                    *self = RepoList::Loading;

                    Command::perform(Repositories::search(), Message::Loaded)
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self {
            RepoList::Loading => {
                column![text("Loading...").size(40),]
                    .width(Length::Shrink)
            }
            RepoList::Loaded { repositories } => column![
                repositories.view(),
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            RepoList::Errored => column![
                text("Whoops! Something went wrong...").size(40),
                button("Try again").on_press(Message::Search)
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
    number: u16,
    name: String,
    description: String,
}

impl Repositories {
    const TOTAL: u16 = 807;

    fn view(&self) -> Element<Message> {
        column![
            row![
                text(&self.name).size(30).width(Length::Fill),
                text(format!("{} Stars", self.number))
                    .size(20)
                    .style(Color::from([0.5, 0.5, 0.5])),
            ]
            .align_items(Alignment::Center)
            .spacing(20),
            self.description.as_ref(),
        ]
        .spacing(20)
        .align_items(Alignment::Start)
        .into()
    }

    async fn search() -> Result<Repositories, Error> {
        use rand::Rng;
        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        struct Repo {
            name: String,
            description: Option<String>,
            stargazers_count: u16,
        }

        
        // Get repos from github api
        let res = reqwest::Client::new()
        .get("https://api.github.com/users/Dot32IsCool/repos?per_page=100")
        .header("User-Agent", "repo_list") // Required by github api
        .send().await?;
        
        // Parse response into a vector of repos
        let text = res.text().await?;
        let mut repos: Vec<Repo> = serde_json::from_str(&text).expect("Failed to parse repos");
        
        // sort repos by stargazer count
        repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
        
        let rng = rand::thread_rng().gen_range(0..repos.len());

        Ok(Repositories {
            number: repos[rng].stargazers_count,
            name: repos[rng].name.clone(),
            description: repos[rng].description.clone().unwrap_or("No description".to_string()),
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

fn button(text: &str) -> widget::Button<'_, Message> {
    widget::button(text).padding(10)
}