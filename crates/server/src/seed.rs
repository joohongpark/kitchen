//! 기본 레시피 데이터 넣기. 같은 제목의 레시피가 있으면 건너뛴다.

use recipes::model::RecipeInput;

use crate::state::AppState;

const DEFAULT_SEED: &str = include_str!("../../../seed/recipes.json");

pub async fn run(state: &AppState, file: Option<&str>) -> anyhow::Result<()> {
    let raw = match file {
        Some(path) => std::fs::read_to_string(path)?,
        None => DEFAULT_SEED.to_string(),
    };
    let inputs: Vec<RecipeInput> = serde_json::from_str(&raw)?;
    let (mut created, mut skipped) = (0, 0);
    for input in inputs {
        let title = input.title.clone();
        match state.recipes.create(input).await {
            Ok(_) => {
                created += 1;
                println!("+ {title}");
            }
            Err(common::Error::AlreadyExists(_)) => {
                skipped += 1;
                println!("= {title} (이미 있음)");
            }
            Err(err) => return Err(anyhow::anyhow!("{title}: {err}")),
        }
    }
    println!("레시피 {created}개 추가, {skipped}개 건너뜀");
    Ok(())
}
