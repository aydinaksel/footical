mod config;
mod database;
mod generate;
mod upload;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load()?;

    println!("Connecting to database...");
    let pool = database::connect(&config.database_url).await?;

    println!("Loading data...");
    let leagues = database::load_leagues(&pool).await?;
    let divisions = database::load_divisions(&pool).await?;
    let teams = database::load_teams(&pool).await?;
    let fixtures = database::load_fixtures(&pool).await?;
    println!(
        "  {} leagues, {} divisions, {} teams, {} fixtures",
        leagues.len(),
        divisions.len(),
        teams.len(),
        fixtures.len()
    );

    println!("Generating files into {}...", config.output_directory.display());
    generate::write_all(&config.output_directory, &leagues, &divisions, &teams, &fixtures)?;
    println!("  done");

    if let Some(r2) = &config.r2 {
        println!("Uploading to R2...");
        upload::upload_all(r2, &config.output_directory).await?;
        println!("  done");
    } else {
        println!("R2 not configured, skipping upload");
    }

    Ok(())
}
