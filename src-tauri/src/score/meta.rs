use crate::{utils::now, AppState};
use serde::{Deserialize, Serialize};
use tauri::State;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    app_version: String,
    title: String,
    subtitle: String,
    dedication: String,
    composer: String,
    arranger: String,
    lyricist: String,
    artist: String,
    copyist: String,
    publisher: String,
    editor: String,
    copyright: String,
    work_number: String,
    composition_year: String,
    composer_dates: String,
    other_information: String,
    created: u128,
}

impl Meta {
    pub fn new() -> Self {
        Meta {
            app_version: String::from(VERSION),
            title: String::from(""),
            subtitle: String::from(""),
            dedication: String::from(""),
            composer: String::from(""),
            arranger: String::from(""),
            lyricist: String::from(""),
            artist: String::from(""),
            copyist: String::from(""),
            publisher: String::from(""),
            editor: String::from(""),
            copyright: String::from(""),
            work_number: String::from(""),
            composition_year: String::from(""),
            composer_dates: String::from(""),
            other_information: String::from(""),
            created: now(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_title_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    title: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.title = title;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_subtitle_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    subtitle: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.subtitle = subtitle;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_dedication_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    dedication: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.dedication = dedication;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_composer_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    composer: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.composer = composer;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_arranger_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    arranger: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.arranger = arranger;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_lyricist_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    lyricist: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.lyricist = lyricist;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_artist_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    artist: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.artist = artist;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_copyist_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    copyist: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.copyist = copyist;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_publisher_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    publisher: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.publisher = publisher;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_editor_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    editor: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.editor = editor;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_copyright_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    copyright: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.copyright = copyright;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_work_number_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    work_number: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.work_number = work_number;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_composition_year_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    composition_year: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.composition_year = composition_year;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_composer_dates_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    composer_dates: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.composer_dates = composer_dates;
    state.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn score_meta_other_information_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    other_information: String,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.score.meta.other_information = other_information;
    state.emit(&app);

    Ok(())
}
