use iced::futures;
use iced::widget::{self, column, container, image, row, text};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};

pub fn main() -> iced::Result {
    Pokedex::run(Settings::default())
}

#[derive(Debug)]
enum Pokedex {
    Loading,
    Loaded { pokemon: Pokemon },
    Errored,
}

#[derive(Debug, Clone)]
enum Message {
    PokemonFound(Result<Pokemon, Error>),
    Search,
}

impl Application for Pokedex {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Pokedex, Command<Message>) {
        (
            Pokedex::Loading,
            Command::perform(Pokemon::search(), Message::PokemonFound),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Pokedex::Loading => "Loading",
            Pokedex::Loaded { pokemon, .. } => &pokemon.name,
            Pokedex::Errored { .. } => "Whoops!",
        };

        format!("{} - Pokédex", subtitle)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PokemonFound(Ok(pokemon)) => {
                *self = Pokedex::Loaded { pokemon };

                Command::none()
            }
            Message::PokemonFound(Err(_error)) => {
                *self = Pokedex::Errored;

                Command::none()
            }
            Message::Search => match self {
                Pokedex::Loading => Command::none(),
                _ => {
                    *self = Pokedex::Loading;

                    Command::perform(Pokemon::search(), Message::PokemonFound)
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self {
            Pokedex::Loading => {
                column![text("Searching for Pokémon...").size(40),]
                    .width(Length::Shrink)
            }
            Pokedex::Loaded { pokemon } => column![
                pokemon.view(),
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Pokedex::Errored => column![
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
}

#[derive(Debug, Clone)]
struct Pokemon {
    number: u16,
    name: String,
    description: String,
}

impl Pokemon {
    const TOTAL: u16 = 807;

    fn view(&self) -> Element<Message> {
        column![
            row![
                text(&self.name).size(30).width(Length::Fill),
                text(format!("#{}", self.number))
                    .size(20)
                    .style(Color::from([0.5, 0.5, 0.5])),
            ]
            .align_items(Alignment::Center)
            .spacing(20),
            self.description.as_ref(),
        ]
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
    }

    async fn search() -> Result<Pokemon, Error> {
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

        Ok(Pokemon {
            number: repos[0].stargazers_count,
            name: repos[0].name.to_uppercase(),
            description: repos[0].description.clone().unwrap_or("No description".to_string()),
        })
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
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