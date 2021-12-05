use crate::app_state::App;

use std::io;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::{AlternateScreen, ToMainScreen};

use tokio::sync::mpsc;
use tokio::time::interval;

use tui::backend::{Backend, TermionBackend};
use tui::style::{Color, Style};
use tui::symbols;
use tui::widgets::{BarChart, Block, Borders};
use tui::Frame;
use tui::Terminal;

enum Event {
    Input(Key),
    Quit,
    Tick,
}

pub async fn run(app: &App) -> anyhow::Result<()> {
    // setup terminal
    let stdout = io::stdout().into_raw_mode()?;
    //let stdout = MouseTerminal::from(stdout);
    let alt_stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(alt_stdout);
    let mut terminal = Terminal::new(backend)?;

    let (events_tx, mut events_rx) = events_listener();
    let key_tx = events_tx.clone();
    loop {
        terminal.draw(|f| draw(f, app))?;

        match events_rx.recv().await {
            Some(Event::Input(Key::Char(c))) => on_key(&key_tx, c).await?,
            Some(Event::Input(_)) => (),
            Some(Event::Tick) => on_tick(),
            Some(Event::Quit) => {
                print!("{}", ToMainScreen);
                terminal.show_cursor()?;
                return Ok(());
            }
            None => (),
        }
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let title = format!(
        "Topic: {}, Filter: {}",
        app.cli_options().topic,
        app.cli_options().filter
    );
    let partition_counts = app.partition_counts().unwrap();
    let mut partition_counts: Vec<(String, u64)> = partition_counts
        .iter()
        .map(|e| (e.key().to_owned(), *e.value()))
        .collect();
    partition_counts.sort();
    let partition_counts: Vec<(&str, u64)> = partition_counts
        .iter()
        .map(|e| (e.0.as_str(), e.1))
        .collect();

    let barchart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .data(&partition_counts)
        .bar_width(3)
        .bar_gap(2)
        .bar_set(symbols::bar::NINE_LEVELS)
        .value_style(Style::default().fg(Color::Black).bg(Color::Green))
        .label_style(Style::default().fg(Color::Yellow))
        .bar_style(Style::default().fg(Color::Green));

    f.render_widget(barchart, f.size());
}

fn events_listener() -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
    let (tx, rx) = mpsc::channel(32);
    let tick_tx = tx.clone();
    let keys_tx = tx.clone();

    tokio::spawn(async move {
        let stdin = io::stdin();

        for event in stdin.keys().flatten() {
            if let Err(err) = keys_tx.send(Event::Input(event)).await {
                eprintln!("{}", err);
                break;
            }
        }
    });

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(250));

        loop {
            interval.tick().await;

            if let Err(err) = tick_tx.send(Event::Tick).await {
                eprintln!("{}", err);
                break;
            }
        }
    });

    (tx, rx)
}

async fn on_key(key_tx: &mpsc::Sender<Event>, c: char) -> anyhow::Result<()> {
    match c {
        'q' | 'Q' => Ok(key_tx
            .send(Event::Quit)
            .await
            .map_err(|_| anyhow::anyhow!("Error sending to tx channel"))?),
        _ => Ok(()),
    }
}

fn on_tick() {}
