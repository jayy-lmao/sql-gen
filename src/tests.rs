#[tokio::test]
async fn test_basic_postgres_tables() -> Result<(), Box<dyn Error>> {
    let pool = setup_pg_db().await;

    Ok(())
}
